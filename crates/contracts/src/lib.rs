//! 版本化传输类型。
//!
//! 这里的类型是对外／跨进程的数据形状，与领域模型分别维护，转换发生在接入边界。
//! 与 `api/` 中的 OpenAPI 规范通过生成或契约测试保持一致。

use serde::{Deserialize, Serialize};

/// Gateway ↔ Adapter 内部契约版本。双方不一致时 Gateway 不应把调用交给该适配器。
pub const ADAPTER_CONTRACT_VERSION: &str = "0.1";

/// 管理 API 的路径前缀版本。
pub const CONTROL_API_VERSION: &str = "v1";

/// 依赖检查结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Ok,
    Degraded,
    Unavailable,
    /// 当前配置下不需要该依赖（例如不留正文时的正文目录）。
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyReport {
    pub name: String,
    pub status: CheckStatus,
    /// 面向运维的说明；不得包含连接串、密钥或正文。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl DependencyReport {
    pub fn ok(name: &str) -> Self {
        Self::with(name, CheckStatus::Ok, None)
    }

    pub fn unavailable(name: &str, detail: &str) -> Self {
        Self::with(name, CheckStatus::Unavailable, Some(detail))
    }

    pub fn skipped(name: &str, detail: &str) -> Self {
        Self::with(name, CheckStatus::Skipped, Some(detail))
    }

    fn with(name: &str, status: CheckStatus, detail: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            status,
            detail: detail.map(str::to_string),
        }
    }
}

/// `/readyz` 等就绪探针的响应体。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub ready: bool,
    pub draining: bool,
    pub dependencies: Vec<DependencyReport>,
}

/// 运行实例信息，供控制台展示。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemInfo {
    pub name: String,
    pub version: String,
    pub roles: Vec<String>,
    pub adapter_contract_version: String,
}

/// 网关在协议尚未实现时返回的统一错误体（OpenAI 风格的外层结构）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayErrorBody {
    pub error: GatewayError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayError {
    pub message: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub code: String,
    pub request_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_serializes_snake_case() {
        let r = ReadinessReport {
            ready: false,
            draining: false,
            dependencies: vec![DependencyReport {
                name: "postgres".into(),
                status: CheckStatus::Unavailable,
                detail: None,
            }],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["dependencies"][0]["status"], "unavailable");
        assert!(v["dependencies"][0].get("detail").is_none());
    }
}
