//! [INPUT]:
//!   source: ../proxy/events.rs ([POS]: HUD 事件结构)
//!   source: ../../../../openspec/changes/add-mini-hud/specs/proxy/spec.md ([POS]: HUD 规范)
//!
//! [OUTPUT]:
//!   - HudEmitter: HUD 事件发射器
//!   - TokenEstimator: Token 估算器
//!   - HUD_EMITTER: 全局 HUD 事件发射器实例
//!
//! [POS]: HUD 事件管理服务，提供 Token 估算和实时事件发射功能
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::proxy::events::HudEvent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tracing::{debug, trace, warn};

// 估算系数常量
const CHARS_PER_TOKEN_ENGLISH: f32 = 4.0;
const TOKENS_PER_CHAR_CHINESE: f32 = 0.6;

/// 全局 HUD 事件发射器
static HUD_EMITTER: once_cell::sync::OnceCell<Arc<HudEmitter>> = once_cell::sync::OnceCell::new();

/// HUD 事件发射器
///
/// 维护 AppHandle 引用，用于向前端发射 HUD 更新事件
pub struct HudEmitter {
    /// Tauri AppHandle
    app_handle: RwLock<Option<AppHandle>>,
    /// 是否启用 HUD 事件发射
    enabled: AtomicBool,
    /// 缓存的最新事件 (用于 Polling 降级方案)
    last_event: RwLock<Option<HudEvent>>,
}

impl HudEmitter {
    /// 创建新的 HUD 事件发射器
    pub fn new() -> Self {
        Self {
            app_handle: RwLock::new(None),
            enabled: AtomicBool::new(true),
            last_event: RwLock::new(None),
        }
    }

    /// 设置 AppHandle
    pub fn set_app_handle(&self, handle: AppHandle) {
        if let Ok(mut guard) = self.app_handle.write() {
            *guard = Some(handle);
            debug!("HUD 事件发射器已设置 AppHandle");
        }
    }

    /// 启用/禁用事件发射
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// 获取最新缓存的事件
    pub fn get_last_event(&self) -> Option<HudEvent> {
        self.last_event.read().ok().and_then(|guard| guard.clone())
    }

    /// 发射 HUD 更新事件
    ///
    /// 异步发射，不阻塞调用线程
    pub fn emit(&self, event: HudEvent) {
        // 更新缓存
        if let Ok(mut guard) = self.last_event.write() {
            *guard = Some(event.clone());
        }

        if !self.is_enabled() {
            return;
        }

        if let Ok(guard) = self.app_handle.read() {
            if let Some(ref handle) = *guard {
                let handle = handle.clone();
                let event_clone = event.clone();
                tokio::spawn(async move {
                    use tauri::Manager; // 引入 Manager trait 以使用 get_webview_window

                    // 尝试定向发射到 Mini HUD 窗口
                    if let Some(window) = handle.get_webview_window("mini-hud") {
                        if let Err(e) = window.emit("hud-update", event_clone) {
                            warn!(error = %e, "定向发射 HUD 事件失败");
                        } else {
                            // OPTIONAL: Uncomment to debug high frequency events
                            tracing::info!("HUD Emitter: Event sent to window 'mini-hud'");
                        }
                    } else {
                        // 回退到全局广播 (No warning needed in production)
                        if let Err(e) = handle.emit("hud-update", event_clone) {
                            warn!(error = %e, "全局发射 HUD 事件失败");
                        }
                    }
                });
            }
        }
    }

    /// 发射流式更新事件
    pub fn emit_streaming(
        &self,
        provider: &str,
        model: &str,
        delta_tokens: i32,
        total_tokens: i32,
        speed: f32,
    ) {
        trace!(
            delta = delta_tokens,
            total = total_tokens,
            speed = speed,
            "HUD emit_streaming 发射流式事件"
        );
        self.emit(HudEvent::streaming(
            provider,
            model,
            delta_tokens,
            total_tokens,
            speed,
        ));
    }

