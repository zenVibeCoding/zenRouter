use std::time::Duration;

use anyhow::Context;
use sqlx::postgres::{PgPool, PgPoolOptions};
use zenrouter_application::health::DependencyReport;
use zenrouter_application::{BoxFuture, DependencyCheck};

/// 建立惰性连接池：进程启动不因数据库暂时不可用而失败，由就绪探针反映状态。
pub fn connect_lazy(url: &str, max_connections: u32) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(3))
        .connect_lazy(url)
        .context("PostgreSQL 连接串无效")
}

/// 执行仓库内 `migrations/` 目录的迁移。应由单独的 `migrate` 角色执行，不让每个副本同时迁移。
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .context("数据库迁移失败")
}

pub struct PostgresCheck {
    pool: PgPool,
}

impl PostgresCheck {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DependencyCheck for PostgresCheck {
    fn name(&self) -> &'static str {
        "postgres"
    }

    fn check(&self) -> BoxFuture<'_, DependencyReport> {
        Box::pin(async move {
            match sqlx::query_scalar::<_, i32>("SELECT 1")
                .fetch_one(&self.pool)
                .await
            {
                Ok(_) => DependencyReport::ok(self.name()),
                Err(err) => {
                    // 只记录错误类别，避免把连接串等敏感信息带进探针响应。
                    tracing::warn!(error.kind = %error_kind(&err), "PostgreSQL 就绪检查失败");
                    DependencyReport::unavailable(self.name(), error_kind(&err))
                }
            }
        })
    }
}

fn error_kind(err: &sqlx::Error) -> &'static str {
    match err {
        sqlx::Error::PoolTimedOut => "连接池获取超时",
        sqlx::Error::Io(_) => "网络或连接错误",
        sqlx::Error::Tls(_) => "TLS 错误",
        sqlx::Error::Database(_) => "数据库返回错误",
        sqlx::Error::Configuration(_) => "连接配置错误",
        _ => "未知错误",
    }
}
