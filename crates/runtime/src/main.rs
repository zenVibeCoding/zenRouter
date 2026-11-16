//! ZenRouter 可执行入口。
//!
//! ```text
//! zenrouter serve --roles all          # 组合运行 gateway + control + worker（可监督 Adapter）
//! zenrouter serve --roles gateway      # 按角色独立运行
//! zenrouter migrate                    # 一次性数据库迁移任务
//! zenrouter check-config               # 校验配置并退出
//! ```

mod config;
mod role;
mod server;
mod supervisor;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use crate::config::{Config, LogFormat};
use crate::role::RoleSet;

#[derive(Parser)]
#[command(
    name = "zenrouter",
    version,
    about = "ZenRouter 企业级 AI 路由与治理平台"
)]
struct Cli {
    /// 配置文件路径（TOML）。环境变量 ZENROUTER__* 覆盖文件中的值。
    #[arg(long, global = true, env = "ZENROUTER_CONFIG")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 启动常驻角色。
    Serve {
        /// all，或逗号分隔的 gateway、control、worker。
        #[arg(long, env = "ZENROUTER_ROLES", default_value = "all")]
        roles: RoleSet,
    },
    /// 执行数据库迁移后退出。
    Migrate,
    /// 校验配置后退出，不连接外部依赖。
    CheckConfig {
        #[arg(long, env = "ZENROUTER_ROLES", default_value = "all")]
        roles: RoleSet,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;
    init_tracing(&config);

    match cli.command {
        Command::Serve { roles } => {
            config.validate_for(&roles)?;
            server::serve(config, roles).await
        }
        Command::Migrate => {
            let url = config
                .database
                .url
                .as_deref()
                .filter(|u| !u.is_empty())
                .ok_or_else(|| anyhow::anyhow!("migrate 需要 database.url"))?;
            let pool = zenrouter_storage::connect_lazy(url, 2)?;
            zenrouter_storage::run_migrations(&pool).await?;
            tracing::info!("数据库迁移完成");
            Ok(())
        }
        Command::CheckConfig { roles } => {
            config.validate_for(&roles)?;
            println!("配置有效（角色：{roles}）");
            Ok(())
        }
    }
}

fn init_tracing(config: &Config) {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.log.filter))
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    match config.log.format {
        LogFormat::Json => builder.json().init(),
        LogFormat::Text => builder.init(),
    }
}
