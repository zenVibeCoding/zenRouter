//! 运行探针：存活与就绪分离。
//!
//! 存活（liveness）只说明进程能响应；就绪（readiness）按角色检查必要依赖与排空状态。
//! 存活检查不因数据库短暂故障而失败，避免编排平台重启全部副本。

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use zenrouter_contracts::{CheckStatus, DependencyReport, ReadinessReport};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// 一项外部依赖的就绪检查，由 storage / integrations 实现。
pub trait DependencyCheck: Send + Sync {
    fn name(&self) -> &'static str;
    fn check(&self) -> BoxFuture<'_, DependencyReport>;
}

/// 某个运行角色的就绪状态：依赖检查 + 排空标记。
#[derive(Clone, Default)]
pub struct Readiness {
    checks: Vec<Arc<dyn DependencyCheck>>,
    draining: Arc<AtomicBool>,
}

impl Readiness {
    pub fn new(checks: Vec<Arc<dyn DependencyCheck>>) -> Self {
        Self {
            checks,
            draining: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 与另一组检查共享同一个排空标记（同一进程内多个角色一起排空）。
    pub fn with_checks(&self, checks: Vec<Arc<dyn DependencyCheck>>) -> Self {
        Self {
            checks,
            draining: Arc::clone(&self.draining),
        }
    }

    /// 进入排空：就绪探针立即失败，使入口停止分配新请求。
    pub fn start_draining(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }

    pub async fn evaluate(&self) -> ReadinessReport {
        let mut dependencies = Vec::with_capacity(self.checks.len());
        for check in &self.checks {
            dependencies.push(check.check().await);
        }
        let draining = self.is_draining();
        let deps_ok = dependencies
            .iter()
            .all(|d| matches!(d.status, CheckStatus::Ok | CheckStatus::Skipped));
        ReadinessReport {
            ready: deps_ok && !draining,
            draining,
            dependencies,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed(&'static str, CheckStatus);

    impl DependencyCheck for Fixed {
        fn name(&self) -> &'static str {
            self.0
        }
        fn check(&self) -> BoxFuture<'_, DependencyReport> {
            Box::pin(async move {
                DependencyReport {
                    name: self.0.into(),
                    status: self.1,
                    detail: None,
                }
            })
        }
    }

    #[tokio::test]
    async fn ready_only_when_all_dependencies_ok_and_not_draining() {
        let r = Readiness::new(vec![
            Arc::new(Fixed("a", CheckStatus::Ok)),
            Arc::new(Fixed("b", CheckStatus::Skipped)),
        ]);
        assert!(r.evaluate().await.ready);

        r.start_draining();
        let report = r.evaluate().await;
        assert!(!report.ready);
        assert!(report.draining);
    }

    #[tokio::test]
    async fn unavailable_dependency_is_not_ready() {
        let r = Readiness::new(vec![Arc::new(Fixed("db", CheckStatus::Unavailable))]);
        assert!(!r.evaluate().await.ready);
    }

    #[tokio::test]
    async fn derived_readiness_shares_draining_flag() {
        let base = Readiness::new(vec![]);
        let derived = base.with_checks(vec![]);
        base.start_draining();
        assert!(derived.is_draining());
    }
}
