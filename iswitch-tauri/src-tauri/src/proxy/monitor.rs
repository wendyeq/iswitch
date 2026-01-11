//! [INPUT]:
//!   source: ../models/log.rs ([POS]: RequestLog 结构体)
//!   source: ../db/request_log.rs ([POS]: 日志持久化)
//!   source: ../services/pricing_service.rs ([POS]: 定价服务)
//!   source: ../services/hud_service.rs ([POS]: HUD 事件发射)
//!
//! [OUTPUT]:
//!   - process_log_entry: 处理请求日志
//!   - monitor_response: 监控响应流并发射 HUD 更新事件
//!
//! [POS]: 响应监控，usage 解析，费用计算（调用 PricingService），日志记录，HUD 实时更新
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::db::request_log;
use crate::models::RequestLog;

use crate::services::hud_service::{get_hud_emitter, TokenEstimator};
use crate::services::pricing_service::calculate_cost;
use axum::body::Bytes;
use serde_json::Value;
use sqlx::{Pool, Sqlite};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{debug, error, trace, warn};

// SSE 协议常量
const SSE_DATA_PREFIX: &str = "data:";
const SSE_DONE_SIGNAL: &str = "[DONE]";
pub struct MonitorContext {
    pub platform: String,
    pub provider: String,
    pub model: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub is_stream: bool,
    pub pool: Arc<Pool<Sqlite>>,
}

pub async fn process_log_entry(
    mut rx: mpsc::Receiver<Bytes>,
    ctx: MonitorContext,
    status: axum::http::StatusCode,
) {
    let mut total_bytes = Vec::new();
    while let Some(chunk) = rx.recv().await {
        total_bytes.extend_from_slice(&chunk);
    }

    let duration = (chrono::Utc::now() - ctx.start_time).num_milliseconds() as f64 / 1000.0;

    let mut log = RequestLog::new(&ctx.platform, &ctx.provider, &ctx.model, ctx.is_stream);
    log.http_code = status.as_u16() as i32;
    log.duration_sec = duration;
    // Format: YYYY-MM-DDTHH:MM:SSZ (Explicit UTC)
    log.created_at = ctx.start_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    if status.is_success() {
        let body_str = String::from_utf8_lossy(&total_bytes);
        if ctx.is_stream {
            parse_usage_from_stream(&body_str, &ctx.platform, &mut log);
        } else {
            if let Ok(json) = serde_json::from_slice::<Value>(&total_bytes) {
                parse_usage_from_json(&json, &ctx.platform, &mut log);
            }
        }
    }

    // 使用 PricingService 计算费用
    calculate_cost(&mut log);

    // Insert to DB
    if let Err(e) = request_log::insert_log(&ctx.pool, &log).await {
        error!("Failed to insert request log: {}", e);
    } else {
        debug!(
            "Request log inserted for {}: in={} (create={}, read={}), out={} (reason={}), cost=${:.6}, duration={}s",
            ctx.model, log.input_tokens, log.cache_create_tokens, log.cache_read_tokens, log.output_tokens, log.reasoning_tokens, log.total_cost, log.duration_sec
        );
    }
}

struct SseContentBuffer {
    buffer: String,
    /// 调试：已处理的 chunk 计数（用于控制日志频率）
    chunk_count: u32,
}

impl SseContentBuffer {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            chunk_count: 0,
        }
    }

    fn push_chunk(&mut self, chunk: &str) -> Vec<String> {
        self.buffer.push_str(chunk);
        self.chunk_count += 1;
        let mut results = Vec::new();

        // 查找最后一个换行符，确保只处理完整的行
        if let Some(last_newline) = self.buffer.rfind('\n') {
            let complete_part = self.buffer[..=last_newline].to_string();
            let remaining = self.buffer[last_newline + 1..].to_string();
            self.buffer = remaining;

            // 提取 SSE 内容
            if let Some(content) = extract_sse_content(&complete_part) {
                results.push(content);
            } else if self.chunk_count <= 3 {
                // 调试：前几个 chunk 如果提取失败，记录原始 SSE 数据
                trace!(
                    chunk_count = self.chunk_count,
                    raw_sse = %complete_part.chars().take(300).collect::<String>(),
                    "SSE 内容提取失败，检查原始数据格式"
                );
            }
        }

        results
    }
}

