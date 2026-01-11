//! [INPUT]:
//!   source: ../../../../code-switch/services/providerservice.go ([POS]: 原 Go Provider 数据模型)
//!   source: ../../../openspec/changes/migrate-to-tauri-stack/design.md ([POS]: Rust 数据结构规范)
//!
//! [OUTPUT]:
//!   - Provider 结构体
//!   - ProviderKind 枚举
//!   - ProviderEnvelope 封装结构
//!
//! [POS]: 供应商数据模型定义，与 Go 版本 JSON 格式完全兼容
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 供应商类型枚举
///
/// 对应 Claude Code 和 Codex 两种 AI 平台
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    /// Claude Code 平台
    #[default]
    Claude,
    /// Codex 平台
    Codex,
}

impl ProviderKind {
    /// 从字符串解析 ProviderKind
    ///
    /// 支持多种变体形式（大小写不敏感）
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" | "claude-code" | "claude_code" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }

    /// 获取对应的配置文件名
    pub fn config_filename(&self) -> &'static str {
        match self {
            Self::Claude => "claude-code.json",
            Self::Codex => "codex.json",
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Codex => write!(f, "codex"),
        }
    }
}

/// 供应商配置
///
/// 完全对应 Go 版本的 Provider 结构体，保持 JSON 序列化兼容性
///
/// # JSON 字段映射
/// - `apiUrl` -> `api_url` (驼峰命名)
/// - `apiKey` -> `api_key` (驼峰命名)  
/// - `supportedModels` -> `supported_models` (驼峰命名)
/// - `model_mapping` -> `model_mapping` (下划线命名)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Provider {
    /// 唯一标识符
    pub id: i64,

    /// 供应商名称（不可修改）
    pub name: String,

    /// API 基础 URL
    #[serde(rename = "apiUrl")]
    pub api_url: String,

    /// API 密钥
    #[serde(rename = "apiKey")]
    pub api_key: String,

    /// 官方网站
    #[serde(rename = "officialSite", default)]
    pub site: String,

    /// 图标名称
    #[serde(default)]
    pub icon: String,

    /// 主题色调
    #[serde(default)]
    pub tint: String,

    /// 强调色
    #[serde(default)]
    pub accent: String,

    /// 是否启用
    #[serde(default)]
    pub enabled: bool,

    /// 模型白名单 - Provider 原生支持的模型名
    ///
    /// 使用 HashMap 实现 O(1) 查找，与 Go 版本的 map[string]bool 兼容
    #[serde(
        rename = "supportedModels",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub supported_models: Option<HashMap<String, bool>>,

    /// 模型映射 - 外部模型名 -> Provider 内部模型名
    ///
    /// 支持精确匹配和通配符（如 "claude-*" -> "anthropic/claude-*"）
    #[serde(
        rename = "modelMapping",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub model_mapping: Option<HashMap<String, String>>,

    /// 优先级分组 - 数字越小优先级越高（1-10，默认 1）
    #[serde(default, skip_serializing_if = "is_zero")]
    pub level: i32,
}

/// 供应商配置文件封装
///
/// 对应 Go 版本的 providerEnvelope，用于 JSON 文件读写
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderEnvelope {
    pub providers: Vec<Provider>,
}

impl Provider {
    /// 检查 provider 是否支持指定的模型
    ///
    /// 支持条件：
    /// 1. 模型在 SupportedModels 中（精确或通配符匹配）
    /// 2. 模型在 ModelMapping 的 key 中（精确或通配符匹配）
    /// 3. 未配置白名单和映射时，假设支持所有模型（向后兼容）
    pub fn is_model_supported(&self, model_name: &str) -> bool {
        let has_supported = self
            .supported_models
            .as_ref()
            .map(|m| !m.is_empty())
            .unwrap_or(false);
        let has_mapping = self
            .model_mapping
            .as_ref()
            .map(|m| !m.is_empty())
            .unwrap_or(false);

        // 向后兼容：如果未配置白名单和映射，假设支持所有模型
        if !has_supported && !has_mapping {
            return true;
        }

        // 场景 A：Provider 原生支持该模型
        if let Some(ref models) = self.supported_models {
            // 精确匹配
            if models.get(model_name).copied().unwrap_or(false) {
                return true;
            }
            // 通配符匹配
            for supported_model in models.keys() {
                if match_wildcard(supported_model, model_name) {
                    return true;
                }
            }
        }

        // 场景 B：Provider 通过映射支持该模型
        if let Some(ref mapping) = self.model_mapping {
            // 精确匹配
            if mapping.contains_key(model_name) {
                return true;
            }
            // 通配符匹配
            for pattern in mapping.keys() {
                if match_wildcard(pattern, model_name) {
                    return true;
                }
            }
        }

        // 场景 C：不支持
        false
    }

