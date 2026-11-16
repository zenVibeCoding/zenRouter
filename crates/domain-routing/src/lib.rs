//! 模型与路由领域：业务别名、端点能力、价格版本、路由筛选、重试与缓存策略。
//!
//! 骨架阶段只定义协议家族、业务别名与重试判断的最小规则。

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 客户端入口与上游端点的协议家族。入口与上游分别建模。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolFamily {
    OpenAiChat,
    OpenAiResponses,
    AnthropicMessages,
    GeminiGenerateContent,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ModelAliasError {
    #[error("业务模型别名不能为空")]
    Empty,
    #[error("业务模型别名过长（最多 {max} 个字符）")]
    TooLong { max: usize },
    #[error("业务模型别名只能包含字母、数字以及 - _ . /")]
    InvalidCharacter,
}

/// 面向下游的业务模型别名；与上游真实模型版本分离。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ModelAlias(String);

impl ModelAlias {
    pub const MAX_LEN: usize = 128;

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ModelAlias {
    type Error = ModelAliasError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ModelAliasError::Empty);
        }
        if value.chars().count() > Self::MAX_LEN {
            return Err(ModelAliasError::TooLong { max: Self::MAX_LEN });
        }
        if !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
        {
            return Err(ModelAliasError::InvalidCharacter);
        }
        Ok(Self(value))
    }
}

impl From<ModelAlias> for String {
    fn from(value: ModelAlias) -> Self {
        value.0
    }
}

/// 单次上游尝试结束后的事实，用于判断能否创建新的尝试。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttemptOutcome {
    /// 错误是否属于策略允许重试的类别（超时、限流等）。
    pub retryable_error: bool,
    /// 是否已经向客户端释放过内容。已释放内容不可撤回，也不能拼接另一份生成。
    pub content_released: bool,
    /// 是否为安全拒绝。安全拒绝不能通过换供应商绕过。
    pub security_rejected: bool,
}

/// 重试约束：次数上限之外，总时限与预算由调用编排另外检查。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
}

impl RetryPolicy {
    /// 仅判断领域层面的必要条件；时限与预算检查在应用层用例中完成。
    pub fn may_retry(&self, attempts_so_far: u32, outcome: AttemptOutcome) -> bool {
        outcome.retryable_error
            && !outcome.content_released
            && !outcome.security_rejected
            && attempts_so_far < self.max_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_validation() {
        assert!(ModelAlias::try_from("chat-default".to_string()).is_ok());
        assert!(ModelAlias::try_from("team/gpt-4.1_mini".to_string()).is_ok());
        assert_eq!(
            ModelAlias::try_from(String::new()),
            Err(ModelAliasError::Empty)
        );
        assert_eq!(
            ModelAlias::try_from("has space".to_string()),
            Err(ModelAliasError::InvalidCharacter)
        );
    }

    #[test]
    fn retry_is_refused_after_release_or_security_rejection() {
        let policy = RetryPolicy { max_attempts: 3 };
        let ok = AttemptOutcome {
            retryable_error: true,
            content_released: false,
            security_rejected: false,
        };
        assert!(policy.may_retry(1, ok));
        assert!(!policy.may_retry(3, ok));
        assert!(!policy.may_retry(
            1,
            AttemptOutcome {
                content_released: true,
                ..ok
            }
        ));
        assert!(!policy.may_retry(
            1,
            AttemptOutcome {
                security_rejected: true,
                ..ok
            }
        ));
    }
}
