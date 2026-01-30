//! [INPUT]:
//!   source: ../../../../code-switch/services/providerrelay.go ([POS]: Go Request Handler - Legacy)
//!   source: ../services/provider_service.rs ([POS]: Provider Service)
//!   source: ../models/provider.rs ([POS]: Provider Model)
//!
//! [OUTPUT]:
//!   - handle_claude_messages
//!   - handle_codex_completions
//!   - handle_codex_chat_completions
//!   - handle_codex_legacy_completions
//!   - handle_options
//!
//! [POS]: Logic for handling and forwarding requests
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use axum::{
    body::Body,
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::models::{Provider, ProviderKind};
use crate::proxy::health::{FailureReason, ProviderHealthTracker};
use crate::services::provider_service::ProviderService;

use sqlx::{Pool, Sqlite};

// Define ProxyState (will be shared from server.rs)
#[derive(Clone)]
pub struct ProxyState {
    pub provider_service: Arc<ProviderService>,
    pub pool: Arc<Pool<Sqlite>>,
    pub http_client: Client,
    pub health_tracker: Arc<ProviderHealthTracker>,
}

/// 处理 OPTIONS 预检请求
pub async fn handle_options() -> impl IntoResponse {
    StatusCode::OK
}

/// 处理 Claude Messages API 请求
pub async fn handle_claude_messages(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    debug!("收到 Claude 请求");
    forward_request(state, headers, body, ProviderKind::Claude, "/v1/messages").await
}

/// 处理 Codex Completions API 请求
pub async fn handle_codex_completions(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    debug!("收到 Codex 请求");
    // Codex 路径可能是 variable, 但通常是 /responses 或者 /v1/chat/completions (OpenAI compatible)
    // 根据 tasks.md 2.4.3 是 /responses
    forward_request(state, headers, body, ProviderKind::Codex, "/responses").await
}

/// 处理 Codex Chat Completions API 请求 (/v1/chat/completions)
pub async fn handle_codex_chat_completions(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    debug!("收到 Codex Chat Completions 请求");
    forward_request(
        state,
        headers,
        body,
        ProviderKind::Codex,
        "/v1/chat/completions",
    )
    .await
}

/// 处理 Codex Legacy Completions API 请求 (/v1/completions)
pub async fn handle_codex_legacy_completions(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    debug!("收到 Codex Legacy Completions 请求");
    forward_request(state, headers, body, ProviderKind::Codex, "/v1/completions").await
}

use crate::db::request_log;
use crate::models::RequestLog;
use crate::proxy::monitor::{self, MonitorContext};
use chrono::Utc;

/// 核心转发逻辑
async fn forward_request(
    state: ProxyState,
    headers: HeaderMap,
    mut body: Value,
    kind: ProviderKind,
    path: &str,
) -> Response {
    let _start_time = Utc::now();

    // 1. 获取并验证请求的模型名
    let model = match body.get("model").and_then(|v| v.as_str()) {
        Some(m) if !m.is_empty() && m.trim().len() > 0 => m.to_string(),
        Some(_) => {
            return (StatusCode::BAD_REQUEST, "Model field cannot be empty").into_response();
        }
        None => {
            return (StatusCode::BAD_REQUEST, "Missing model field").into_response();
        }
    };

    // 验证请求体字段：/v1/completions 使用 prompt，其他路径使用 messages
    if path == "/v1/completions" {
        if body.get("prompt").is_none() {
            return (StatusCode::BAD_REQUEST, "Missing prompt field").into_response();
        }
    } else if body.get("messages").is_none() {
        return (StatusCode::BAD_REQUEST, "Missing messages field").into_response();
    }

    let is_stream = body
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // 2. 加载可用 Provider
    let providers = match state
        .provider_service
        .load_providers(&kind.to_string())
        .await
    {
        Ok(p) => p,
        Err(e) => {
            error!("加载 Provider 失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load providers",
            )
                .into_response();
        }
    };

    // 3. 筛选并排序 Provider
    let mut eligible_providers: Vec<&Provider> = providers
        .iter()
        .filter(|p| p.is_valid() && p.is_model_supported(&model))
        .collect();

    if eligible_providers.is_empty() {
        warn!("未找到支持模型 '{}' 的可用 Provider", model);
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("No available provider for model {}", model),
        )
            .into_response();
    }

    // 按 level 排序 (level 小的优先, level > 0)
    eligible_providers.sort_by_key(|p| if p.level == 0 { i32::MAX } else { p.level });

    // 4. 健康检查：过滤降级的供应商
    let provider_keys: Vec<(ProviderKind, i64)> =
        eligible_providers.iter().map(|p| (kind, p.id)).collect();

    // 检查是否所有供应商都降级（保底策略）
    let all_degraded = state.health_tracker.all_degraded(&provider_keys);
    if all_degraded {
        warn!("[ProviderHealth] 所有 Provider 均已降级，启用保底策略");
    }

    // 筛选出可用的供应商（未降级或已过恢复超时）
    let healthy_providers: Vec<&Provider> = if all_degraded {
        // 保底策略：忽略降级状态，按优先级尝试所有供应商
        eligible_providers.clone()
    } else {
        eligible_providers
            .iter()
            .filter(|p| state.health_tracker.is_available(kind, p.id, &p.name))
            .copied()
            .collect()
    };

    // 记录被跳过的降级供应商
    for p in &eligible_providers {
        if !state.health_tracker.is_available(kind, p.id, &p.name) && !all_degraded {
            let failure_count = state.health_tracker.get_failure_count(kind, p.id);
            debug!("跳过降级供应商: {} (连续失败 {} 次)", p.name, failure_count);
        }
    }

    // 使用健康的供应商列表进行转发
    let providers_to_try = if healthy_providers.is_empty() {
        // 如果过滤后没有可用供应商，使用原始列表（不应该发生，但作为防御）
        eligible_providers
    } else {
        healthy_providers
    };

    // 5. 尝试转发
    for provider in providers_to_try {
        // 计算目标模型名 (处理映射)
        let target_model = provider.get_effective_model(&model);

        // 替换 Body 中的模型名
        if let Some(obj) = body.as_object_mut() {
            obj.insert("model".to_string(), Value::String(target_model.clone()));

            // 对于 Codex 流式请求，根据 Provider 类型注入 stream_options
            // 只有 OpenAI 官方 API 支持此参数，Azure 和其他第三方不支持
            if kind == ProviderKind::Codex && is_stream {
                let supports_stream_options = provider.api_url.contains("api.openai.com");
                if supports_stream_options {
                    obj.insert(
                        "stream_options".to_string(),
                        serde_json::json!({ "include_usage": true }),
                    );
                } else {
                    // 确保移除可能存在的 stream_options（防止之前循环遗留）
                    obj.remove("stream_options");
                }
            }
        }

        // 构建目标 URL
        let url = format!("{}{}", provider.api_url.trim_end_matches('/'), path);

        debug!(
            "尝试转发至 Provider: {} ({}) -> {}",
            provider.name, model, target_model
        );

        // 构建请求
        let mut req_builder = state.http_client.post(&url).json(&body);

        // 复制 Header (过滤掉敏感头和 Hop-by-hop 头)
        for (key, value) in headers.iter() {
            let key_str = key.as_str().to_lowercase();
            if key_str == "host"
                || key_str == "content-length"
                || key_str == "content-type"
                || key_str == "x-api-key"
                || key_str == "authorization"
                || key_str == "connection"
                || key_str == "upgrade"
                || key_str == "accept-encoding"
            {
                continue;
            }
            req_builder = req_builder.header(key, value);
        }

        // 添加 API Key
        match kind {
            ProviderKind::Claude => {
                let is_official = provider.api_url.contains("anthropic.com");
                if is_official {
                    req_builder = req_builder.header("x-api-key", &provider.api_key);
                } else {
                    // 非官方域名（如 MiniMax），通常使用 Authorization: Bearer
                    // 为了最大兼容性且不触碰 ALB 限制，只发送这一个
                    req_builder =
                        req_builder.header("Authorization", format!("Bearer {}", provider.api_key));
                }
                // 强制要求不压缩数据，以便 monitor.rs 能够解析 Body 中的 Token 信息
                req_builder = req_builder.header("Accept-Encoding", "identity");
            }
            ProviderKind::Codex => {
                req_builder =
                    req_builder.header("Authorization", format!("Bearer {}", provider.api_key));
            }
        }

        // 发送请求
        let request_start = Utc::now();
        match req_builder.send().await {
            Ok(resp) => {
                let status = resp.status();

                if status.is_success() {
                    info!("转发成功: {} -> {}", provider.name, status);

                    // 记录成功，重置健康状态
                    state
                        .health_tracker
                        .record_success(kind, provider.id, &provider.name);

                    // 构造响应
                    let mut response_builder = Response::builder().status(status);

                    // 复制响应 Header
                    if let Some(headers_mut) = response_builder.headers_mut() {
                        for (key, value) in resp.headers() {
                            headers_mut.insert(key, value.clone());
                        }
                    }

                    // 流式响应体
                    let stream = Body::from_stream(resp.bytes_stream());
                    match response_builder.body(stream) {
                        Ok(res) => {
                            // Monitor and Log Success response
                            let ctx = MonitorContext {
                                platform: kind.to_string(),
                                provider: provider.name.clone(),
                                model: target_model.clone(),
                                start_time: request_start, // Log duration relative to actual request
                                is_stream,
                                pool: state.pool.clone(),
                            };
                            return monitor::monitor_response(res, ctx);
                        }
                        Err(e) => {
                            error!("构建响应失败: {}", e);
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Failed to build response",
                            )
                                .into_response();
                        }
                    }
                } else {
                    // Log failure
                    let pool = state.pool.clone();
                    let log_model = target_model.clone();
                    let log_provider = provider.name.clone();
                    let log_platform = kind.to_string();
                    let duration = (Utc::now() - request_start).num_milliseconds() as f64 / 1000.0;

                    tokio::spawn(async move {
                        let mut log =
                            RequestLog::new(&log_platform, &log_provider, &log_model, is_stream);
                        log.http_code = status.as_u16() as i32;
                        log.duration_sec = duration;
                        log.created_at = request_start.format("%Y-%m-%dT%H:%M:%SZ").to_string();

                        if let Err(e) = request_log::insert_log(&pool, &log).await {
                            error!("Failed to insert request log: {}", e);
                        } else {
                            debug!("Inserted failure log for provider {}", log_provider);
                        }
                    });

                    // 策略调整: 所有非成功状态码 (4xx/5xx) 均触发故障转移
                    // 这确保了 404 (模型不存在), 401 (Key失效) 等情况也能自动尝试其他 Provider

                    // 1. 记录健康状态
                    let reason = if status.as_u16() == 429 {
                        FailureReason::RateLimited
                    } else {
                        // 将所有非 429 的错误归类为 ServerError (包含 ClientError如 404)，用于健康统计
                        FailureReason::ServerError(status.as_u16())
                    };
                    state
                        .health_tracker
                        .record_failure(kind, provider.id, &provider.name, reason);

                    // 2. 读取错误详情用于服务端日志 (Warning)
                    // 注意: 我们不把上游的详细错误返回给客户端，只记录在服务端日志用于排查
                    let _ = match resp.text().await {
                        Ok(text) => {
                            warn!(
                                "Provider {} 请求失败: status={}, 响应: {}",
                                provider.name, status, text
                            );
                        }
                        Err(_) => {
                            warn!(
                                "Provider {} 请求失败: status={}, 无法读取响应体",
                                provider.name, status
                            );
                        }
                    };

                    // 3. 继续尝试下一个 Provider
                    continue;
                }
            }
            Err(e) => {
                error!("Provider {} 连接失败: {}, 尝试下一个...", provider.name, e);

                // 记录连接失败
                state.health_tracker.record_failure(
                    kind,
                    provider.id,
                    &provider.name,
                    FailureReason::ConnectionError,
                );

                // Log connection failure
                let pool = state.pool.clone();
                let log_model = target_model.clone();
                let log_provider = provider.name.clone();
                let log_platform = kind.to_string();

                tokio::spawn(async move {
                    let mut log =
                        RequestLog::new(&log_platform, &log_provider, &log_model, is_stream);
                    log.http_code = 502; // Bad Gateway/Connection Error
                    log.created_at = request_start.format("%Y-%m-%dT%H:%M:%SZ").to_string();

                    if let Err(e) = request_log::insert_log(&pool, &log).await {
                        error!("Failed to insert connection failure log: {}", e);
                    }
                });

                continue;
            }
        }
    }

    warn!("所有 Provider 均尝试失败");
    (StatusCode::BAD_GATEWAY, "All providers failed").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // 辅助函数：创建测试环境
    async fn setup_env() -> (MockServer, ProxyState, tempfile::TempDir) {
        let mock_server = MockServer::start().await;
        // 保持 temp_dir 不被 drop，直到测试结束
        let temp_dir = tempdir().unwrap();

        let provider_service = Arc::new(ProviderService::with_root(temp_dir.path().to_path_buf()));

        // Mock Pool
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let state = ProxyState {
            provider_service,
            pool: Arc::new(pool),
            http_client: Client::new(),
            health_tracker: Arc::new(ProviderHealthTracker::with_defaults()),
        };

        (mock_server, state, temp_dir)
    }

    // 测试成功转发
    #[tokio::test]
    async fn test_forward_request_success() {
        let (mock_server, state, _temp_dir) = setup_env().await;

        // 1. 设置 Mock Server 响应
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "content": "hello" })))
            .mount(&mock_server)
            .await;

        // 2. 配置 Provider
        let providers = vec![Provider {
            id: 1,
            name: "Test Provider".to_string(),
            api_url: mock_server.uri(), // 指向 Mock Server
            api_key: "test-key".to_string(),
            enabled: true,
            supported_models: None, // 支持所有
            ..Default::default()
        }];
        state
            .provider_service
            .save_providers("claude", providers)
            .await
            .unwrap();

        // 3. 构造请求
        let body = json!({ "model": "claude-3-opus", "messages": [] });
        let headers = HeaderMap::new();

        // 4. 调用 handle_claude_messages
        let response = handle_claude_messages(State(state.clone()), headers, Json(body)).await;

        // 5. 验证响应
        assert_eq!(response.status(), StatusCode::OK);
        // 注意：Body 是 stream，单元测试只能验证 Status，内容验证比较麻烦，通常 status ok 就行
    }

    // 测试自动故障转移 (Failover)
    #[tokio::test]
    async fn test_forward_request_failover() {
        let (mock_server, state, _temp_dir) = setup_env().await;

        // P1 返回 500
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("Authorization", "Bearer key-1"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        // P2 返回 200
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("Authorization", "Bearer key-2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "resp-2" })))
            .mount(&mock_server)
            .await;

        // 配置 2 个 Provider
        let providers = vec![
            Provider {
                id: 1,
                name: "Provider 1".to_string(),
                api_url: mock_server.uri(),
                api_key: "key-1".to_string(),
                enabled: true,
                level: 1,
                ..Default::default()
            },
            Provider {
                id: 2,
                name: "Provider 2".to_string(),
                api_url: mock_server.uri(),
                api_key: "key-2".to_string(),
                enabled: true,
                level: 2,
                ..Default::default()
            },
        ];
        state
            .provider_service
            .save_providers("claude", providers)
            .await
            .unwrap();

        let body = json!({ "model": "claude-3", "messages": [] });
        let response = handle_claude_messages(State(state), HeaderMap::new(), Json(body)).await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    // 测试无匹配 Provider
    #[tokio::test]
    async fn test_no_provider_available() {
        let (_, state, _temp_dir) = setup_env().await;

        // 未配置 Provider
        let body = json!({ "model": "claude-3", "messages": [] });
        let response = handle_claude_messages(State(state), HeaderMap::new(), Json(body)).await;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    // 测试模型映射
    #[tokio::test]
    async fn test_model_mapping() {
        let (mock_server, state, _temp_dir) = setup_env().await;

        // 验证 Mock Server 收到的请求体中 model 是否被替换
        Mock::given(method("POST"))
            .and(body_json(json!({
                "model": "Internal-Model",
                "messages": []
            })))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let mut mapping = std::collections::HashMap::new();
        mapping.insert("User-Model".to_string(), "Internal-Model".to_string());

        let providers = vec![Provider {
            id: 1,
            name: "Test Mapping".to_string(),
            api_url: mock_server.uri(),
            api_key: "sk".to_string(),
            enabled: true,
            model_mapping: Some(mapping),
            ..Default::default()
        }];
        state
            .provider_service
            .save_providers("claude", providers)
            .await
            .unwrap();

        let body = json!({ "model": "User-Model", "messages": [] });
        let response = handle_claude_messages(State(state), HeaderMap::new(), Json(body)).await;

        assert_eq!(response.status(), StatusCode::OK);
    }
    // 测试通配符模型映射
    #[tokio::test]
    async fn test_wildcard_model_mapping() {
        let (mock_server, state, _temp_dir) = setup_env().await;

        // 验证 Mock Server 收到的请求体中 model 是否被正确替换
        // 期望: claude-3-haiku -> custom-claude-3-haiku
        Mock::given(method("POST"))
            .and(body_json(json!({
                "model": "custom-claude-3-haiku",
                "messages": []
            })))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let mut mapping = std::collections::HashMap::new();
        // 设置通配符映射: claude-* -> custom-claude-*
        mapping.insert("claude-*".to_string(), "custom-claude-*".to_string());

        let providers = vec![Provider {
            id: 1,
            name: "Test Wildcard Mapping".to_string(),
            api_url: mock_server.uri(),
            api_key: "sk".to_string(),
            enabled: true,
            model_mapping: Some(mapping),
            ..Default::default()
        }];
        state
            .provider_service
            .save_providers("claude", providers)
            .await
            .unwrap();

        // 发送请求: claude-3-haiku
        let body = json!({ "model": "claude-3-haiku", "messages": [] });
        let response = handle_claude_messages(State(state), HeaderMap::new(), Json(body)).await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    // 测试 Codex Chat Completions 路由
    #[tokio::test]
    async fn test_codex_chat_completions_route() {
        let (mock_server, state, _temp_dir) = setup_env().await;

        // 1. 设置 Mock Server 响应 for /v1/chat/completions
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions")) // Verify correct path forwarded
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "chatcmpl-123" })))
            .mount(&mock_server)
            .await;

        // 2. 配置 Provider (Codex)
        let providers = vec![Provider {
            id: 1,
            name: "OpenAI".to_string(),
            api_url: mock_server.uri(),
            api_key: "sk-test".to_string(),
            enabled: true,
            supported_models: None,
            ..Default::default()
        }];
        state
            .provider_service
            .save_providers("codex", providers)
            .await
            .unwrap();

        // 3. 构造请求
        let body = json!({ "model": "gpt-4", "messages": [] });

        // 4. 调用
        let response =
            handle_codex_chat_completions(State(state), HeaderMap::new(), Json(body)).await;

        // 5. 验证
        assert_eq!(response.status(), StatusCode::OK);
    }

    // 测试 Codex Legacy Completions 路由
    #[tokio::test]
    async fn test_codex_legacy_completions_route() {
        let (mock_server, state, _temp_dir) = setup_env().await;

        // 1. 设置 Mock Server for /v1/completions
        Mock::given(method("POST"))
            .and(path("/v1/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "cmpl-123" })))
            .mount(&mock_server)
            .await;

        // 2. 配置 Provider
        let providers = vec![Provider {
            id: 1,
            name: "OpenAI Legacy".to_string(),
            api_url: mock_server.uri(),
            api_key: "sk-test".to_string(),
            enabled: true,
            supported_models: None,
            ..Default::default()
        }];
        state
            .provider_service
            .save_providers("codex", providers)
            .await
            .unwrap();

        // 3. Request
        let body = json!({ "model": "davinci-002", "prompt": "test" });

        // 4. Call
        let response =
            handle_codex_legacy_completions(State(state), HeaderMap::new(), Json(body)).await;

        assert_eq!(response.status(), StatusCode::OK);
    }
}
