//! Python Adapter 子进程监督。
//!
//! 单应用容器与本地 `all` 角色由 Rust 主进程启动 Adapter：
//! - 启动后等待 Adapter 健康检查通过（契约版本一致）才认为就绪；
//! - Adapter 意外退出时触发整体退出，避免容器“看似正常却无法调用”；
//! - 退出时最后停止 Adapter：先 SIGTERM，超时后强制结束并回收。

use std::time::Duration;

use anyhow::{Context, bail};
use tokio::process::{Child, Command};
use tokio_util::sync::CancellationToken;
use zenrouter_integrations::AdapterClient;

use crate::config::AdapterConfig;

pub struct AdapterProcess {
    child: Child,
    stop_timeout: Duration,
}

impl AdapterProcess {
    pub fn spawn(config: &AdapterConfig) -> anyhow::Result<Self> {
        let (program, args) = config
            .command
            .split_first()
            .context("adapter.command 为空")?;
        let mut cmd = Command::new(program);
        cmd.args(args).kill_on_drop(true);
        if let Some(dir) = &config.workdir {
            cmd.current_dir(dir);
        }
        let child = cmd
            .spawn()
            .with_context(|| format!("无法启动 Adapter 进程：{program}"))?;
        tracing::info!(pid = child.id(), program = %program, "Adapter 子进程已启动");
        Ok(Self {
            child,
            stop_timeout: Duration::from_secs(config.stop_timeout_secs),
        })
    }

    /// 轮询健康检查直到通过；期间子进程退出或超时都视为启动失败。
    pub async fn wait_ready(
        &mut self,
        client: &AdapterClient,
        timeout: Duration,
    ) -> anyhow::Result<()> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Some(status) = self.child.try_wait()? {
                bail!("Adapter 在就绪前退出：{status}");
            }
            match client.health().await {
                Ok(()) => {
                    tracing::info!(url = client.base_url(), "Adapter 已就绪");
                    return Ok(());
                }
                Err(err) if tokio::time::Instant::now() >= deadline => {
                    bail!("Adapter 在 {timeout:?} 内未就绪：{err:?}");
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(300)).await,
            }
        }
    }

    /// 监视子进程；它意外退出时取消 `fatal`，让主进程以失败状态整体退出。
    /// 收到 `stop` 后执行有序停止并返回。
    pub async fn supervise(mut self, stop: CancellationToken, fatal: CancellationToken) {
        tokio::select! {
            status = self.child.wait() => {
                match status {
                    Ok(s) => tracing::error!(status = %s, "Adapter 进程意外退出，开始整体退出"),
                    Err(e) => tracing::error!(error = %e, "等待 Adapter 进程失败，开始整体退出"),
                }
                fatal.cancel();
            }
            _ = stop.cancelled() => self.terminate().await,
        }
    }

    async fn terminate(&mut self) {
        terminate_gracefully(&mut self.child);
        match tokio::time::timeout(self.stop_timeout, self.child.wait()).await {
            Ok(Ok(status)) => tracing::info!(%status, "Adapter 已停止"),
            Ok(Err(e)) => tracing::warn!(error = %e, "等待 Adapter 退出失败"),
            Err(_) => {
                tracing::warn!("Adapter 未在宽限时间内退出，强制结束");
                let _ = self.child.kill().await;
            }
        }
    }
}

#[cfg(unix)]
fn terminate_gracefully(child: &mut Child) {
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;
    if let Some(pid) = child.id().and_then(|p| i32::try_from(p).ok()) {
        let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);
    }
}

#[cfg(not(unix))]
fn terminate_gracefully(child: &mut Child) {
    let _ = child.start_kill();
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn cfg(cmd: &[&str]) -> AdapterConfig {
        AdapterConfig {
            command: cmd.iter().map(|s| s.to_string()).collect(),
            stop_timeout_secs: 2,
            ..AdapterConfig::default()
        }
    }

    #[tokio::test]
    async fn unexpected_exit_triggers_shutdown() {
        let proc = AdapterProcess::spawn(&cfg(&["sh", "-c", "exit 3"])).unwrap();
        let shutdown = CancellationToken::new();
        tokio::time::timeout(
            Duration::from_secs(5),
            proc.supervise(CancellationToken::new(), shutdown.clone()),
        )
        .await
        .unwrap();
        assert!(shutdown.is_cancelled());
    }

    #[tokio::test]
    async fn stop_terminates_child_without_triggering_shutdown() {
        let proc = AdapterProcess::spawn(&cfg(&["sleep", "30"])).unwrap();
        let stop = CancellationToken::new();
        let shutdown = CancellationToken::new();
        stop.cancel();
        tokio::time::timeout(
            Duration::from_secs(5),
            proc.supervise(stop, shutdown.clone()),
        )
        .await
        .unwrap();
        assert!(!shutdown.is_cancelled());
    }

    #[tokio::test]
    async fn wait_ready_fails_when_child_exits() {
        let mut proc = AdapterProcess::spawn(&cfg(&["sh", "-c", "exit 1"])).unwrap();
        let client = AdapterClient::new("http://127.0.0.1:9", "0.1").unwrap();
        let err = proc
            .wait_ready(&client, Duration::from_secs(5))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("退出"));
    }
}