pub fn monitor_response(
    response: axum::response::Response,
    ctx: MonitorContext,
) -> axum::response::Response {
    use crate::proxy::events::HudEvent;
    use axum::body::Body;
    use futures::StreamExt;

    let (parts, body) = response.into_parts();

    // 使用有界channel防止内存泄漏（容量1000）
    let (tx, rx) = mpsc::channel::<Bytes>(1000);

    let body_stream = body.into_data_stream();

    let status = parts.status;

    // 克隆上下文信息用于日志处理
    let ctx_for_log = MonitorContext {
        platform: ctx.platform.clone(),
        provider: ctx.provider.clone(),
        model: ctx.model.clone(),
        start_time: ctx.start_time,
        is_stream: ctx.is_stream,
        pool: ctx.pool.clone(),
    };

    tokio::spawn(async move {
        process_log_entry(rx, ctx_for_log, status).await;
    });

    // 只在流式请求时发射 HUD 事件
    if ctx.is_stream {
        // 获取输出 token 价格
        let output_cost = crate::services::pricing_service::PRICING_SERVICE
            .get_pricing(&ctx.model)
            .map(|p| p.output_cost_per_token)
            .unwrap_or(0.0);

        // 创建 Token 估算器和状态（使用 Arc<Mutex> 在闭包间共享）
        let estimator = Arc::new(Mutex::new(TokenEstimator::new(
            &ctx.provider,
            &ctx.model,
            output_cost,
        )));
        let sse_buffer = Arc::new(Mutex::new(SseContentBuffer::new()));
        let emitter = get_hud_emitter();
        let provider = ctx.provider.clone();
        let model = ctx.model.clone();

        // 发射启动事件
        emitter.emit(HudEvent::new(&provider, &model));

        let buffer = sse_buffer.clone();
        let estimator_for_stream = estimator.clone();
        let emitter_for_stream = emitter.clone();
        let provider_for_stream = provider.clone();
        let model_for_stream = model.clone();

        let new_stream = body_stream.map(move |chunk| {
            if let Ok(ref bytes) = chunk {
                // 发送到日志处理 channel
                if let Err(e) = tx.try_send(bytes.clone()) {
                    warn!("Log channel full/closed, dropping chunk (stream): {}", e);
                }

                // 尝试解析 SSE 数据并估算 token
                if let Ok(text) = std::str::from_utf8(bytes) {
                    let segments = buffer
                        .lock()
                        .map(|mut guard| guard.push_chunk(text))
                        .unwrap_or_default();

                    if !segments.is_empty() {
                        if let Ok(mut estimator_guard) = estimator_for_stream.lock() {
                            let mut delta_total = 0;
                            for segment in segments {
                                delta_total += estimator_guard.estimate_chunk(&segment);
                            }

                            if delta_total > 0 {
                                let total = estimator_guard.total_tokens();
                                let speed = estimator_guard.current_speed();

                                emitter_for_stream.emit_streaming(
                                    &provider_for_stream,
                                    &model_for_stream,
                                    delta_total,
                                    total,
                                    speed,
                                );

                                trace!(
                                    delta = delta_total,
                                    total = total,
                                    speed = speed,
                                    "HUD streaming update emitted"
                                );
                            }
                        }
                    }
                }
            }
            chunk
        });

        axum::response::Response::from_parts(parts, Body::from_stream(new_stream))
    } else {
        // 非流式请求，不发射 HUD 事件
        let new_stream = body_stream.map(move |chunk| {
            if let Ok(ref bytes) = chunk {
                if let Err(e) = tx.try_send(bytes.clone()) {
                    warn!(
                        "Log channel full/closed, dropping chunk (non-stream): {}",
                        e
                    );
                }
            }
            chunk
        });

        axum::response::Response::from_parts(parts, Body::from_stream(new_stream))
    }
}

