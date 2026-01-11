//! [INPUT]:
//!   source: ../resources/model-pricing.json ([POS]: 模型定价配置)
//!   source: ../models/log.rs ([POS]: RequestLog 结构体)
//!
//! [OUTPUT]:
//!   - PricingService: 定价服务，提供模型定价查询和费用计算
//!   - ModelPricing: 模型定价信息结构体
//!
//! [POS]: 动态模型定价服务，从配置文件加载定价数据并计算请求费用
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::models::RequestLog;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 全局定价服务实例
pub static PRICING_SERVICE: Lazy<PricingService> = Lazy::new(PricingService::init);

/// 模型定价信息
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModelPricing {
    /// 输入 token 价格 (per token)
    #[serde(default)]
    pub input_cost_per_token: f64,

    /// 输出 token 价格 (per token)
    #[serde(default)]
    pub output_cost_per_token: f64,

    /// 缓存创建价格 (per token)
    /// 如未提供，使用 input_cost_per_token * 1.25 作为默认值
    #[serde(default)]
    pub cache_creation_input_token_cost: Option<f64>,

    /// 缓存读取价格 (per token)
    /// 如未提供，使用 input_cost_per_token * 0.1 作为默认值
    #[serde(default)]
    pub cache_read_input_token_cost: Option<f64>,

    /// 超过 200k token 后的输入价格 (预留，暂未实现)
    #[serde(default)]
    pub input_cost_per_token_above_200k_tokens: Option<f64>,

    /// 超过 200k token 后的输出价格 (预留，暂未实现)
    #[serde(default)]
    pub output_cost_per_token_above_200k_tokens: Option<f64>,
}

/// 定价配置文件格式
pub type PricingData = HashMap<String, ModelPricing>;

/// 定价服务
pub struct PricingService {
    data: RwLock<PricingData>,
}

impl PricingService {
    /// 初始化定价服务
    /// 加载优先级：用户配置 > 内置配置 > 硬编码默认值
    pub fn init() -> Self {
        let data = match Self::load_user_config() {
            Ok(d) => {
                info!("已加载用户定价配置，共 {} 个模型", d.len());
                d
            }
            Err(e) => {
                debug!("用户定价配置加载失败: {}，尝试内置配置", e);
                match Self::load_builtin_config() {
                    Ok(d) => {
                        info!("已加载内置定价配置，共 {} 个模型", d.len());
                        d
                    }
                    Err(e) => {
                        error!("内置定价配置加载失败: {}，使用硬编码默认值", e);
                        Self::default_pricing()
                    }
                }
            }
        };

        Self {
            data: RwLock::new(data),
        }
    }

    /// 从用户配置目录加载定价配置
    /// ~/.iswitch/model-pricing.json
    fn load_user_config() -> Result<PricingData, String> {
        let home = dirs::home_dir().ok_or("无法获取用户目录")?;
        let path = home.join(".iswitch").join("model-pricing.json");

        if !path.exists() {
            return Err("用户配置文件不存在".to_string());
        }

        let content = fs::read_to_string(&path).map_err(|e| format!("读取用户配置失败: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("解析用户配置失败: {}", e))
    }

    /// 从内置资源加载定价配置
    /// 运行时从 resources/model-pricing.json 加载
    fn load_builtin_config() -> Result<PricingData, String> {
        // 尝试多种可能的路径
        let possible_paths = vec![
            // 开发模式：相对于 src-tauri 目录
            PathBuf::from("resources/model-pricing.json"),
            // 打包模式：相对于可执行文件
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("resources/model-pricing.json")))
                .unwrap_or_default(),
            // macOS 打包模式：Resources 目录
            std::env::current_exe()
                .ok()
                .and_then(|p| {
                    p.parent()
                        .and_then(|p| p.parent())
                        .map(|p| p.join("Resources/resources/model-pricing.json"))
                })
                .unwrap_or_default(),
        ];

        for path in possible_paths {
            if path.exists() {
                debug!("从 {:?} 加载内置定价配置", path);
                let content =
                    fs::read_to_string(&path).map_err(|e| format!("读取内置配置失败: {}", e))?;
                return serde_json::from_str(&content)
                    .map_err(|e| format!("解析内置配置失败: {}", e));
            }
        }

        Err("内置配置文件不存在".to_string())
    }