    /// 发射完成事件
    pub fn emit_completed(&self, provider: &str, model: &str, total_tokens: i32, speed: f32) {
        self.emit(HudEvent::completed(provider, model, total_tokens, speed));
    }

    /// 发射错误事件
    pub fn emit_error(&self, provider: &str, model: &str) {
        self.emit(HudEvent::error(provider, model));
    }
}

impl Default for HudEmitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Token 估算器
///
/// 使用字符比例启发式方法估算 Token 数量
pub struct TokenEstimator {
    /// 累计估算 Token 数
    total_estimated_tokens: i32,
    /// 开始时间
    start_time: Instant,
    /// 上次更新时间
    last_update_time: Instant,
    /// 供应商名称
    provider: String,
    /// 模型名称
    model: String,
}

impl TokenEstimator {
    /// 创建新的 Token 估算器
    pub fn new(provider: &str, model: &str, _output_cost_per_token: f64) -> Self {
        let now = Instant::now();
        Self {
            total_estimated_tokens: 0,
            start_time: now,
            last_update_time: now,
            provider: provider.to_string(),
            model: model.to_string(),
        }
    }

    /// 处理新的文本块，估算 Token 数量
    ///
    /// # 参数
    /// - `text`: 文本内容
    ///
    /// # 返回
    /// 估算的 Token 增量
    pub fn estimate_chunk(&mut self, text: &str) -> i32 {
        let delta = estimate_tokens(text);
        self.total_estimated_tokens += delta;
        self.last_update_time = Instant::now();
        delta
    }

    /// 获取当前速度（tokens/sec）
    pub fn current_speed(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        if elapsed > 0.0 {
            self.total_estimated_tokens as f32 / elapsed
        } else {
            0.0
        }
    }

    /// 获取累计 Token 数
    pub fn total_tokens(&self) -> i32 {
        self.total_estimated_tokens
    }

    /// 用精确值校准（覆盖估算值）
    pub fn calibrate(&mut self, exact_tokens: i32) {
        self.total_estimated_tokens = exact_tokens;
    }

    /// 获取供应商名称
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// 获取模型名称
    pub fn model(&self) -> &str {
        &self.model
    }
}

/// 估算文本的 Token 数量（启发式方法）
///
/// 策略：
/// - 英文: 字符数 / 4
/// - 中文: 字符数 * 0.6
/// - 混合: 加权平均
fn estimate_tokens(text: &str) -> i32 {
    if text.is_empty() {
        return 0;
    }

    let total_chars = text.chars().count();
    let cjk_chars = text.chars().filter(|c| is_cjk_char(*c)).count();
    let non_cjk_chars = total_chars - cjk_chars;

    // 英文: 约 4 字符/token, 中文: 约 0.6 字符/token (即 1 字符 ≈ 1.67 tokens)
    let english_tokens = (non_cjk_chars as f32 / CHARS_PER_TOKEN_ENGLISH).ceil() as i32;
    let chinese_tokens = (cjk_chars as f32 * TOKENS_PER_CHAR_CHINESE).ceil() as i32;

    english_tokens + chinese_tokens
}

/// 判断字符是否为 CJK（中日韩）字符
fn is_cjk_char(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |  // CJK Unified Ideographs Extension A
        '\u{20000}'..='\u{2A6DF}' | // CJK Unified Ideographs Extension B
        '\u{2A700}'..='\u{2B73F}' | // CJK Unified Ideographs Extension C
        '\u{2B740}'..='\u{2B81F}' | // CJK Unified Ideographs Extension D
        '\u{F900}'..='\u{FAFF}' |  // CJK Compatibility Ideographs
        '\u{2F800}'..='\u{2FA1F}'  // CJK Compatibility Ideographs Supplement
    )
}

/// 获取全局 HUD 事件发射器
pub fn get_hud_emitter() -> Arc<HudEmitter> {
    HUD_EMITTER
        .get_or_init(|| Arc::new(HudEmitter::new()))
        .clone()
}