/// 从 SSE 数据中提取内容文本
///
/// 解析格式如: `data: {"content": "Hello", ...}` 或 `data: {"delta": {"text": "Hello"}, ...}`
fn extract_sse_content(text: &str) -> Option<String> {
    let mut content_parts = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with(SSE_DATA_PREFIX) {
            continue;
        }

        let json_str = line[SSE_DATA_PREFIX.len()..].trim();
        if json_str == SSE_DONE_SIGNAL || json_str.is_empty() {
            continue;
        }

        match serde_json::from_str::<Value>(json_str) {
            Ok(val) => collect_text_candidates(&val, &mut content_parts, None),
            Err(_) => {
                // 备用方案：某些 provider 直接返回纯文本
                if !json_str.starts_with('{') && !json_str.starts_with('[') && !json_str.is_empty()
                {
                    content_parts.push(json_str.to_string());
                }
            }
        }
    }

    if content_parts.is_empty() {
        None
    } else {
        Some(content_parts.join(""))
    }
}

fn collect_text_candidates(value: &Value, parts: &mut Vec<String>, key_hint: Option<&str>) {
    match value {
        Value::String(s) => {
            if should_collect_string(key_hint, s) {
                parts.push(s.to_string());
            }
        }
        Value::Array(arr) => {
            for item in arr {
                collect_text_candidates(item, parts, key_hint);
            }
        }
        Value::Object(map) => {
            for (key, child) in map {
                let next_hint = Some(key.as_str());
                collect_text_candidates(child, parts, next_hint);
            }
        }
        _ => {}
    }
}

fn should_collect_string(key_hint: Option<&str>, value: &str) -> bool {
    if value.trim().is_empty() {
        return false;
    }

    // 支持的 SSE 内容字段名（兼容多种 provider 格式）
    // - text: 标准文本字段
    // - delta: OpenAI stream delta
    // - content: Claude/OpenAI 的内容字段
    // - output_text, response_text: 一些 provider 的变体
    // - message, partial_message: MiniMax 等 provider 可能使用
    // - reasoning, reasoning_content: 推理模型的思考内容
    // - thinking: MiniMax 扩展 Claude API 的思考内容字段
    matches!(
        key_hint,
        Some("text")
            | Some("delta")
            | Some("content")
            | Some("output_text")
            | Some("response_text")
            | Some("message")
            | Some("partial_message")
            | Some("reasoning")
            | Some("reasoning_content")
            | Some("thinking")
    )
}

