//! 外部连接的 Rust 实现：Python Adapter、身份源、检测器等。
//!
//! 骨架阶段只实现 Adapter 的健康与契约版本检查。

pub mod adapter;

pub use adapter::{AdapterCheck, AdapterClient};