    /// 获取实际应该使用的模型名
    ///
    /// 如果存在映射（精确或通配符），返回映射后的模型名；否则返回原模型名
    pub fn get_effective_model(&self, requested_model: &str) -> String {
        let Some(ref mapping) = self.model_mapping else {
            return requested_model.to_string();
        };

        if mapping.is_empty() {
            return requested_model.to_string();
        }

        // 优先查找精确映射
        if let Some(mapped) = mapping.get(requested_model) {
            return mapped.clone();
        }

        // 查找通配符映射
        for (pattern, replacement) in mapping {
            if match_wildcard(pattern, requested_model) {
                return apply_wildcard_mapping(pattern, replacement, requested_model);
            }
        }

        // 无映射，返回原模型名
        requested_model.to_string()
    }

    /// 验证 provider 的模型配置
    ///
    /// 返回验证错误列表（空则表示验证通过）
    pub fn validate_configuration(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // 规则 1：ModelMapping 的 value 必须在 SupportedModels 中
        if let (Some(ref mapping), Some(ref supported)) =
            (&self.model_mapping, &self.supported_models)
        {
            for (external_model, internal_model) in mapping {
                // 检查是否为通配符映射
                if internal_model.contains('*') {
                    // 通配符映射暂不验证（需要具体请求才能展开）
                    continue;
                }

                // 精确映射需要验证
                let mut is_supported = false;

                if supported.get(internal_model).copied().unwrap_or(false) {
                    is_supported = true;
                } else {
                    // 检查通配符白名单
                    for supported_pattern in supported.keys() {
                        if match_wildcard(supported_pattern, internal_model) {
                            is_supported = true;
                            break;
                        }
                    }
                }

                if !is_supported {
                    errors.push(format!(
                        "模型映射无效：'{}' -> '{}'，目标模型 '{}' 不在 supportedModels 中",
                        external_model, internal_model, internal_model
                    ));
                }
            }
        }

        // 规则 2：如果配置了 ModelMapping 但未配置 SupportedModels，给出警告
        let has_mapping = self
            .model_mapping
            .as_ref()
            .map(|m| !m.is_empty())
            .unwrap_or(false);
        let has_supported = self
            .supported_models
            .as_ref()
            .map(|m| !m.is_empty())
            .unwrap_or(false);

        if has_mapping && !has_supported {
            errors.push(
                "警告：配置了 modelMapping 但未配置 supportedModels，映射的目标模型无法验证"
                    .to_string(),
            );
        }

        // 规则 3：检测自映射（通常无意义，但不是错误）
        if let Some(ref mapping) = self.model_mapping {
            for (external, internal) in mapping {
                if external == internal {
                    errors.push(format!(
                        "警告：模型 '{}' 映射到自身，这通常无意义",
                        external
                    ));
                }
            }
        }

        errors
    }

    /// 检查 provider 是否有效（可用于请求转发）
    pub fn is_valid(&self) -> bool {
        self.enabled && !self.api_url.is_empty() && !self.api_key.is_empty()
    }
}

/// 通配符匹配函数
///
/// 支持 `*` 通配符，如 "claude-*" 匹配 "claude-sonnet-4"
pub fn match_wildcard(pattern: &str, text: &str) -> bool {
    // 如果没有通配符，使用精确匹配
    if !pattern.contains('*') {
        return pattern == text;
    }

    // 简化实现：只支持单个 * 通配符
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 2 {
        let (prefix, suffix) = (parts[0], parts[1]);
        return text.starts_with(prefix) && text.ends_with(suffix);
    }

    // 多个 * 的情况（更复杂，暂不支持）
    false
}