fn parse_usage_from_stream(body: &str, platform: &str, log: &mut RequestLog) {
    if platform == "claude" {
        // SSE format:
        // event: message_start
        // data: { ... "usage": { "input_tokens": 123 } ... }
        //
        // event: message_delta
        // data: { ... "usage": { "output_tokens": 456 } ... }

        let _in_input = false;
        let _in_output = false;

        for line in body.lines() {
            let line = line.trim();
            if line.starts_with(SSE_DATA_PREFIX) {
                let json_str = line[SSE_DATA_PREFIX.len()..].trim();
                if json_str == SSE_DONE_SIGNAL || json_str.is_empty() {
                    continue;
                }
                if let Ok(val) = serde_json::from_str::<Value>(json_str) {
                    // 递归查找 usage 信息 (通用策略)
                    let found_usage = find_usage_recursive(&val);
                    if let Some(u) = found_usage {
                        if u.input > 0 {
                            log.input_tokens = std::cmp::max(log.input_tokens, u.input as i32);
                        }
                        if u.output > 0 {
                            log.output_tokens = std::cmp::max(log.output_tokens, u.output as i32);
                        }
                        if u.reasoning > 0 {
                            log.reasoning_tokens =
                                std::cmp::max(log.reasoning_tokens, u.reasoning as i32);
                        }
                        if u.cache_create > 0 {
                            log.cache_create_tokens =
                                std::cmp::max(log.cache_create_tokens, u.cache_create as i32);
                        }
                        if u.cache_read > 0 {
                            log.cache_read_tokens =
                                std::cmp::max(log.cache_read_tokens, u.cache_read as i32);
                        }
                    }

                    // 如果推理 token 独立存在于顶层，也尝试捕获 (部分厂商支持)
                    if let Some(rt) = val
                        .get("usage")
                        .and_then(|u| u.get("reasoning_tokens"))
                        .and_then(|v| v.as_i64())
                    {
                        log.reasoning_tokens = std::cmp::max(log.reasoning_tokens, rt as i32);
                    }

                    // 备选方案：检查 Anthropic 特有的 message.usage
                    if let Some(usage) = val.get("message").and_then(|m| m.get("usage")) {
                        if let Some(it) = usage.get("input_tokens").and_then(|v| v.as_i64()) {
                            log.input_tokens = std::cmp::max(log.input_tokens, it as i32);
                        }
                        if let Some(cc) = usage
                            .get("cache_creation_input_tokens")
                            .and_then(|v| v.as_i64())
                        {
                            log.cache_create_tokens =
                                std::cmp::max(log.cache_create_tokens, cc as i32);
                        }
                        if let Some(cr) = usage
                            .get("cache_read_input_tokens")
                            .and_then(|v| v.as_i64())
                        {
                            log.cache_read_tokens = std::cmp::max(log.cache_read_tokens, cr as i32);
                        }
                    }
                }
            }
        }
    } else if platform == "codex" {
        // OpenAI Responses API 格式 (与 Go 实现对齐)
        // 响应结构: { "response": { "usage": { "input_tokens": 123, ... } } }
        for line in body.lines() {
            let line = line.trim();
            if line.starts_with(SSE_DATA_PREFIX) {
                let json_str = line[SSE_DATA_PREFIX.len()..].trim();
                if json_str == SSE_DONE_SIGNAL || json_str.is_empty() {
                    continue;
                }
                if let Ok(val) = serde_json::from_str::<Value>(json_str) {
                    // 优先尝试 response.usage (Responses API 格式)
                    if let Some(usage) = val.get("response").and_then(|r| r.get("usage")) {
                        if let Some(it) = usage.get("input_tokens").and_then(|v| v.as_i64()) {
                            log.input_tokens += it as i32;
                        }
                        if let Some(ot) = usage.get("output_tokens").and_then(|v| v.as_i64()) {
                            log.output_tokens += ot as i32;
                        }
                        // 解析 cached_tokens (在 input_tokens_details 中)
                        if let Some(ct) = usage
                            .get("input_tokens_details")
                            .and_then(|d| d.get("cached_tokens"))
                            .and_then(|v| v.as_i64())
                        {
                            log.cache_read_tokens += ct as i32;
                        }
                        // 解析 reasoning_tokens (在 output_tokens_details 中)
                        if let Some(rt) = usage
                            .get("output_tokens_details")
                            .and_then(|d| d.get("reasoning_tokens"))
                            .and_then(|v| v.as_i64())
                        {
                            log.reasoning_tokens += rt as i32;
                        }
                    }
                    // 备选：直接检查 usage (Chat Completions API 格式)
                    else if let Some(usage) = val.get("usage") {
                        if let Some(it) = usage
                            .get("prompt_tokens")
                            .or(usage.get("input_tokens"))
                            .and_then(|v| v.as_i64())
                        {
                            log.input_tokens = it as i32;
                        }
                        if let Some(ot) = usage
                            .get("completion_tokens")
                            .or(usage.get("output_tokens"))
                            .and_then(|v| v.as_i64())
                        {
                            log.output_tokens = ot as i32;
                        }
                        // 解析推理 token
                        if let Some(rt) = usage
                            .get("reasoning_tokens")
                            .or(usage
                                .get("completion_tokens_details")
                                .and_then(|d| d.get("reasoning_tokens")))
                            .and_then(|v| v.as_i64())
                        {
                            log.reasoning_tokens = rt as i32;
                        }
                        // 解析缓存读取 token
                        if let Some(ct) = usage
                            .get("prompt_tokens_details")
                            .and_then(|d| d.get("cached_tokens"))
                            .and_then(|v| v.as_i64())
                        {
                            log.cache_read_tokens = ct as i32;
                        }
                    }
                }
            }
        }
    }
}

