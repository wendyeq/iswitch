//! [INPUT]:
//!   source: ./handler.rs ([POS]: Request Handler)
//!
//! [OUTPUT]:
//!   - create_router
//!
//! [POS]: Router definition for proxy server
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::proxy::handler::{
    handle_claude_messages, handle_codex_chat_completions, handle_codex_completions,
    handle_codex_legacy_completions, handle_options, ProxyState,
};
use axum::{routing::post, Router};
use tower_http::cors::{Any, CorsLayer};

/// 创建 Axum Router
pub fn create_router(state: ProxyState) -> Router {
    // Phase 2: 允许所有 CORS，后续 Phase 7 安全审计时收紧
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Claude Code Route
        .route(
            "/v1/messages",
            post(handle_claude_messages).options(handle_options),
        )
        // Codex Route
        .route(
            "/responses",
            post(handle_codex_completions).options(handle_options),
        )
        // Codex Chat Completions Route (Standard OpenAI)
        .route(
            "/v1/chat/completions",
            post(handle_codex_chat_completions).options(handle_options),
        )
        // Codex Legacy Completions Route (Standard OpenAI)
        .route(
            "/v1/completions",
            post(handle_codex_legacy_completions).options(handle_options),
        )
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::health::ProviderHealthTracker;
    use crate::services::provider_service::ProviderService;
    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};
    use chrono::Duration;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use tower::Service;

    fn build_state() -> ProxyState {
        let provider_service = Arc::new(ProviderService::new());
        let pool = Arc::new(SqlitePool::connect_lazy("sqlite::memory:").unwrap());
        let health_tracker = Arc::new(ProviderHealthTracker::new(3, Duration::seconds(60)));

        let http_client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("failed to build test client");

        ProxyState {
            provider_service,
            pool,
            http_client,
            health_tracker,
        }
    }

    #[tokio::test]
    async fn test_claude_route_allows_cors_options() {
        let mut router = create_router(build_state());
        let response = router
            .call(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/v1/messages")
                    .header(header::ORIGIN, "https://example.com")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let cors_header = response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|v: &axum::http::HeaderValue| v.to_str().ok());
        assert_eq!(cors_header, Some("*"));
    }

    #[tokio::test]
    async fn test_codex_route_allows_cors_options() {
        let mut router = create_router(build_state());
        let response = router
            .call(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/responses")
                    .header(header::ORIGIN, "https://example.com")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