/// 初始化 HUD 事件发射器（设置 AppHandle）
pub fn init_hud_emitter(handle: AppHandle) {
    get_hud_emitter().set_app_handle(handle);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::events::HudStatus;

    #[test]
    fn test_estimate_tokens_english() {
        // 约 4 字符 = 1 token
        let text = "Hello world!"; // 12 chars
        let tokens = estimate_tokens(text);
        assert!(tokens >= 2 && tokens <= 4);
    }

    #[test]
    fn test_estimate_tokens_chinese() {
        // 中文每个字符约 0.6 tokens
        let text = "你好世界"; // 4 chars
        let tokens = estimate_tokens(text);
        assert!(tokens >= 2 && tokens <= 4);
    }

    #[test]
    fn test_estimate_tokens_mixed() {
        let text = "Hello 世界!"; // 混合
        let tokens = estimate_tokens(text);
        assert!(tokens >= 2 && tokens <= 5);
    }

    #[test]
    fn test_estimate_tokens_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_token_estimator() {
        // 假设输出价格为 0.000015 (Claude Sonnet 4)
        let price = 0.000015;
        let mut estimator = TokenEstimator::new("claude", "claude-sonnet-4", price);

        let delta1 = estimator.estimate_chunk("Hello world!");
        assert!(delta1 > 0);

        let delta2 = estimator.estimate_chunk("你好世界");
        assert!(delta2 > 0);

        assert_eq!(estimator.total_tokens(), delta1 + delta2);
        assert!(estimator.current_speed() > 0.0);
    }

    #[test]
    fn test_is_cjk_char() {
        assert!(is_cjk_char('中'));
        assert!(is_cjk_char('日'));
        assert!(is_cjk_char('韓'));
        assert!(!is_cjk_char('A'));
        assert!(!is_cjk_char('1'));
        assert!(!is_cjk_char(' '));
    }

    #[test]
    fn test_hud_emitter_default() {
        let emitter = HudEmitter::new();
        assert!(emitter.is_enabled());

        emitter.set_enabled(false);
        assert!(!emitter.is_enabled());
    }

    #[test]
    fn test_token_estimator_calibrate_and_cost() {
        let mut estimator = TokenEstimator::new("anthropic", "claude-3", 0.001);
        estimator.estimate_chunk("Hello Claude");
        estimator.calibrate(120);

        assert_eq!(estimator.total_tokens(), 120);
        assert_eq!(estimator.provider(), "anthropic");
        assert_eq!(estimator.model(), "claude-3");
    }

    #[test]
    fn test_estimate_tokens_cjk_vs_ascii_distribution() {
        let ascii = estimate_tokens("aaaa");
        let cjk = estimate_tokens("你好你好");
        let mixed = estimate_tokens("Hi 世界");

        assert!(ascii > 0);
        assert!(cjk > ascii, "CJK tokens should weigh heavier than ASCII");
        assert!(mixed >= ascii && mixed <= cjk + ascii);
    }

    #[test]
    fn test_hud_emitter_caches_events_when_disabled() {
        let emitter = HudEmitter::new();
        emitter.set_enabled(false);

        let event = HudEvent::error("anthropic", "claude");
        emitter.emit(event.clone());

        let cached = emitter
            .get_last_event()
            .expect("cache should store last event");
        assert_eq!(cached.provider, "anthropic");
        assert_eq!(cached.status, HudStatus::Error);
    }

    #[test]
    fn test_hud_emitter_emit_helpers_store_latest_status() {
        let emitter = HudEmitter::new();

        emitter.emit_streaming("anthropic", "claude", 5, 10, 12.0);
        let streaming = emitter.get_last_event().unwrap();
        assert_eq!(streaming.status, HudStatus::Streaming);
        assert_eq!(streaming.delta_tokens, 5);

        emitter.emit_completed("anthropic", "claude", 10, 8.0);
        let completed = emitter.get_last_event().unwrap();
        assert_eq!(completed.status, HudStatus::Completed);
        assert_eq!(completed.total_tokens, 10);

        emitter.emit_error("anthropic", "claude");
        let error = emitter.get_last_event().unwrap();
        assert_eq!(error.status, HudStatus::Error);
    }
}
