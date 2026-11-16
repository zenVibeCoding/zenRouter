//! Gateway → Python Adapter 的内部连接。
//!
//! Adapter 只执行 Gateway 已选定的一次上游尝试；HTTP 客户端不得自动重放请求，
//! 内部重试只由 Gateway 决定（docs/technical-architecture.md 第 5 节）。

use std::time::Duration;

use serde::Deserialize;
use zenrouter_application::health::DependencyReport;
use zenrouter_application::{BoxFuture, DependencyCheck};

const CONTRACT_HEADER: &str = "x-zenrouter-adapter-contract";

#[derive(Clone)]
pub struct AdapterClient {
    http: reqwest::Client,
    base_url: String,
    expected_contract: &'static str,
}

#[derive(Debug, Deserialize)]
struct AdapterHealth {
    status: String,
    contract_version: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AdapterHealthError {
    Unreachable,
    BadStatus(u16),
    InvalidBody,
    NotOk(String),
    ContractMismatch { expected: String, actual: String },
}

impl AdapterClient {
    pub fn new(
        base_url: impl Into<String>,
        expected_contract: &'static str,
    ) -> reqwest::Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(3))
            // Adapter 只在本机 loopback 或受控内网，不经过系统代理。
            .no_proxy()
            .build()?;
        Ok(Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            expected_contract,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn health(&self) -> Result<(), AdapterHealthError> {
        let resp = self
            .http
            .get(format!("{}/healthz", self.base_url))
            .header(CONTRACT_HEADER, self.expected_contract)
            .send()
            .await
            .map_err(|_| AdapterHealthError::Unreachable)?;
        if !resp.status().is_success() {
            return Err(AdapterHealthError::BadStatus(resp.status().as_u16()));
        }
        let body: AdapterHealth = resp
            .json()
            .await
            .map_err(|_| AdapterHealthError::InvalidBody)?;
        if body.status != "ok" {
            return Err(AdapterHealthError::NotOk(body.status));
        }
        if body.contract_version != self.expected_contract {
            return Err(AdapterHealthError::ContractMismatch {
                expected: self.expected_contract.to_string(),
                actual: body.contract_version,
            });
        }
        Ok(())
    }
}

pub struct AdapterCheck {
    client: AdapterClient,
}

impl AdapterCheck {
    pub fn new(client: AdapterClient) -> Self {
        Self { client }
    }
}

impl DependencyCheck for AdapterCheck {
    fn name(&self) -> &'static str {
        "adapter"
    }

    fn check(&self) -> BoxFuture<'_, DependencyReport> {
        Box::pin(async move {
            match self.client.health().await {
                Ok(()) => DependencyReport::ok(self.name()),
                Err(AdapterHealthError::ContractMismatch { expected, actual }) => {
                    DependencyReport::unavailable(
                        self.name(),
                        &format!("契约版本不一致：期望 {expected}，实际 {actual}"),
                    )
                }
                Err(AdapterHealthError::Unreachable) => {
                    DependencyReport::unavailable(self.name(), "无法连接适配器")
                }
                Err(err) => {
                    tracing::warn!(?err, "适配器健康检查失败");
                    DependencyReport::unavailable(self.name(), "适配器状态异常")
                }
            }
        })
    }
}