    /// 硬编码的默认定价 (作为最后的回退)
    fn default_pricing() -> PricingData {
        let mut m = HashMap::new();

        // Claude Sonnet 4 ($/token)
        m.insert(
            "claude-sonnet-4-20250514".to_string(),
            ModelPricing {
                input_cost_per_token: 3e-06,
                output_cost_per_token: 1.5e-05,
                cache_creation_input_token_cost: Some(3.75e-06),
                cache_read_input_token_cost: Some(3e-07),
                ..Default::default()
            },
        );

        // Claude 3.5 Sonnet ($/token)
        m.insert(
            "claude-3-5-sonnet-20241022".to_string(),
            ModelPricing {
                input_cost_per_token: 3e-06,
                output_cost_per_token: 1.5e-05,
                cache_creation_input_token_cost: Some(3.75e-06),
                cache_read_input_token_cost: Some(3e-07),
                ..Default::default()
            },
        );

        // Claude 3.5 Haiku ($/token)
        m.insert(
            "claude-3-5-haiku-20241022".to_string(),
            ModelPricing {
                input_cost_per_token: 8e-07,
                output_cost_per_token: 4e-06,
                cache_creation_input_token_cost: Some(1e-06),
                cache_read_input_token_cost: Some(8e-08),
                ..Default::default()
            },
        );

        // Claude 3 Opus ($/token)
        m.insert(
            "claude-3-opus-20240229".to_string(),
            ModelPricing {
                input_cost_per_token: 1.5e-05,
                output_cost_per_token: 7.5e-05,
                cache_creation_input_token_cost: Some(1.875e-05),
                cache_read_input_token_cost: Some(1.5e-06),
                ..Default::default()
            },
        );

        // GPT-4o ($/token)
        m.insert(
            "gpt-4o".to_string(),
            ModelPricing {
                input_cost_per_token: 2.5e-06,
                output_cost_per_token: 1e-05,
                cache_read_input_token_cost: Some(1.25e-06),
                ..Default::default()
            },
        );

        // GPT-4o-mini ($/token)
        m.insert(
            "gpt-4o-mini".to_string(),
            ModelPricing {
                input_cost_per_token: 1.5e-07,
                output_cost_per_token: 6e-07,
                cache_read_input_token_cost: Some(7.5e-08),
                ..Default::default()
            },
        );

        warn!("使用硬编码默认定价，仅包含 {} 个模型", m.len());
        m
    }

    /// 根据模型名称获取定价信息
    ///
    /// 匹配策略 (按优先级):
    /// 1. 精确匹配
    /// 2. 移除 provider 前缀后匹配 (如 anthropic/claude-sonnet-4 -> claude-sonnet-4)
    /// 3. 模糊匹配 (包含关系)
    pub fn get_pricing(&self, model: &str) -> Option<ModelPricing> {
        let data = self.data.read().unwrap();

        // 1. 精确匹配
        if let Some(p) = data.get(model) {
            return Some(p.clone());
        }

        // 2. 移除 provider 前缀后匹配
        let normalized = model.split('/').last().unwrap_or(model);
        if normalized != model {
            if let Some(p) = data.get(normalized) {
                return Some(p.clone());
            }
        }

        // 3. 模糊匹配 - 查找模型名中包含配置 key 的情况
        // 优先匹配更长的 key (更具体)
        let mut best_match: Option<(&str, &ModelPricing)> = None;
        for (key, pricing) in data.iter() {
            if model.contains(key) {
                if best_match.is_none() || key.len() > best_match.as_ref().unwrap().0.len() {
                    best_match = Some((key, pricing));
                }
            }
        }
        if let Some((_, pricing)) = best_match {
            return Some(pricing.clone());
        }

        // 4. 模糊匹配 - 查找配置 key 包含模型名的情况
        for (key, pricing) in data.iter() {
            if key.contains(model) {
                return Some(pricing.clone());
            }
        }

        None
    }

