//! ---
//! [INPUT]: {OpenSpec}
//!   source: ../../../../openspec/changes/add-mini-hud/specs/proxy/spec.md ([POS]: HUD 事件格式规范)
//!
//! [OUTPUT]: {HudEvent, HudStatus} - HUD 更新事件结构体，用于前端实时显示
//!
//! [POS]: 代理模块 HUD 事件定义，包含实时 Token 统计和状态信息
//!
//! [PROTOCOL]: FractalFlow v1.0
//! ---

use serde::{Deserialize, Serialize};

/// HUD 事件状态类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HudStatus {
    /// 请求开始
    Starting,
    /// 流式传输中
    Streaming,
    /// 请求完成
    Completed,
    /// 请求失败
    Error,
}

impl Default for HudStatus {
    fn default() -> Self {
        Self::Starting
    }
}

/// HUD 更新事件
///
/// 符合 OpenSpec 规范的 Payload 结构:
/// ```json
/// {
///   "provider": "claude",
///   "model": "claude-sonnet-4-20250514",
///   "delta_tokens": 12,
///   "total_tokens": 156,
///   "current_cost": 0.0042,
///   "speed": 45.5,
///   "status": "streaming"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudEvent {
    /// 供应商名称
    pub provider: String,
    /// 模型名称
    pub model: String,
    /// 本次 token 增量（估算或精确）
    pub delta_tokens: i32,
    /// 累计 token 总数
    pub total_tokens: i32,
    /// 速度（tokens/sec）
    pub speed: f32,
    /// 事件状态
    pub status: HudStatus,
}

impl HudEvent {
    /// 创建新的 HUD 事件
    pub fn new(provider: &str, model: &str) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            delta_tokens: 0,
            total_tokens: 0,
            speed: 0.0,
            status: HudStatus::Starting,
        }
    }

    /// 创建流式更新事件
    pub fn streaming(
        provider: &str,
        model: &str,
        delta_tokens: i32,
        total_tokens: i32,
        speed: f32,
    ) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            delta_tokens,
            total_tokens,
            speed,
            status: HudStatus::Streaming,
        }
    }

    /// 创建完成事件
    pub fn completed(provider: &str, model: &str, total_tokens: i32, speed: f32) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            delta_tokens: 0,
            total_tokens,
            speed,
            status: HudStatus::Completed,
        }
    }

    /// 创建错误事件
    pub fn error(provider: &str, model: &str) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            delta_tokens: 0,
            total_tokens: 0,
            speed: 0.0,
            status: HudStatus::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_event_new() {
        let event = HudEvent::new("claude", "claude-sonnet-4");
        assert_eq!(event.provider, "claude");
        assert_eq!(event.model, "claude-sonnet-4");
        assert_eq!(event.status, HudStatus::Starting);
    }

    #[test]
    fn test_hud_event_streaming() {
        let event = HudEvent::streaming("openai", "gpt-4", 12, 156, 45.5);
        assert_eq!(event.delta_tokens, 12);
        assert_eq!(event.total_tokens, 156);
        assert_eq!(event.status, HudStatus::Streaming);
    }

    #[test]
    fn test_hud_status_serialization() {
        let status = HudStatus::Streaming;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""streaming""#);
    }
}
