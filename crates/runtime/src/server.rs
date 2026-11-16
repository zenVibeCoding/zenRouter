//! 依赖装配、HTTP 监听与优雅退出编排。
//!
//! 退出顺序：标记排空（就绪探针失败）→ 停止接收新请求并等待存量请求（有宽限上限）
//! → 停止 worker 领取 → 最后停止 Adapter 子进程。

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use zenrouter_application::{DependencyCheck, Readiness};
use zenrouter_contracts::ADAPTER_CONTRACT_VERSION;
use zenrouter_control_api::ControlState;
use zenrouter_integrations::{AdapterCheck, AdapterClient};
use zenrouter_storage::{ContentDirCheck, PostgresCheck};

use crate::config::Config;
use crate::role::{Role, RoleSet};
use crate::supervisor::AdapterProcess;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn serve(config: Config, roles: RoleSet) -> anyhow::Result<()> {
    tracing::info!(version = VERSION, %roles, "ZenRouter 启动");

    // 外部退出信号（正常退出）与关键组件失败（非零退出）分开表示。
    let shutdown = CancellationToken::new();
    let fatal = CancellationToken::new();
    tokio::spawn(forward_signals(shutdown.clone()));

    // ---- 共享依赖 ----
    let db_url = config.database.url.as_deref().unwrap_or_default();
    let pool = zenrouter_storage::connect_lazy(db_url, config.database.max_connections)?;
    let postgres: Arc<dyn DependencyCheck> = Arc::new(PostgresCheck::new(pool));
    let base = Readiness::new(vec![]);

    // ---- Adapter（可选监督）----
    let adapter_client = AdapterClient::new(&config.adapter.url, ADAPTER_CONTRACT_VERSION)
        .context("创建 Adapter 客户端失败")?;
    let adapter_stop = CancellationToken::new();
    let mut adapter_task = None;
    if roles.supervises_adapter(&config) {
        let mut process = AdapterProcess::spawn(&config.adapter)?;
        // 未就绪时直接返回，子进程随 `kill_on_drop` 一起回收。
        process
            .wait_ready(&adapter_client, Duration::from_secs(30))
            .await?;
        adapter_task = Some(tokio::spawn(
            process.supervise(adapter_stop.clone(), fatal.clone()),
        ));
    }

    // ---- 各角色 ----
    let mut servers = JoinSet::new();
    let http_shutdown = CancellationToken::new();
    let grace = Duration::from_secs(config.server.shutdown_grace_secs);

    if roles.contains(Role::Gateway) {
        let readiness = base.with_checks(vec![
            Arc::clone(&postgres),
            Arc::new(AdapterCheck::new(adapter_client.clone())),
            Arc::new(ContentDirCheck::new(config.content_dir())),
        ]);
        let app = zenrouter_gateway::router().merge(probes(readiness));
        spawn_http(
            &mut servers,
            "gateway",
            config.server.gateway_listen,
            app,
            &http_shutdown,
        )
        .await?;
    }

    if roles.contains(Role::Control) {
        let readiness = base.with_checks(vec![Arc::clone(&postgres)]);
        let mut app = zenrouter_control_api::router(ControlState {
            version: VERSION,
            roles: roles.names(),
        })
        .merge(probes(readiness));
        match zenrouter_control_api::console_service(config.server.console_dir.clone()) {
            Some(console) => app = app.merge(console),
            None => tracing::info!(
                dir = %config.server.console_dir.display(),
                "未找到控制台构建产物，不托管页面（开发期请使用 Vite 开发服务器）"
            ),
        }
        spawn_http(
            &mut servers,
            "control",
            config.server.control_listen,
            app,
            &http_shutdown,
        )
        .await?;
    }

    let worker_stop = CancellationToken::new();
    let mut worker_task = None;
    if roles.contains(Role::Worker) {
        if !roles.has_http_role() {
            // 独立 worker 也需要探针端口，供编排平台判断存活与就绪。
            let readiness = base.with_checks(vec![Arc::clone(&postgres)]);
            spawn_http(
                &mut servers,
                "worker-probe",
                config.server.probe_listen,
                probes(readiness),
                &http_shutdown,
            )
            .await?;
        }
        worker_task = Some(tokio::spawn(zenrouter_worker::run(
            zenrouter_worker::WorkerConfig::default(),
            worker_stop.clone(),
        )));
    }

    // ---- 等待退出 ----
    // 外部信号属于正常退出；关键组件意外结束则以非零码退出，便于编排平台识别并重启。
    let failure = tokio::select! {
        _ = shutdown.cancelled() => None,
        _ = fatal.cancelled() => Some("Adapter 进程意外退出"),
        Some(res) = servers.join_next() => {
            tracing::error!(?res, "HTTP 服务意外结束，开始整体退出");
            Some("HTTP 服务意外结束")
        }
    };

    tracing::info!(grace_secs = grace.as_secs(), "开始排空");
    base.start_draining();
    http_shutdown.cancel();
    if tokio::time::timeout(grace, async {
        while servers.join_next().await.is_some() {}
    })
    .await
    .is_err()
    {
        tracing::warn!("存量请求未在宽限时间内结束，强制关闭");
        servers.abort_all();
    }

    worker_stop.cancel();
    if let Some(task) = worker_task {
        let _ = task.await;
    }

    adapter_stop.cancel();
    if let Some(task) = adapter_task {
        let _ = task.await;
    }

    match failure {
        None => {
            tracing::info!("ZenRouter 已退出");
            Ok(())
        }
        Some(reason) => anyhow::bail!("ZenRouter 因{reason}而退出"),
    }
}

async fn spawn_http(
    set: &mut JoinSet<std::io::Result<()>>,
    name: &'static str,
    addr: std::net::SocketAddr,
    app: Router,
    shutdown: &CancellationToken,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("{name} 无法监听 {addr}"))?;
    tracing::info!(role = name, %addr, "开始监听");
    let token = shutdown.clone();
    set.spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move { token.cancelled().await })
            .await
    });
    Ok(())
}

/// 存活与就绪探针。存活不检查外部依赖。
fn probes(readiness: Readiness) -> Router {
    Router::new()
        .route("/livez", get(|| async { "ok" }))
        .route("/readyz", get(ready))
        .with_state(readiness)
}

async fn ready(State(readiness): State<Readiness>) -> impl IntoResponse {
    let report = readiness.evaluate().await;
    let status = if report.ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(report))
}

async fn forward_signals(shutdown: CancellationToken) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "无法注册 SIGTERM 处理");
                return;
            }
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => tracing::info!("收到 SIGINT"),
            _ = term.recv() => tracing::info!("收到 SIGTERM"),
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("收到 Ctrl-C");
    }
    shutdown.cancel();
}
