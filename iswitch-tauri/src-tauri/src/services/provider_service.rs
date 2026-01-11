//! [INPUT]:
//!   source: ../../../../code-switch/services/providerservice.go ([POS]: 原 Go 实现参考)
//!   source: ../models/provider.rs ([POS]: Provider 数据模型)
//!
//! [OUTPUT]:
//!   - ProviderService 结构体及其方法
//!   - load_providers(), save_providers() 公开 API
//!
//! [POS]: 供应商配置的加载、保存和验证逻辑
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::AppResult;
use crate::models::{Provider, ProviderEnvelope, ProviderKind};
use crate::utils::paths;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// 供应商服务
///
/// 管理 Claude Code 和 Codex 的 Provider 配置
pub struct ProviderService {
    /// 内部锁（用于并发安全）
    _lock: RwLock<()>,
    /// 配置根目录（可选，用于测试）
    root_path: Option<PathBuf>,
}

impl ProviderService {
    /// 创建新的 ProviderService 实例
    pub fn new() -> Self {
        Self {
            _lock: RwLock::new(()),
            root_path: None,
        }
    }

    /// 创建带有指定根目录的 ProviderService (仅用于测试)
    #[cfg(test)]
    pub fn with_root(path: PathBuf) -> Self {
        Self {
            _lock: RwLock::new(()),
            root_path: Some(path),
        }
    }

    /// 加载指定类型的 Provider 列表
    pub async fn load_providers(&self, kind: &str) -> AppResult<Vec<Provider>> {
        let path = self.provider_file_path(kind)?;

        // 如果文件不存在，返回空列表
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
            crate::error::AppError::ConfigRead {
                path: path.display().to_string(),
                source: e,
            }
        })?;

        if content.is_empty() {
            return Ok(Vec::new());
        }

        let envelope: ProviderEnvelope = serde_json::from_str(&content)?;
        Ok(envelope.providers)
    }

    /// 保存 Provider 列表
    pub async fn save_providers(&self, kind: &str, providers: Vec<Provider>) -> AppResult<()> {
        let path = self.provider_file_path(kind)?;

        // 确保目录存在
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                crate::error::AppError::DirCreate {
                    path: parent.display().to_string(),
                    source: e,
                }
            })?;
        }

        let envelope = ProviderEnvelope { providers };
        let content = serde_json::to_string_pretty(&envelope)?;

        // 原子写入：先写临时文件，再重命名
        let tmp_path = path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, &content).await.map_err(|e| {
            crate::error::AppError::ConfigWrite {
                path: tmp_path.display().to_string(),
                source: e,
            }
        })?;

        tokio::fs::rename(&tmp_path, &path).await.map_err(|e| {
            crate::error::AppError::ConfigWrite {
                path: path.display().to_string(),
                source: e,
            }
        })?;

        Ok(())
    }

    /// 获取 Provider 配置文件路径
    fn provider_file_path(&self, kind: &str) -> AppResult<PathBuf> {
        let provider_kind = ProviderKind::from_str(kind).ok_or_else(|| {
            crate::error::AppError::InvalidArgument(format!("未知的 provider 类型: {}", kind))
        })?;

        let dir = if let Some(ref p) = self.root_path {
            p.clone()
        } else {
            paths::iswitch_dir()
        };
        Ok(dir.join(provider_kind.config_filename()))
    }
}

impl Default for ProviderService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Provider;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_save_and_load_providers() {
        let dir = tempdir().unwrap();
        let service = ProviderService::with_root(dir.path().to_path_buf());

        let providers = vec![Provider {
            id: 1,
            name: "Test Claude".to_string(),
            api_url: "https://api.anthropic.com".to_string(),
            api_key: "sk-test".to_string(),
            ..Default::default()
        }];

        // 1. Save
        service
            .save_providers("claude", providers.clone())
            .await
            .expect("Save failed");

