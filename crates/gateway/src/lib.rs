//! 调用面入口：多协议入口、可信身份、治理检查、预算、路由、尝试生命周期与流式输出。
//!
//! 骨架阶段只注册四类协议入口的路由，统一返回 501，并为每个请求分配可信 request_id。
//! 薄调用链（验证门槛 A）在此基础上实现。

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use zenrouter_contracts::{GatewayError, GatewayErrorBody};
use zenrouter_domain_security::RequestId;

/// 响应中回传网关分配的请求标识。
pub const REQUEST_ID_HEADER: &str = "x-zenrouter-request-id";

/// 调用面路由。入口与上游协议分别建模，这里只列客户端入口。
pub fn router() -> Router {
    Router::new()
        // OpenAI 兼容入口（必备）
        .route(
            "/v1/chat/completions",
            post(|| not_implemented("openai_chat")),
        )
        .route(
            "/v1/responses",
            post(|| not_implemented("openai_responses")),
        )
        // Anthropic 兼容入口
        .route(
            "/v1/messages",
            post(|| not_implemented("anthropic_messages")),
        )
        // Gemini 原生入口，例如 /v1beta/models/gemini-x:generateContent
        .route("/v1beta/models/{*target}", post(gemini))
}

async fn gemini(Path(target): Path<String>) -> Response {
    if target.ends_with(":generateContent") || target.ends_with(":streamGenerateContent") {
        not_implemented("gemini_generate_content").await
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn not_implemented(protocol: &'static str) -> Response {
    let request_id = RequestId::new().to_string();
    tracing::info!(%request_id, protocol, "协议入口尚未实现");
    let body = GatewayErrorBody {
        error: GatewayError {
            message: format!("{protocol} 入口尚未实现（项目骨架阶段）"),
            kind: "not_implemented".into(),
            code: "not_implemented".into(),
            request_id: request_id.clone(),
        },
    };
    (
        StatusCode::NOT_IMPLEMENTED,
        [(REQUEST_ID_HEADER, request_id)],
        Json(body),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use tower::ServiceExt;

    async fn post_status(uri: &str) -> (StatusCode, Option<String>, serde_json::Value) {
        let resp = router()
            .oneshot(
                Request::post(uri)
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let rid = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .map(|v| v.to_str().unwrap().to_string());
        let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, rid, json)
    }

    #[tokio::test]
    async fn protocol_entries_return_501_with_request_id() {
        for uri in [
            "/v1/chat/completions",
            "/v1/responses",
            "/v1/messages",
            "/v1beta/models/gemini-pro:generateContent",
        ] {
            let (status, rid, json) = post_status(uri).await;
            assert_eq!(status, StatusCode::NOT_IMPLEMENTED, "{uri}");
            let rid = rid.expect("request id header");
            assert_eq!(json["error"]["request_id"], rid);
        }
    }

    #[tokio::test]
    async fn unknown_gemini_method_is_404() {
        let (status, _, _) = post_status("/v1beta/models/gemini-pro:unknown").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
