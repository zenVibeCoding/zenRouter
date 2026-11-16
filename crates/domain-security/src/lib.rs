//! 安全与审计领域：检测策略、处置、接收与尝试证据、原文授权、复核和生命周期。
//!
//! 骨架阶段定义调用标识与企业正文留存模式。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 网关在接收时分配的可信请求标识；不以客户端 trace 字段替代。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RequestId(pub Uuid);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// 单次上游尝试的标识；每次重试或切换都生成新的 attempt_id。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttemptId(pub Uuid);

impl AttemptId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for AttemptId {
    fn default() -> Self {
        Self::new()
    }
}

/// 企业正文留存模式。请求与响应可分别设置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionMode {
    /// 不持久化正文及其派生内容；必要元数据与账本另按最小字段策略处理。
    None,
    /// 限定范围或脱敏后留存。
    Partial,
    /// 完整留存获准 payload。
    Full,
}

impl RetentionMode {
    /// 是否需要进入正文持久化分支。`None` 时必须在写入前排除，不能先写后删。
    pub fn persists_content(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// 一次调用实际生效的留存政策（按版本记录在请求索引中）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionDecision {
    pub request: RetentionMode,
    pub response: RetentionMode,
    pub policy_version: u64,
}

/// 内容检测结论。未检测、检测失败与通过必须分别表示。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionVerdict {
    Passed,
    Flagged,
    Blocked,
    NotDetected,
    DetectorFailed,
}

impl DetectionVerdict {
    /// 只有明确通过才可展示为“安全”。
    pub fn is_affirmative_pass(self) -> bool {
        matches!(self, Self::Passed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_mode_never_persists() {
        assert!(!RetentionMode::None.persists_content());
        assert!(RetentionMode::Partial.persists_content());
        assert!(RetentionMode::Full.persists_content());
    }

    #[test]
    fn failures_are_not_passes() {
        assert!(!DetectionVerdict::NotDetected.is_affirmative_pass());
        assert!(!DetectionVerdict::DetectorFailed.is_affirmative_pass());
        assert!(DetectionVerdict::Passed.is_affirmative_pass());
    }
}
