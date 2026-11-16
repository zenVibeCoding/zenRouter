//! 运行角色。同一发行产物按角色组合或独立运行：
//! `all` = gateway + control + worker（并按配置监督 Python Adapter）。
//! `migrate` 是一次性任务，不与常驻角色组合。

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    Gateway,
    Control,
    Worker,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gateway => "gateway",
            Self::Control => "control",
            Self::Worker => "worker",
        }
    }
}

/// 一组常驻角色，至少包含一个。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleSet {
    roles: BTreeSet<Role>,
    all: bool,
}

impl RoleSet {
    pub fn contains(&self, role: Role) -> bool {
        self.roles.contains(&role)
    }

    pub fn names(&self) -> Vec<String> {
        self.roles.iter().map(|r| r.as_str().to_string()).collect()
    }

    pub fn needs_database(&self) -> bool {
        !self.roles.is_empty()
    }

    /// 只有 `all` 组合或包含 gateway 时才可能监督 Adapter；是否真正监督由配置决定。
    pub fn supervises_adapter(&self, config: &Config) -> bool {
        config.adapter.supervise && self.contains(Role::Gateway)
    }

    pub fn has_http_role(&self) -> bool {
        self.contains(Role::Gateway) || self.contains(Role::Control)
    }
}

impl fmt::Display for RoleSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.all {
            return f.write_str("all");
        }
        f.write_str(&self.names().join(","))
    }
}

impl FromStr for RoleSet {
    type Err = String;

    /// 接受 `all` 或逗号分隔的 `gateway,control,worker`。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut roles = BTreeSet::new();
        let mut all = false;
        for part in s.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            match part {
                "all" => {
                    all = true;
                    roles.extend([Role::Gateway, Role::Control, Role::Worker]);
                }
                "gateway" => {
                    roles.insert(Role::Gateway);
                }
                "control" => {
                    roles.insert(Role::Control);
                }
                "worker" => {
                    roles.insert(Role::Worker);
                }
                "migrate" => {
                    return Err("migrate 是一次性任务，请使用 `zenrouter migrate`".into());
                }
                other => {
                    return Err(format!(
                        "未知角色：{other}（可选 all、gateway、control、worker）"
                    ));
                }
            }
        }
        if roles.is_empty() {
            return Err("至少需要一个角色".into());
        }
        Ok(Self { roles, all })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roles() {
        let all: RoleSet = "all".parse().unwrap();
        assert!(all.contains(Role::Gateway) && all.contains(Role::Worker));
        assert_eq!(all.to_string(), "all");

        let gw: RoleSet = "gateway, worker".parse().unwrap();
        assert!(gw.contains(Role::Gateway) && !gw.contains(Role::Control));
        assert_eq!(gw.to_string(), "gateway,worker");

        assert!("".parse::<RoleSet>().is_err());
        assert!("adapter".parse::<RoleSet>().is_err());
        assert!("migrate".parse::<RoleSet>().is_err());
    }
}
