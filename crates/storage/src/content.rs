//! 正文文件后端（首期：企业指定目录，JSONL 追加写入；Parquet 封存后续加入）。
//!
//! 骨架阶段只检查目录是否可写。真正的写入需要验证文件同步、目录元数据与崩溃恢复语义，
//! 见 docs/technical-architecture.md 第 8 节与验证门槛 C。

use std::path::PathBuf;

use zenrouter_application::health::DependencyReport;
use zenrouter_application::{BoxFuture, DependencyCheck};

/// 正文目录就绪检查。`dir` 为 `None` 表示当前政策不留正文，检查跳过。
pub struct ContentDirCheck {
    dir: Option<PathBuf>,
}

impl ContentDirCheck {
    pub fn new(dir: Option<PathBuf>) -> Self {
        Self { dir }
    }
}

impl DependencyCheck for ContentDirCheck {
    fn name(&self) -> &'static str {
        "content_store"
    }

    fn check(&self) -> BoxFuture<'_, DependencyReport> {
        Box::pin(async move {
            let Some(dir) = self.dir.clone() else {
                return DependencyReport::skipped(self.name(), "未启用正文留存");
            };
            let probe = dir.join(format!(".zenrouter-probe-{}", uuid::Uuid::now_v7()));
            let result = tokio::task::spawn_blocking(move || {
                std::fs::write(&probe, b"")?;
                std::fs::remove_file(&probe)
            })
            .await;
            match result {
                Ok(Ok(())) => DependencyReport::ok(self.name()),
                Ok(Err(_)) | Err(_) => DependencyReport::unavailable(self.name(), "正文目录不可写"),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zenrouter_application::health::CheckStatus;

    #[tokio::test]
    async fn skipped_when_retention_disabled() {
        let r = ContentDirCheck::new(None).check().await;
        assert_eq!(r.status, CheckStatus::Skipped);
    }

    #[tokio::test]
    async fn writable_dir_is_ok_and_missing_dir_is_unavailable() {
        let tmp = std::env::temp_dir();
        assert_eq!(
            ContentDirCheck::new(Some(tmp.clone())).check().await.status,
            CheckStatus::Ok
        );
        let missing = tmp.join(format!("zenrouter-missing-{}", uuid::Uuid::now_v7()));
        assert_eq!(
            ContentDirCheck::new(Some(missing)).check().await.status,
            CheckStatus::Unavailable
        );
    }
}
