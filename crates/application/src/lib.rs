//! 应用层：按业务组织用例，编排跨域操作，定义事务边界和所需的存储／外部服务接口。
//!
//! 依赖方向：入口层（gateway / control-api / worker）→ application → domain-*。
//! storage 与 integrations 实现本层定义的接口，由 runtime 装配。
//!
//! 骨架阶段只提供运行探针用例；四个领域的用例模块按详细设计逐步加入：
//! `identity`、`routing`、`security`、`billing`。

pub mod health;

pub use health::{BoxFuture, DependencyCheck, Readiness};

/// 领域模型的便捷入口，入口层通过应用层使用领域类型。
pub mod domain {
    pub use zenrouter_domain_billing as billing;
    pub use zenrouter_domain_identity as identity;
    pub use zenrouter_domain_routing as routing;
    pub use zenrouter_domain_security as security;
}