    /// 检查定价服务是否已加载数据
    pub fn is_loaded(&self) -> bool {
        !self.data.read().unwrap().is_empty()
    }

    /// 获取已加载的模型数量
    pub fn model_count(&self) -> usize {
        self.data.read().unwrap().len()
    }
}

/// 计算请求费用
///
/// 根据模型定价计算输入、输出、缓存创建和缓存读取的费用
pub fn calculate_cost(log: &mut RequestLog) {
    if let Some(pricing) = PRICING_SERVICE.get_pricing(&log.model) {
        log.has_pricing = true;

        // 缓存创建价格，默认为输入价格的 1.25 倍
        let cache_create_price = pricing
            .cache_creation_input_token_cost
            .unwrap_or(pricing.input_cost_per_token * 1.25);

        // 缓存读取价格，默认为输入价格的 0.1 倍
        let cache_read_price = pricing
            .cache_read_input_token_cost
            .unwrap_or(pricing.input_cost_per_token * 0.1);

        // 基础输入费用 (不含缓存读取的 tokens)
        // 注意: input_tokens 已经包含了 cache_read_tokens，需要减去
        let base_input_tokens = (log.input_tokens - log.cache_read_tokens).max(0) as f64;
        log.input_cost = base_input_tokens * pricing.input_cost_per_token;

        // 缓存创建费用
        log.cache_create_cost = log.cache_create_tokens as f64 * cache_create_price;

        // 缓存读取费用
        log.cache_read_cost = log.cache_read_tokens as f64 * cache_read_price;

        // 输出费用
        log.output_cost = log.output_tokens as f64 * pricing.output_cost_per_token;

        // 总费用
        log.total_cost =
            log.input_cost + log.output_cost + log.cache_create_cost + log.cache_read_cost;
    }

    // 确保总费用大于 0 以便热力图显示颜色
    if log.total_cost == 0.0 && (log.input_tokens > 0 || log.output_tokens > 0) {
        log.total_cost = 0.0001;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pricing_data() -> PricingData {
        let mut m = HashMap::new();
        m.insert(
            "claude-sonnet-4-20250514".to_string(),
            ModelPricing {
                input_cost_per_token: 3e-06,
                output_cost_per_token: 1.5e-05,
                cache_creation_input_token_cost: Some(3.75e-06),
                cache_read_input_token_cost: Some(3e-07),
                ..Default::default()
            },
        );
        m.insert(
            "gpt-4o".to_string(),
            ModelPricing {
                input_cost_per_token: 2.5e-06,
                output_cost_per_token: 1e-05,
                cache_read_input_token_cost: Some(1.25e-06),
                ..Default::default()
            },
        );
        m
    }

    #[test]
    fn test_exact_match() {
        let service = PricingService {
            data: RwLock::new(create_test_pricing_data()),
        };

        let pricing = service.get_pricing("claude-sonnet-4-20250514");
        assert!(pricing.is_some());
        let p = pricing.unwrap();
        assert!((p.input_cost_per_token - 3e-06).abs() < 1e-10);
        assert!((p.output_cost_per_token - 1.5e-05).abs() < 1e-10);
    }

    #[test]
    fn test_normalized_match() {
        let service = PricingService {
            data: RwLock::new(create_test_pricing_data()),
        };

        // 带 provider 前缀的模型名
        let pricing = service.get_pricing("anthropic/claude-sonnet-4-20250514");
        assert!(pricing.is_some());
    }

    #[test]
    fn test_fuzzy_match() {
        let service = PricingService {
            data: RwLock::new(create_test_pricing_data()),
        };

        // 模型名包含配置 key
        let pricing = service.get_pricing("claude-sonnet-4-20250514-v1");
        assert!(pricing.is_some());
    }

    #[test]
    fn test_unknown_model() {
        let service = PricingService {
            data: RwLock::new(create_test_pricing_data()),
        };

        let pricing = service.get_pricing("unknown-model");
        assert!(pricing.is_none());
    }

    #[test]
    fn test_calculate_cost_claude_sonnet_4() {
        // 临时初始化内置数据用于测试
        let mut log = RequestLog::default();
        log.model = "claude-sonnet-4-20250514".to_string();
        log.input_tokens = 1_000_000;
        log.output_tokens = 100_000;
        log.cache_create_tokens = 50_000;
        log.cache_read_tokens = 200_000;

        // 手动计算预期值
        // input_cost = (1M - 200k) * 3e-06 = 800k * 3e-06 = $2.40
        // cache_create_cost = 50k * 3.75e-06 = $0.1875
        // cache_read_cost = 200k * 3e-07 = $0.06
        // output_cost = 100k * 1.5e-05 = $1.50
        // total = 2.40 + 0.1875 + 0.06 + 1.50 = $4.1475

        calculate_cost(&mut log);

        assert!(log.has_pricing);
        assert!((log.input_cost - 2.40).abs() < 0.01);
        assert!((log.cache_create_cost - 0.1875).abs() < 0.01);
        assert!((log.cache_read_cost - 0.06).abs() < 0.01);
        assert!((log.output_cost - 1.50).abs() < 0.01);
        assert!((log.total_cost - 4.1475).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cost_gpt4o_no_cache_create() {
        let mut log = RequestLog::default();
        log.model = "gpt-4o".to_string();
        log.input_tokens = 1_000_000;
        log.output_tokens = 100_000;
        log.cache_create_tokens = 0;
        log.cache_read_tokens = 0;

        // input_cost = 1M * 2.5e-06 = $2.50
        // output_cost = 100k * 1e-05 = $1.00
        // total = $3.50

        calculate_cost(&mut log);

        assert!(log.has_pricing);
        assert!((log.input_cost - 2.50).abs() < 0.01);
        assert!((log.output_cost - 1.00).abs() < 0.01);
        assert!((log.total_cost - 3.50).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cost_unknown_model() {
        let mut log = RequestLog::default();
        log.model = "unknown-model".to_string();
        log.input_tokens = 1000;
        log.output_tokens = 500;

        calculate_cost(&mut log);

        // 未知模型费用应为最小值
        assert!(!log.has_pricing);
        assert!((log.total_cost - 0.0001).abs() < 1e-10);
    }

    #[test]
    fn test_default_pricing() {
        let pricing = PricingService::default_pricing();
        assert!(!pricing.is_empty());
        assert!(pricing.contains_key("claude-sonnet-4-20250514"));
        assert!(pricing.contains_key("gpt-4o"));
    }

    // ===== 边界条件测试 =====

    /// 测试零 tokens 不产生费用
    #[test]
    fn test_calculate_cost_zero_tokens() {
        let mut log = RequestLog::default();
        log.model = "claude-sonnet-4-20250514".to_string();
        log.input_tokens = 0;
        log.output_tokens = 0;
        log.cache_create_tokens = 0;
        log.cache_read_tokens = 0;

        calculate_cost(&mut log);

        assert!(log.has_pricing);
        assert_eq!(log.total_cost, 0.0);
    }

    /// 测试只有 input tokens
    #[test]
    fn test_calculate_cost_input_only() {
        let mut log = RequestLog::default();
        log.model = "claude-sonnet-4-20250514".to_string();
        log.input_tokens = 1_000_000;
        log.output_tokens = 0;

        calculate_cost(&mut log);

        assert!(log.has_pricing);
        // 1M * 3e-06 = $3.00
        assert!((log.input_cost - 3.0).abs() < 0.01);
        assert_eq!(log.output_cost, 0.0);
    }

    /// 测试只有 output tokens
    #[test]
    fn test_calculate_cost_output_only() {
        let mut log = RequestLog::default();
        log.model = "claude-sonnet-4-20250514".to_string();
        log.input_tokens = 0;
        log.output_tokens = 100_000;

        calculate_cost(&mut log);

        assert!(log.has_pricing);
        assert_eq!(log.input_cost, 0.0);
        // 100k * 1.5e-05 = $1.50
        assert!((log.output_cost - 1.50).abs() < 0.01);
    }

    /// 测试 cache_read_tokens 超过 input_tokens 的情况
    /// 这种情况在实际中不应该发生，但代码应该能处理
    #[test]
    fn test_calculate_cost_cache_read_exceeds_input() {
        let mut log = RequestLog::default();
        log.model = "claude-sonnet-4-20250514".to_string();
        log.input_tokens = 100;
        log.cache_read_tokens = 200; // 超过 input_tokens

        calculate_cost(&mut log);

        assert!(log.has_pricing);
        // base_input_tokens = (100 - 200).max(0) = 0
        assert_eq!(log.input_cost, 0.0);
        // cache_read_cost = 200 * 3e-07 = 0.00006
        assert!(log.cache_read_cost > 0.0);
    }

    /// 测试默认缓存价格计算（当配置中没有指定缓存价格时）
    #[test]
    fn test_calculate_cost_default_cache_prices() {
        // 创建一个没有缓存价格配置的模型
        let mut data = HashMap::new();
        data.insert(
            "test-model".to_string(),
            ModelPricing {
                input_cost_per_token: 1e-06, // $1 per 1M
                output_cost_per_token: 2e-06,
                cache_creation_input_token_cost: None, // 未配置
                cache_read_input_token_cost: None,     // 未配置
                ..Default::default()
            },
        );

        let service = PricingService {
            data: RwLock::new(data),
        };

        let pricing = service.get_pricing("test-model").unwrap();

        // 默认缓存创建价格 = input * 1.25
        let default_cache_create = pricing
            .cache_creation_input_token_cost
            .unwrap_or(pricing.input_cost_per_token * 1.25);
        assert!((default_cache_create - 1.25e-06).abs() < 1e-12);

        // 默认缓存读取价格 = input * 0.1
        let default_cache_read = pricing
            .cache_read_input_token_cost
            .unwrap_or(pricing.input_cost_per_token * 0.1);
        assert!((default_cache_read - 1e-07).abs() < 1e-12);
    }

    /// 测试服务状态
    #[test]
    fn test_service_is_loaded() {
        let service = PricingService {
            data: RwLock::new(create_test_pricing_data()),
        };

        assert!(service.is_loaded());
        assert_eq!(service.model_count(), 2);
    }

    /// 测试空服务
    #[test]
    fn test_service_empty() {
        let service = PricingService {
            data: RwLock::new(HashMap::new()),
        };

        assert!(!service.is_loaded());
        assert_eq!(service.model_count(), 0);
    }

    /// 测试多层 provider 前缀
    #[test]
    fn test_multi_level_provider_prefix() {
        let service = PricingService {
            data: RwLock::new(create_test_pricing_data()),
        };

        // 多层前缀，应该取最后一部分
        let pricing = service.get_pricing("vendor/anthropic/claude-sonnet-4-20250514");
        assert!(pricing.is_some());
    }

    /// 测试配置 key 包含模型名的反向模糊匹配
    #[test]
    fn test_reverse_fuzzy_match() {
        let mut data = HashMap::new();
        data.insert(
            "claude-sonnet".to_string(), // key 较短
            ModelPricing {
                input_cost_per_token: 1e-06,
                output_cost_per_token: 2e-06,
                ..Default::default()
            },
        );

        let service = PricingService {
            data: RwLock::new(data),
        };

        // 模型名包含在 key 中
        let pricing = service.get_pricing("claude");
        assert!(pricing.is_some());
    }

    /// 测试最长匹配优先
    #[test]
    fn test_longest_match_priority() {
        let mut data = HashMap::new();
        data.insert(
            "claude".to_string(),
            ModelPricing {
                input_cost_per_token: 1e-06,
                ..Default::default()
            },
        );
        data.insert(
            "claude-sonnet".to_string(),
            ModelPricing {
                input_cost_per_token: 2e-06,
                ..Default::default()
            },
        );

        let service = PricingService {
            data: RwLock::new(data),
        };

        // 应该匹配更长的 "claude-sonnet"
        let pricing = service.get_pricing("claude-sonnet-4-20250514");
        assert!(pricing.is_some());
        assert!((pricing.unwrap().input_cost_per_token - 2e-06).abs() < 1e-12);
    }
}