        // 2. Load
        let loaded = service.load_providers("claude").await.expect("Load failed");

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Test Claude");
        assert_eq!(loaded[0].api_key, "sk-test");
    }

    #[tokio::test]
    async fn test_load_non_existent() {
        let dir = tempdir().unwrap();
        let service = ProviderService::with_root(dir.path().to_path_buf());

        let loaded = service.load_providers("codex").await.expect("Load failed");
        assert!(loaded.is_empty());
    }

    // ===== 边界条件测试 =====

    /// 测试保存空列表
    #[tokio::test]
    async fn test_save_empty_list() {
        let dir = tempdir().unwrap();
        let service = ProviderService::with_root(dir.path().to_path_buf());

        // 保存空列表
        service
            .save_providers("claude", vec![])
            .await
            .expect("Save empty list failed");

        // 加载应该返回空列表
        let loaded = service.load_providers("claude").await.expect("Load failed");
        assert!(loaded.is_empty());
    }

    /// 测试多个 Provider 的保存与加载
    #[tokio::test]
    async fn test_multiple_providers() {
        let dir = tempdir().unwrap();
        let service = ProviderService::with_root(dir.path().to_path_buf());

        let providers = vec![
            Provider {
                id: 1,
                name: "Provider 1".to_string(),
                api_url: "https://api1.example.com".to_string(),
                api_key: "key1".to_string(),
                level: 1,
                ..Default::default()
            },
            Provider {
                id: 2,
                name: "Provider 2".to_string(),
                api_url: "https://api2.example.com".to_string(),
                api_key: "key2".to_string(),
                level: 2,
                ..Default::default()
            },
            Provider {
                id: 3,
                name: "Provider 3".to_string(),
                api_url: "https://api3.example.com".to_string(),
                api_key: "key3".to_string(),
                level: 3,
                ..Default::default()
            },
        ];

        service
            .save_providers("codex", providers.clone())
            .await
            .expect("Save failed");

        let loaded = service.load_providers("codex").await.expect("Load failed");

        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].name, "Provider 1");
        assert_eq!(loaded[1].name, "Provider 2");
        assert_eq!(loaded[2].name, "Provider 3");
    }

    /// 测试未知的 provider kind
    #[tokio::test]
    async fn test_invalid_kind() {
        let dir = tempdir().unwrap();
        let service = ProviderService::with_root(dir.path().to_path_buf());

        let result = service.load_providers("invalid_kind").await;
        assert!(result.is_err());

        let result = service.save_providers("unknown", vec![]).await;
        assert!(result.is_err());
    }

    /// 测试覆盖保存
    #[tokio::test]
    async fn test_overwrite_save() {
        let dir = tempdir().unwrap();
        let service = ProviderService::with_root(dir.path().to_path_buf());

        // 第一次保存
        let providers1 = vec![Provider {
            id: 1,
            name: "Original".to_string(),
            api_url: "https://original.com".to_string(),
            api_key: "key1".to_string(),
            ..Default::default()
        }];
        service
            .save_providers("claude", providers1)
            .await
            .expect("First save failed");

        // 第二次保存（覆盖）
        let providers2 = vec![Provider {
            id: 2,
            name: "Updated".to_string(),
            api_url: "https://updated.com".to_string(),
            api_key: "key2".to_string(),
            ..Default::default()
        }];
        service
            .save_providers("claude", providers2)
            .await
            .expect("Second save failed");

        // 加载应该是第二次的内容
        let loaded = service.load_providers("claude").await.expect("Load failed");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Updated");
    }

    /// 测试空文件处理
    #[tokio::test]
    async fn test_load_empty_file() {
        let dir = tempdir().unwrap();
        let service = ProviderService::with_root(dir.path().to_path_buf());

        // 手动创建空文件
        let path = dir.path().join("claude-providers.json");
        tokio::fs::write(&path, "")
            .await
            .expect("Write empty file failed");

        // 加载应该返回空列表
        let loaded = service.load_providers("claude").await.expect("Load failed");
        assert!(loaded.is_empty());
    }

    /// 测试 Claude 和 Codex 配置独立
    #[tokio::test]
    async fn test_claude_and_codex_independent() {
        let dir = tempdir().unwrap();
        let service = ProviderService::with_root(dir.path().to_path_buf());

        // 保存 Claude provider
        let claude_providers = vec![Provider {
            id: 1,
            name: "Claude Provider".to_string(),
            api_url: "https://api.anthropic.com".to_string(),
            api_key: "claude-key".to_string(),
            ..Default::default()
        }];
        service
            .save_providers("claude", claude_providers)
            .await
            .expect("Save claude failed");

        // 保存 Codex provider
        let codex_providers = vec![Provider {
            id: 2,
            name: "Codex Provider".to_string(),
            api_url: "https://api.openai.com".to_string(),
            api_key: "codex-key".to_string(),
            ..Default::default()
        }];
        service
            .save_providers("codex", codex_providers)
            .await
            .expect("Save codex failed");

        // 分别加载，验证独立
        let loaded_claude = service
            .load_providers("claude")
            .await
            .expect("Load claude failed");
        let loaded_codex = service
            .load_providers("codex")
            .await
            .expect("Load codex failed");

        assert_eq!(loaded_claude.len(), 1);
        assert_eq!(loaded_claude[0].name, "Claude Provider");

        assert_eq!(loaded_codex.len(), 1);
        assert_eq!(loaded_codex[0].name, "Codex Provider");
    }

    /// 测试 Provider 的所有字段持久化
    #[tokio::test]
    async fn test_all_fields_persisted() {
        let dir = tempdir().unwrap();
        let service = ProviderService::with_root(dir.path().to_path_buf());

        let mut model_mapping = std::collections::HashMap::new();
        model_mapping.insert("claude-*".to_string(), "custom-claude-*".to_string());

        let mut supported_models = std::collections::HashMap::new();
        supported_models.insert("model-a".to_string(), true);
        supported_models.insert("model-b".to_string(), true);

        let providers = vec![Provider {
            id: 42,
            name: "Full Provider".to_string(),
            api_url: "https://api.example.com".to_string(),
            api_key: "secret-key".to_string(),
            site: "https://example.com".to_string(),
            icon: "test-icon".to_string(),
            tint: "#ff0000".to_string(),
            accent: "#00ff00".to_string(),
            enabled: false,
            level: 10,
            supported_models: Some(supported_models.clone()),
            model_mapping: Some(model_mapping.clone()),
        }];

        service
            .save_providers("claude", providers)
            .await
            .expect("Save failed");

        let loaded = service.load_providers("claude").await.expect("Load failed");

        assert_eq!(loaded.len(), 1);
        let p = &loaded[0];
        assert_eq!(p.id, 42);
        assert_eq!(p.name, "Full Provider");
        assert_eq!(p.api_url, "https://api.example.com");
        assert_eq!(p.api_key, "secret-key");
        assert_eq!(p.site, "https://example.com");
        assert_eq!(p.icon, "test-icon");
        assert_eq!(p.tint, "#ff0000");
        assert_eq!(p.accent, "#00ff00");
        assert!(!p.enabled); // false
        assert_eq!(p.level, 10);
        assert_eq!(p.supported_models, Some(supported_models));
        assert_eq!(p.model_mapping, Some(model_mapping));
    }

    /// 测试 Default trait
    #[test]
    fn test_default() {
        let service = ProviderService::default();
        assert!(service.root_path.is_none());
    }
}
