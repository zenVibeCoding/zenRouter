//! 持久化实现：PostgreSQL（权威状态）、Valkey（可重建状态，后续加入）与正文文件后端。
//!
//! 按领域分子模块实现应用层定义的持久化接口。骨架阶段只提供连接、迁移与就绪检查。

pub mod content;
pub mod postgres;

pub use content::ContentDirCheck;
pub use postgres::{PostgresCheck, connect_lazy, run_migrations};