/// 提取的 Token 信息结构
struct ExtractedUsage {
    input: i64,
    output: i64,
    reasoning: i64,
    cache_create: i64,
    cache_read: i64,
}

/// 递归查找 JSON 中的 usage 字段并提取各类 tokens
fn find_usage_recursive(val: &Value) -> Option<ExtractedUsage> {
    if let Some(obj) = val.as_object() {
        if let Some(usage) = obj.get("usage") {
            let input = usage
                .get("input_tokens")
                .or(usage.get("prompt_tokens"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let output = usage
                .get("output_tokens")
                .or(usage.get("completion_tokens"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let reasoning = usage
                .get("reasoning_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let cache_create = usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let cache_read = usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            // 如果返回了推理 token，累加到输出中 (或保持独立)
            return Some(ExtractedUsage {
                input,
                output,
                reasoning,
                cache_create,
                cache_read,
            });
        }
        for v in obj.values() {
            if let Some(res) = find_usage_recursive(v) {
                return Some(res);
            }
        }
    } else if let Some(arr) = val.as_array() {
        for v in arr {
            if let Some(res) = find_usage_recursive(v) {
                return Some(res);
            }
        }
    }
    None
}

fn parse_usage_from_json(json: &Value, _platform: &str, log: &mut RequestLog) {
    if let Some(usage) = json.get("usage") {
        let it = usage
            .get("input_tokens")
            .or(usage.get("prompt_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let ot = usage
            .get("output_tokens")
            .or(usage.get("completion_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let rt = usage
            .get("reasoning_tokens")
            .or(usage
                .get("output_tokens_details")
                .and_then(|d| d.get("reasoning_tokens")))
            .or(usage
                .get("completion_tokens_details")
                .and_then(|d| d.get("reasoning_tokens")))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        log.input_tokens = it as i32;
        log.output_tokens = ot as i32;
        log.reasoning_tokens = rt as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== parse_usage_from_stream 测试 =====

    #[test]
    fn test_parse_usage_claude() {
        let mut log = RequestLog::default();
        let body = "event: message_start\ndata: {\"message\":{\"usage\":{\"input_tokens\":10}}}\n\nevent: message_delta\ndata: {\"usage\":{\"output_tokens\":20}}";
        parse_usage_from_stream(body, "claude", &mut log);
        assert_eq!(log.input_tokens, 10);
        assert_eq!(log.output_tokens, 20);
    }

    #[test]
    fn test_parse_usage_codex() {
        // 测试 OpenAI Responses API 格式 (response.usage)
        let mut log = RequestLog::default();
        let body = r#"data: {"response":{"usage":{"input_tokens":100,"output_tokens":50,"input_tokens_details":{"cached_tokens":20},"output_tokens_details":{"reasoning_tokens":10}}}}"#;
        parse_usage_from_stream(body, "codex", &mut log);
        assert_eq!(log.input_tokens, 100);
        assert_eq!(log.output_tokens, 50);
        assert_eq!(log.cache_read_tokens, 20);
        assert_eq!(log.reasoning_tokens, 10);
    }

    #[test]
    fn test_parse_usage_codex_chat_completions_fallback() {
        // 测试 Chat Completions API 格式 (usage 在顶层) 作为回退
        let mut log = RequestLog::default();
        let body = "data: {\"usage\":{\"prompt_tokens\":5,\"completion_tokens\":15}}";
        parse_usage_from_stream(body, "codex", &mut log);
        assert_eq!(log.input_tokens, 5);
        assert_eq!(log.output_tokens, 15);
    }

    #[test]
    fn test_calculate_cost_integration() {
        // 测试与 PricingService 的集成
        let mut log = RequestLog::default();
        log.model = "claude-sonnet-4-20250514".to_string();
        log.input_tokens = 1_000_000;
        log.output_tokens = 100_000;
        log.cache_create_tokens = 50_000;
        log.cache_read_tokens = 200_000;

        calculate_cost(&mut log);

        // 验证费用计算使用了新的定价服务
        // 如果 has_pricing 为 true，说明成功匹配到定价
        assert!(log.has_pricing);
        assert!(log.total_cost > 0.0);
    }

    // ===== Claude cache tokens 测试 =====

    /// 测试 Claude 的 cache_creation_input_tokens 和 cache_read_input_tokens 解析
    #[test]
    fn test_parse_usage_claude_cache_tokens() {
        let mut log = RequestLog::default();
        let body = r#"event: message_start
data: {"message":{"usage":{"input_tokens":100,"cache_creation_input_tokens":50,"cache_read_input_tokens":30}}}
event: message_delta
data: {"usage":{"output_tokens":200}}"#;
        parse_usage_from_stream(body, "claude", &mut log);
        assert_eq!(log.input_tokens, 100);
        assert_eq!(log.output_tokens, 200);
        assert_eq!(log.cache_create_tokens, 50);
        assert_eq!(log.cache_read_tokens, 30);
    }

    // ===== 边界条件测试 =====

    /// 测试空数据不 panic
    #[test]
    fn test_parse_usage_empty_body() {
        let mut log = RequestLog::default();
        parse_usage_from_stream("", "claude", &mut log);
        assert_eq!(log.input_tokens, 0);
        assert_eq!(log.output_tokens, 0);

        let mut log2 = RequestLog::default();
        parse_usage_from_stream("", "codex", &mut log2);
        assert_eq!(log2.input_tokens, 0);
        assert_eq!(log2.output_tokens, 0);
    }

    /// 测试 [DONE] 信号被正确跳过
    #[test]
    fn test_parse_usage_done_signal() {
        let mut log = RequestLog::default();
        let body = "data: {\"usage\":{\"input_tokens\":10}}\ndata: [DONE]";
        parse_usage_from_stream(body, "claude", &mut log);
        assert_eq!(log.input_tokens, 10);
        // [DONE] 之后不应干扰结果
    }

    /// 测试格式错误的 JSON 不 panic
    #[test]
    fn test_parse_usage_malformed_json() {
        let mut log = RequestLog::default();
        let body = "data: {invalid json}\ndata: {\"usage\":{\"input_tokens\":5}}";
        parse_usage_from_stream(body, "claude", &mut log);
        // 应该跳过无效 JSON，解析有效的部分
        assert_eq!(log.input_tokens, 5);
    }

    /// 测试只有 event 行没有 data 行
    #[test]
    fn test_parse_usage_event_only() {
        let mut log = RequestLog::default();
        let body = "event: message_start\nevent: message_delta";
        parse_usage_from_stream(body, "claude", &mut log);
        assert_eq!(log.input_tokens, 0);
        assert_eq!(log.output_tokens, 0);
    }

    /// 测试未知平台不 panic
    #[test]
    fn test_parse_usage_unknown_platform() {
        let mut log = RequestLog::default();
        let body = "data: {\"usage\":{\"input_tokens\":999}}";
        parse_usage_from_stream(body, "unknown_platform", &mut log);
        // 未知平台应该不处理，tokens 保持为 0
        assert_eq!(log.input_tokens, 0);
    }

    // ===== parse_usage_from_json 测试 =====

    /// 测试非流式 JSON 响应解析 (input_tokens/output_tokens)
    #[test]
    fn test_parse_usage_from_json_standard() {
        let mut log = RequestLog::default();
        let json: Value =
            serde_json::from_str(r#"{"usage":{"input_tokens":100,"output_tokens":200}}"#).unwrap();
        parse_usage_from_json(&json, "claude", &mut log);
        assert_eq!(log.input_tokens, 100);
        assert_eq!(log.output_tokens, 200);
    }

    /// 测试非流式 JSON 响应解析 (prompt_tokens/completion_tokens OpenAI 格式)
    #[test]
    fn test_parse_usage_from_json_openai_format() {
        let mut log = RequestLog::default();
        let json: Value =
            serde_json::from_str(r#"{"usage":{"prompt_tokens":50,"completion_tokens":150}}"#)
                .unwrap();
        parse_usage_from_json(&json, "codex", &mut log);
        assert_eq!(log.input_tokens, 50);
        assert_eq!(log.output_tokens, 150);
    }

    /// 测试 JSON 中没有 usage 字段
    #[test]
    fn test_parse_usage_from_json_no_usage() {
        let mut log = RequestLog::default();
        let json: Value = serde_json::from_str(r#"{"content":"hello"}"#).unwrap();
        parse_usage_from_json(&json, "claude", &mut log);
        assert_eq!(log.input_tokens, 0);
        assert_eq!(log.output_tokens, 0);
    }

    // ===== find_usage_recursive 测试 =====

    /// 测试顶层 usage
    #[test]
    fn test_find_usage_recursive_top_level() {
        let json: Value =
            serde_json::from_str(r#"{"usage":{"input_tokens":10,"output_tokens":20}}"#).unwrap();
        let result = find_usage_recursive(&json);
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.input, 10);
        assert_eq!(u.output, 20);
    }

    /// 测试嵌套 usage
    #[test]
    fn test_find_usage_recursive_nested() {
        let json: Value = serde_json::from_str(
            r#"{"response":{"data":{"usage":{"input_tokens":5,"output_tokens":15}}}}"#,
        )
        .unwrap();
        let result = find_usage_recursive(&json);
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.input, 5);
        assert_eq!(u.output, 15);
    }

    /// 测试数组中的 usage
    #[test]
    fn test_find_usage_recursive_in_array() {
        let json: Value = serde_json::from_str(
            r#"{"items":[{"id":1},{"usage":{"input_tokens":3,"output_tokens":7}}]}"#,
        )
        .unwrap();
        let result = find_usage_recursive(&json);
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.input, 3);
        assert_eq!(u.output, 7);
    }

    /// 测试没有 usage 返回 None
    #[test]
    fn test_find_usage_recursive_not_found() {
        let json: Value = serde_json::from_str(r#"{"content":"hello","id":123}"#).unwrap();
        let result = find_usage_recursive(&json);
        assert!(result.is_none());
    }

    /// 测试 cache tokens 和 reasoning tokens
    #[test]
    fn test_find_usage_recursive_with_cache_and_reasoning() {
        let json: Value = serde_json::from_str(
            r#"{"usage":{
                "input_tokens":100,
                "output_tokens":50,
                "cache_creation_input_tokens":25,
                "cache_read_input_tokens":10,
                "reasoning_tokens":30
            }}"#,
        )
        .unwrap();
        let result = find_usage_recursive(&json);
        assert!(result.is_some());
        let u = result.unwrap();
        assert_eq!(u.input, 100);
        // output 保持原样，不再合并 reasoning (合并逻辑移动到了 calculation)
        assert_eq!(u.output, 50);
        assert_eq!(u.reasoning, 30);
        assert_eq!(u.cache_create, 25);
        assert_eq!(u.cache_read, 10);
    }

    // ===== 多条 SSE 消息累加测试 =====

    /// 测试 Codex 多条消息累加
    #[test]
    fn test_parse_usage_codex_multiple_messages() {
        let mut log = RequestLog::default();
        let body = r#"data: {"response":{"usage":{"input_tokens":10,"output_tokens":5}}}
data: {"response":{"usage":{"input_tokens":20,"output_tokens":10}}}"#;
        parse_usage_from_stream(body, "codex", &mut log);
        // Codex 使用 += 累加，所以应该是 30 和 15
        assert_eq!(log.input_tokens, 30);
        assert_eq!(log.output_tokens, 15);
    }

    /// 测试 Claude 多条消息取最大值
    #[test]
    fn test_parse_usage_claude_multiple_messages_max() {
        let mut log = RequestLog::default();
        let body = "data: {\"usage\":{\"input_tokens\":5}}\ndata: {\"usage\":{\"input_tokens\":10,\"output_tokens\":20}}";
        parse_usage_from_stream(body, "claude", &mut log);
        // Claude 使用 max()，所以应该是 10 和 20
        assert_eq!(log.input_tokens, 10);
        assert_eq!(log.output_tokens, 20);
    }

    // ===== extract_sse_content 测试 =====

    /// 测试 Claude 格式的 SSE 内容提取
    #[test]
    fn test_extract_sse_content_claude() {
        let sse = r#"data: {"type":"content_block_delta","delta":{"text":"Hello"}}"#;
        let content = extract_sse_content(sse);
        assert_eq!(content, Some("Hello".to_string()));
    }

    /// 测试 OpenAI/Codex 格式的 SSE 内容提取
    #[test]
    fn test_extract_sse_content_openai() {
        let sse = r#"data: {"choices":[{"delta":{"content":" world"}}]}"#;
        let content = extract_sse_content(sse);
        assert_eq!(content, Some(" world".to_string()));
    }

    /// 测试 Responses API 的 response.delta 结构
    #[test]
    fn test_extract_sse_content_response_delta_array() {
        let sse = r#"data: {"type":"response.delta","delta":{"content":[{"type":"output_text.delta","text":"Hello"},{"type":"output_text.delta","text":" World"}]}}"#;
        let content = extract_sse_content(sse);
        assert_eq!(content, Some("Hello World".to_string()));
    }

    /// 测试 response.output_text.delta 结构
    #[test]
    fn test_extract_sse_content_output_text_delta() {
        let sse = r#"data: {"type":"response.output_text.delta","delta":"Hello"}"#;
        let content = extract_sse_content(sse);
        assert_eq!(content, Some("Hello".to_string()));
    }

    /// 测试多行 SSE 数据
    #[test]
    fn test_extract_sse_content_multiline() {
        let sse = r#"data: {"delta":{"text":"Hello"}}
data: {"delta":{"text":" World"}}"#;
        let content = extract_sse_content(sse);
        assert_eq!(content, Some("Hello World".to_string()));
    }

    /// 测试 [DONE] 信号被跳过
    #[test]
    fn test_extract_sse_content_done() {
        let sse = r#"data: {"delta":{"text":"Hello"}}
data: [DONE]"#;
        let content = extract_sse_content(sse);
        assert_eq!(content, Some("Hello".to_string()));
    }

    /// 测试空 SSE 数据
    #[test]
    fn test_extract_sse_content_empty() {
        let sse = "data: {}";
        let content = extract_sse_content(sse);
        assert_eq!(content, None);
    }

    /// 测试无 data 前缀的行
    #[test]
    fn test_extract_sse_content_no_data_prefix() {
        let sse = r#"event: content_block_delta
id: msg123"#;
        let content = extract_sse_content(sse);
        assert_eq!(content, None);
    }

    /// 测试 SSE buffer 在 chunk 被拆分时仍能恢复完整文本
    #[test]
    fn test_sse_buffer_handles_split_chunks() {
        let mut buffer = SseContentBuffer::new();
        // 第一段没有换行，不应返回内容
        assert!(buffer
            .push_chunk("data: {\"delta\":{\"text\":\"Hello")
            .is_empty());

        let result = buffer.push_chunk(" World\"}}\n\n");
        assert_eq!(result, vec!["Hello World".to_string()]);
    }
}
