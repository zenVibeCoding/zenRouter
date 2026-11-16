//! 统一配置模型。四种部署形态使用相同的配置结构与覆盖顺序：
//! 内置默认值 → 配置文件（`--config` / `ZENROUTER_CONFIG`）→ 环境变量 `ZENROUTER__段__键`。

use std::fmt;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

use crate::role::RoleSet;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub adapter: AdapterConfig,
    #[serde(default)]
    pub content: ContentConfig,
    #[serde(default)]
    pub log: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ServerConfig {
    /// 调用面（OpenAI / Anthropic / Gemini 兼容入口）监听地址。
    pub gateway_listen: SocketAddr,
    /// 管理面与控制台监听地址。
    pub control_listen: SocketAddr,
    /// 仅运行 worker 角色时的探针监听地址。
    pub probe_listen: SocketAddr,
    /// 控制台静态资源目录；不存在时不托管页面。
    pub console_dir: PathBuf,
    /// 收到退出信号后等待存量请求结束的最长秒数。
    pub shutdown_grace_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            gateway_listen: ([0, 0, 0, 0], 8080).into(),
            control_listen: ([0, 0, 0, 0], 8081).into(),
            probe_listen: ([0, 0, 0, 0], 8082).into(),
            console_dir: PathBuf::from("apps/console/dist"),
            shutdown_grace_secs: 30,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DatabaseConfig {
    /// PostgreSQL 连接串。含密码，不会出现在日志与 Debug 输出中。
    pub url: Option<String>,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: 10,
        }
    }
}

impl fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("url", &self.url.as_ref().map(|_| "<redacted>"))
            .field("max_connections", &self.max_connections)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AdapterConfig {
    /// Python Adapter 地址；只应监听 loopback 或受控内网。
    pub url: String,
    /// 是否由本进程启动并监督 Adapter 子进程（单应用容器与本地 `all` 角色使用）。
    pub supervise: bool,
    /// 监督模式下的启动命令（argv）。
    pub command: Vec<String>,
    /// 子进程工作目录；为空时继承当前目录。
    pub workdir: Option<PathBuf>,
    /// 子进程收到 SIGTERM 后的等待秒数，超时强制结束。
    pub stop_timeout_secs: u64,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8090".into(),
            supervise: false,
            command: Vec::new(),
            workdir: None,
            stop_timeout_secs: 10,
        }
    }
}

/// 正文留存的部署侧开关。具体留存范围由企业政策决定，这里只决定是否启用文件后端。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContentBackend {
    /// 不启用正文存储链路，不要求正文卷。
    #[default]
    None,
    /// 企业指定目录（JSONL 写入，可选 Parquet 封存）。
    File,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContentConfig {
    pub backend: ContentBackend,
    pub dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    #[default]
    Text,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LogConfig {
    pub format: LogFormat,
    /// `tracing` 过滤表达式；环境变量 `RUST_LOG` 优先。
    pub filter: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::Text,
            filter: "info".into(),
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        Self::figment(path)
            .extract()
            .context("配置解析失败（检查配置文件与 ZENROUTER__* 环境变量）")
    }

    fn figment(path: Option<&Path>) -> Figment {
        let mut fig = Figment::from(Serialized::defaults(Config::default()));
        if let Some(path) = path {
            fig = fig.merge(Toml::file_exact(path));
        }
        fig.merge(Env::prefixed("ZENROUTER__").split("__"))
    }

    /// 按将要运行的角色校验必需配置。错误信息不包含任何秘密。
    pub fn validate_for(&self, roles: &RoleSet) -> anyhow::Result<()> {
        if roles.needs_database() && self.database.url.as_deref().unwrap_or("").is_empty() {
            bail!("角色 {roles} 需要 database.url（或环境变量 ZENROUTER__DATABASE__URL）");
        }
        if self.database.max_connections == 0 {
            bail!("database.max_connections 必须大于 0");
        }
        if self.content.backend == ContentBackend::File && self.content.dir.is_none() {
            bail!("content.backend = \"file\" 时必须配置 content.dir");
        }
        if roles.supervises_adapter(self) && self.adapter.command.is_empty() {
            bail!("adapter.supervise = true 时必须配置 adapter.command");
        }
        Ok(())
    }

    /// 生效的正文目录；不留正文时为 `None`。
    pub fn content_dir(&self) -> Option<PathBuf> {
        match self.content.backend {
            ContentBackend::None => None,
            ContentBackend::File => self.content.dir.clone(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::result_large_err)] // figment::Jail 的闭包签名由库决定
mod tests {
    use super::*;
    use figment::Jail;

    #[test]
    fn env_overrides_file_and_defaults() {
        Jail::expect_with(|jail| {
            jail.create_file(
                "z.toml",
                r#"
                [server]
                gateway_listen = "127.0.0.1:9000"
                [database]
                url = "postgres://file"
                "#,
            )?;
            jail.set_env("ZENROUTER__DATABASE__URL", "postgres://env");
            jail.set_env("ZENROUTER__CONTENT__BACKEND", "file");
            jail.set_env("ZENROUTER__CONTENT__DIR", "/data/content");
            let cfg = Config::load(Some(Path::new("z.toml"))).unwrap();
            assert_eq!(cfg.server.gateway_listen.port(), 9000);
            assert_eq!(cfg.server.control_listen.port(), 8081);
            assert_eq!(cfg.database.url.as_deref(), Some("postgres://env"));
            assert_eq!(cfg.content_dir(), Some(PathBuf::from("/data/content")));
            Ok(())
        });
    }

    #[test]
    fn unknown_keys_are_rejected() {
        Jail::expect_with(|jail| {
            jail.create_file("z.toml", "[server]\ngateway_port = 1\n")?;
            assert!(Config::load(Some(Path::new("z.toml"))).is_err());
            Ok(())
        });
    }

    #[test]
    fn database_url_never_appears_in_debug() {
        let cfg = DatabaseConfig {
            url: Some("postgres://u:secret@h/db".into()),
            max_connections: 1,
        };
        assert!(!format!("{cfg:?}").contains("secret"));
    }

    #[test]
    fn validation_depends_on_roles() {
        let cfg = Config::default();
        assert!(cfg.validate_for(&"worker".parse().unwrap()).is_err());

        let mut cfg = Config::default();
        cfg.database.url = Some("postgres://x".into());
        assert!(cfg.validate_for(&"all".parse().unwrap()).is_ok());

        cfg.content.backend = ContentBackend::File;
        assert!(cfg.validate_for(&"gateway".parse().unwrap()).is_err());
    }
}
