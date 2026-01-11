//! [INPUT]:
//!   source: ../../../../code-switch/services/logservice.go ([POS]: 原 Go Log 数据模型)
//!
//! [OUTPUT]:
//!   - RequestLog, LogStats, HeatmapStat, ProviderDailyStat
//!
//! [POS]: 日志与统计数据模型定义
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use serde::{Deserialize, Serialize};

/// 请求日志记录
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestLog {
    pub id: i64,
    pub platform: String,
    pub model: String,
    pub provider: String,
    pub http_code: i32,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub cache_create_tokens: i32,
    pub cache_read_tokens: i32,
    pub reasoning_tokens: i32,
    pub is_stream: bool,
    pub duration_sec: f64,
    pub created_at: String,
    #[serde(default)]
    pub has_pricing: bool,
    #[serde(default)]
    pub input_cost: f64,
    #[serde(default)]
    pub output_cost: f64,
    #[serde(default)]
    pub cache_create_cost: f64,
    #[serde(default)]
    pub cache_read_cost: f64,
    #[serde(default)]
    pub ephemeral_5m_cost: f64,
    #[serde(default)]
    pub ephemeral_1h_cost: f64,
    #[serde(default)]
    pub total_cost: f64,
}

/// 热力图统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeatmapStat {
    pub day: String,
    pub total_requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub total_cost: f64,
}

/// 日志统计汇总
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogStats {
    pub total_requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_create_tokens: i64,
    pub cache_read_tokens: i64,
    pub cost_total: f64,
    pub cost_input: f64,
    pub cost_output: f64,
    pub cost_cache_create: f64,
    pub cost_cache_read: f64,
    pub series: Vec<LogStatsSeries>,
}

/// 时序统计数据点
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogStatsSeries {
    pub day: String,
    pub total_requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_create_tokens: i64,
    pub cache_read_tokens: i64,
    pub total_cost: f64,
}

/// 供应商每日统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderDailyStat {
    pub provider: String,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub success_rate: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_create_tokens: i64,
    pub cache_read_tokens: i64,
    pub cost_total: f64,
    #[serde(default)]
    pub hourly_requests: Vec<i64>,
}

/// 费用明细
#[derive(Debug, Clone, Default)]
pub struct CostBreakdown {
    pub has_pricing: bool,
    pub input_cost: f64,
    pub output_cost: f64,
    pub cache_create_cost: f64,
    pub cache_read_cost: f64,
    pub ephemeral_5m_cost: f64,
    pub ephemeral_1h_cost: f64,
    pub total_cost: f64,
}

impl RequestLog {
    pub fn new(platform: &str, provider: &str, model: &str, is_stream: bool) -> Self {
        Self {
            platform: platform.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            is_stream,
            ..Default::default()
        }
    }
}
