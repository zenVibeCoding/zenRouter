//! 组织与权限领域：外部身份绑定、组织成员、应用身份、角色范围、撤权与委派。
//!
//! 骨架阶段只放置跨领域共享的标识类型与最基本的不变量；
//! 具体实体、仓储接口和授权规则在详细设计后补齐。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// 生成按时间有序的新标识（UUIDv7）。
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_type!(
    /// 企业组织。所有资源归属都以可信的 organization_id 为根。
    OrganizationId
);
id_type!(
    /// 组织内的部门（层级额度与归因的节点）。
    DepartmentId
);
id_type!(
    /// 组织成员。
    MemberId
);
id_type!(
    /// 接入网关的应用。
    ApplicationId
);
id_type!(
    /// 网关凭证（虚拟密钥）。数据库只保存可验证摘要。
    CredentialId
);

/// 网关凭证状态。只有 `Active` 允许发起调用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStatus {
    Active,
    Disabled,
    Expired,
    Revoked,
}

impl CredentialStatus {
    pub fn allows_invocation(self) -> bool {
        matches!(self, Self::Active)
    }
}

/// 经认证建立的调用主体。部门与用户归属来自认证结果，不信任客户端自报标头。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Principal {
    pub organization_id: OrganizationId,
    pub application_id: ApplicationId,
    pub credential_id: CredentialId,
    pub department_id: Option<DepartmentId>,
    pub member_id: Option<MemberId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_active_credentials_may_invoke() {
        assert!(CredentialStatus::Active.allows_invocation());
        for s in [
            CredentialStatus::Disabled,
            CredentialStatus::Expired,
            CredentialStatus::Revoked,
        ] {
            assert!(!s.allows_invocation());
        }
    }

    #[test]
    fn ids_are_unique_and_time_ordered() {
        let a = OrganizationId::new();
        let b = OrganizationId::new();
        assert_ne!(a, b);
        assert!(a.0 < b.0);
    }
}