/// 应用通配符映射
///
/// 将 pattern 中的 `*` 匹配部分替换到 replacement 的 `*` 位置
///
/// # 示例
/// ```ignore
/// use iswitch_lib::models::provider::apply_wildcard_mapping;
/// let result = apply_wildcard_mapping("claude-*", "anthropic/claude-*", "claude-sonnet-4");
/// assert_eq!(result, "anthropic/claude-sonnet-4");
/// ```
pub fn apply_wildcard_mapping(pattern: &str, replacement: &str, input: &str) -> String {
    // 如果 pattern 或 replacement 没有通配符，直接返回 replacement
    if !pattern.contains('*') || !replacement.contains('*') {
        return replacement.to_string();
    }

    // 提取通配符匹配的部分
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() != 2 {
        return replacement.to_string(); // 不支持多个通配符
    }

    let (prefix, suffix) = (parts[0], parts[1]);

    // 验证 input 确实匹配 pattern
    if !input.starts_with(prefix) || !input.ends_with(suffix) {
        return replacement.to_string();
    }

    // 提取中间部分
    let wildcard_part = &input[prefix.len()..input.len() - suffix.len()];

    // 替换 replacement 中的 *
    replacement.replacen('*', wildcard_part, 1)
}

/// 辅助函数：检查值是否为零（用于 serde skip_serializing_if）
fn is_zero(value: &i32) -> bool {
    *value == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_kind_from_str() {
        assert_eq!(ProviderKind::from_str("claude"), Some(ProviderKind::Claude));
        assert_eq!(
            ProviderKind::from_str("claude-code"),
            Some(ProviderKind::Claude)
        );
        assert_eq!(
            ProviderKind::from_str("CLAUDE_CODE"),
            Some(ProviderKind::Claude)
        );
        assert_eq!(ProviderKind::from_str("codex"), Some(ProviderKind::Codex));
        assert_eq!(ProviderKind::from_str("unknown"), None);
    }

    #[test]
    fn test_match_wildcard() {
        assert!(match_wildcard("claude-*", "claude-sonnet-4"));
        assert!(match_wildcard("claude-*", "claude-3"));
        assert!(!match_wildcard("claude-*", "gpt-4"));
        assert!(match_wildcard("*-mini", "gpt-4o-mini"));
        assert!(match_wildcard("gpt-*-turbo", "gpt-4-turbo"));
    }

    #[test]
    fn test_apply_wildcard_mapping() {
        assert_eq!(
            apply_wildcard_mapping("claude-*", "anthropic/claude-*", "claude-sonnet-4"),
            "anthropic/claude-sonnet-4"
        );
        assert_eq!(
            apply_wildcard_mapping("gpt-*", "openai/gpt-*", "gpt-4o"),
            "openai/gpt-4o"
        );
    }

    #[test]
    fn test_provider_is_model_supported() {
        let mut provider = Provider::default();

        // 无配置时，假设支持所有模型
        assert!(provider.is_model_supported("any-model"));

        // 配置 supported_models
        let mut supported = HashMap::new();
        supported.insert("claude-sonnet-4".to_string(), true);
        supported.insert("claude-*".to_string(), true);
        provider.supported_models = Some(supported);

        assert!(provider.is_model_supported("claude-sonnet-4"));
        assert!(provider.is_model_supported("claude-3"));
        assert!(!provider.is_model_supported("gpt-4"));
    }

    #[test]
    fn test_provider_get_effective_model() {
        let mut provider = Provider::default();

        // 无映射时返回原模型
        assert_eq!(provider.get_effective_model("gpt-4"), "gpt-4");

        // 配置映射
        let mut mapping = HashMap::new();
        mapping.insert("claude-*".to_string(), "anthropic/claude-*".to_string());
        mapping.insert("gpt-4".to_string(), "openai/gpt-4".to_string());
        provider.model_mapping = Some(mapping);

        assert_eq!(provider.get_effective_model("gpt-4"), "openai/gpt-4");
        assert_eq!(
            provider.get_effective_model("claude-sonnet-4"),
            "anthropic/claude-sonnet-4"
        );
        assert_eq!(provider.get_effective_model("unknown"), "unknown");
    }

    #[test]
    fn test_provider_json_compat() {
        // 测试与 Go 版本 JSON 格式的兼容性
        let json = r#"{
            "id": 1,
            "name": "Test Provider",
            "apiUrl": "https://api.example.com",
            "apiKey": "sk-test",
            "officialSite": "https://example.com",
            "enabled": true,
            "supportedModels": {"claude-sonnet-4": true},
            "modelMapping": {"gpt-4": "openai/gpt-4"}
        }"#;

        let provider: Provider = serde_json::from_str(json).unwrap();
        assert_eq!(provider.id, 1);
        assert_eq!(provider.name, "Test Provider");
        assert_eq!(provider.api_url, "https://api.example.com");
        assert!(provider.enabled);

        // 验证可以序列化回 JSON
        let serialized = serde_json::to_string(&provider).unwrap();
        assert!(serialized.contains("apiUrl"));
        assert!(serialized.contains("apiKey"));
    }
}
