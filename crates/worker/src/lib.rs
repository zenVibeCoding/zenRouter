//! 后台任务入口：可靠任务、正文归档、结算恢复、风险重扫、汇总、对账及导出。
//!
//! 首期采用 PostgreSQL 任务表／outbox，限量领取（SKIP LOCKED）与租约重试。
//! 骨架阶段只提供可取消的空轮询循环，便于验证进程监督与优雅退出。

use std::time::Duration;

use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub poll_interval: Duration,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),
        }
    }
}

/// 运行任务循环直到收到取消信号。取消时先停止领取新任务，再返回。
pub async fn run(config: WorkerConfig, shutdown: CancellationToken) {
    tracing::info!(
        poll_interval_ms = config.poll_interval.as_millis() as u64,
        "worker 启动"
    );
    let mut ticker = tokio::time::interval(config.poll_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = ticker.tick() => {
                // TODO(工作包 B/C)：领取 outbox 任务、正文封存、结算恢复。
                tracing::trace!("worker 轮询（骨架阶段暂无任务）");
            }
        }
    }
    tracing::info!("worker 已停止领取任务");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stops_when_cancelled() {
        let token = CancellationToken::new();
        let handle = tokio::spawn(run(
            WorkerConfig {
                poll_interval: Duration::from_millis(10),
            },
            token.clone(),
        ));
        tokio::time::sleep(Duration::from_millis(30)).await;
        token.cancel();
        tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("worker 应在取消后退出")
            .unwrap();
    }
}
