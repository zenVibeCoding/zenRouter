//! 管理面入口：身份接入、四模块管理接口、配置校验发布、受控查询与导出。
//!
//! 骨架阶段提供 `/api/v1/system/info`，并可选托管控制台静态资源（SPA 回退到 index.html）。

use std::path::PathBuf;

use axum::routing::get;
use axum::{Json, Router};
use tower_http::services::{ServeDir, ServeFile};
use zenrouter_contracts::{ADAPTER_CONTRACT_VERSION, SystemInfo};

#[derive(Clone)]
pub struct ControlState {
    pub version: &'static str,
    pub roles: Vec<String>,
}

/// 管理 API 路由，挂载在 `/api/v1` 下。
pub fn router(state: ControlState) -> Router {
    let api = Router::new().route(
        "/system/info",
        get(move || {
            let state = state.clone();
            async move {
                Json(SystemInfo {
                    name: "ZenRouter".into(),
                    version: state.version.into(),
                    roles: state.roles,
                    adapter_contract_version: ADAPTER_CONTRACT_VERSION.into(),
                })
            }
        }),
    );
    Router::new().nest("/api/v1", api)
}

/// 控制台静态资源。目录不存在时返回 `None`，开发期由 Vite 开发服务器提供页面。
pub fn console_service(dir: PathBuf) -> Option<Router> {
    let index = dir.join("index.html");
    if !index.is_file() {
        return None;
    }
    let serve = ServeDir::new(&dir).fallback(ServeFile::new(index));
    Some(Router::new().fallback_service(serve))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn system_info_reports_roles_and_contract() {
        let app = router(ControlState {
            version: "0.1.0",
            roles: vec!["control".into()],
        });
        let resp = app
            .oneshot(
                Request::get("/api/v1/system/info")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let v: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(v["roles"][0], "control");
        assert_eq!(v["adapter_contract_version"], ADAPTER_CONTRACT_VERSION);
    }

    #[test]
    fn console_service_absent_without_index() {
        let dir = std::env::temp_dir().join("zenrouter-no-console");
        assert!(console_service(dir).is_none());
    }
}
