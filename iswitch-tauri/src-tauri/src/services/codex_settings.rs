/**
[INPUT]:
  - source: ../../../../openspec/changes/relocate-controls-to-capsule/specs/capsule-nav/spec.md ([POS]: 胶囊导航/Codex 设置规范)
[OUTPUT]:
  - CodexSettingsService 结构体
  - proxy_status(), enable_proxy(), disable_proxy() API
[POS]: Codex 代理设置管理
[PROTOCOL]: FractalFlow v1.0 - 分形自洽
*/
use std::path::PathBuf;

use crate::error::{AppError, AppResult};
use crate::utils::paths::ensure_dir;
use crate::utils::security::secure_write;
use serde::{Deserialize, Serialize};
use toml::Table;
use tracing::info;

const CODEX_SETTINGS_DIR: &str = ".codex";
const CODEX_CONFIG_FILENAME: &str = "config.toml";
const CODEX_BACKUP_CONFIG_NAME: &str = "cc-studio.back.config.toml";
const CODEX_AUTH_FILENAME: &str = "auth.json";
const CODEX_BACKUP_AUTH_NAME: &str = "cc-studio.back.auth.json";
const CODEX_PREFERRED_AUTH: &str = "apikey";
const CODEX_DEFAULT_MODEL: &str = "gpt-5.1-codex";
const CODEX_PROVIDER_KEY: &str = "iswitch";
const CODEX_WIRE_API: &str = "responses";
const CODEX_TOKEN_VALUE: &str = "iswitch";

#[derive(Debug, Serialize, Deserialize)]
struct CodexConfig {
    preferred_auth_method: Option<String>,
    model: Option<String>,
    model_provider: Option<String>,
    model_providers: Option<Table>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CodexProvider {
    name: String,
    base_url: String,
    wire_api: String,
    requires_openai_auth: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthFile {
    #[serde(rename = "OPENAI_API_KEY")]
    openai_api_key: Option<String>,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
}

/// Codex 设置服务
pub struct CodexSettingsService {
    root: Option<PathBuf>,
}

impl CodexSettingsService {
    pub fn new() -> Self {
        Self { root: None }
    }

    #[cfg(test)]
    pub fn new_with_root(root: PathBuf) -> Self {
        Self { root: Some(root) }
    }

    fn paths(&self) -> AppResult<(PathBuf, PathBuf)> {
        let home = if let Some(ref r) = self.root {
            r.clone()
        } else {
            dirs::home_dir().ok_or_else(|| AppError::Internal("无法获取用户主目录".to_string()))?
        };
        let dir = home.join(CODEX_SETTINGS_DIR);
        Ok((
            dir.join(CODEX_CONFIG_FILENAME),
            dir.join(CODEX_BACKUP_CONFIG_NAME),
        ))
    }

    fn auth_paths(&self) -> AppResult<(PathBuf, PathBuf)> {
        let home = if let Some(ref r) = self.root {
            r.clone()
        } else {
            dirs::home_dir().ok_or_else(|| AppError::Internal("无法获取用户主目录".to_string()))?
        };
        let dir = home.join(CODEX_SETTINGS_DIR);
        Ok((
            dir.join(CODEX_AUTH_FILENAME),
            dir.join(CODEX_BACKUP_AUTH_NAME),
        ))
    }

    /// 检测代理启用状态
    pub async fn proxy_status(&self) -> AppResult<bool> {
        let (config_path, _) = self.paths()?;
        if !config_path.exists() {
            return Ok(false);
        }

        let content =
            tokio::fs::read_to_string(&config_path)
                .await
                .map_err(|e| AppError::ConfigRead {
                    path: config_path.to_string_lossy().to_string(),
                    source: e,
                })?;

        let config: CodexConfig = match toml::from_str(&content) {
            Ok(c) => c,
            Err(_) => return Ok(false),
        };

        if let Some(mp) = config.model_provider {
            if mp == CODEX_PROVIDER_KEY {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 启用代理
    pub async fn enable_proxy(&self, proxy_url: &str) -> AppResult<()> {
        let (config_path, backup_path) = self.paths()?;

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            ensure_dir(&parent.to_path_buf()).map_err(|e| AppError::DirCreate {
                path: parent.to_string_lossy().to_string(),
                source: e,
            })?;
        }

        // 1. Handle Config File
        // Backup
        if config_path.exists() {
            if !backup_path.exists() {
                tokio::fs::copy(&config_path, &backup_path)
                    .await
                    .map_err(|e| AppError::ConfigWrite {
                        path: backup_path.to_string_lossy().to_string(),
                        source: e,
                    })?;
                info!(path = %backup_path.display(), "已备份 Codex 配置文件");
            }
        }

        // Read existing or create new
        let mut config_table: Table = if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await.map_err(|e| {
                AppError::ConfigRead {
                    path: config_path.to_string_lossy().to_string(),
                    source: e,
                }
            })?;
            toml::from_str(&content).unwrap_or_else(|_| Table::new())
        } else {
            Table::new()
        };

        // Update fields
        config_table.insert(
            "preferred_auth_method".to_string(),
            toml::Value::String(CODEX_PREFERRED_AUTH.to_string()),
        );
        config_table.insert(
            "model".to_string(),
            toml::Value::String(CODEX_DEFAULT_MODEL.to_string()),
        );
        config_table.insert(
            "model_provider".to_string(),
            toml::Value::String(CODEX_PROVIDER_KEY.to_string()),
        );

        // Update model_providers table
        let provider = CodexProvider {
            name: CODEX_PROVIDER_KEY.to_string(),
            base_url: proxy_url.to_string(),
            wire_api: CODEX_WIRE_API.to_string(),
            requires_openai_auth: false,
        };

        // Convert CodexProvider to Table
        // Need a bit of workaround to convert struct to toml::Value::Table
        let provider_value = toml::Value::try_from(&provider).unwrap();

        let model_providers_entry = config_table
            .entry("model_providers")
            .or_insert(toml::Value::Table(Table::new()));

        if let toml::Value::Table(ref mut mps) = model_providers_entry {
            mps.insert(CODEX_PROVIDER_KEY.to_string(), provider_value);
        }

        let new_content = toml::to_string_pretty(&config_table)?;
        secure_write(&config_path, new_content.as_bytes())?;
        info!("已更新 Codex 配置文件");

        // 2. Handle Auth File
        self.enable_auth_proxy().await?;

        info!("Codex 代理已启用");
        Ok(())
    }

    /// 禁用代理
    pub async fn disable_proxy(&self) -> AppResult<()> {
        let (config_path, _) = self.paths()?;

        if !config_path.exists() {
            return Ok(());
        }

        let content =
            tokio::fs::read_to_string(&config_path)
                .await
                .map_err(|e| AppError::ConfigRead {
                    path: config_path.to_string_lossy().to_string(),
                    source: e,
                })?;

        let mut config_table: Table = toml::from_str(&content).unwrap_or_else(|_| Table::new());

        // Remove proxy-specific fields
        config_table.remove("model_provider");

        // Remove specific provider from model_providers
        if let Some(model_providers) = config_table.get_mut("model_providers") {
            if let Some(table) = model_providers.as_table_mut() {
                table.remove(CODEX_PROVIDER_KEY);
            }
        }

        // Note: we generally want to leave preferred_auth_method and model if they were set by us,
        // but it's hard to know if user set them.
        // For safer side, let's keep them or reset to default if we really want strict clean up.
        // But the critical part is removing model_provider.
        // Let's remove them if they match our defaults to be clean.

        if let Some(val) = config_table.get("preferred_auth_method") {
            if val.as_str() == Some(CODEX_PREFERRED_AUTH) {
                config_table.remove("preferred_auth_method");
            }
        }
        if let Some(val) = config_table.get("model") {
            if val.as_str() == Some(CODEX_DEFAULT_MODEL) {
                config_table.remove("model");
            }
        }

        let new_content = toml::to_string_pretty(&config_table)?;
        secure_write(&config_path, new_content.as_bytes())?;
        info!("已更新 Codex 配置文件（禁用代理）");

        // Disable Auth
        self.disable_auth_proxy().await?;

        Ok(())
    }

    async fn enable_auth_proxy(&self) -> AppResult<()> {
        let (auth_path, backup_path) = self.auth_paths()?;
        if let Some(parent) = auth_path.parent() {
            ensure_dir(&parent.to_path_buf()).map_err(|e| AppError::DirCreate {
                path: parent.to_string_lossy().to_string(),
                source: e,
            })?;
        }

        // Backup existing
        if auth_path.exists() {
            if !backup_path.exists() {
                if let Err(e) = tokio::fs::copy(&auth_path, &backup_path).await {
                    info!(error = %e, "备份 Codex Auth 文件失败");
                } else {
                    info!(path = %backup_path.display(), "已备份 Codex Auth 文件");
                }
            }
        }

        // Read or Create
        let mut auth_file = if auth_path.exists() {
            let content =
                tokio::fs::read_to_string(&auth_path)
                    .await
                    .map_err(|e| AppError::ConfigRead {
                        path: auth_path.to_string_lossy().to_string(),
                        source: e,
                    })?;
            serde_json::from_str::<AuthFile>(&content).unwrap_or_else(|_| AuthFile {
                openai_api_key: None,
                other: serde_json::Map::new(),
            })
        } else {
            AuthFile {
                openai_api_key: None,
                other: serde_json::Map::new(),
            }
        };

        // Update
        auth_file.openai_api_key = Some(CODEX_TOKEN_VALUE.to_string());

        let content = serde_json::to_string_pretty(&auth_file)?;
        secure_write(&auth_path, content.as_bytes())?;

        Ok(())
    }

    async fn disable_auth_proxy(&self) -> AppResult<()> {
        let (auth_path, _) = self.auth_paths()?;
        if !auth_path.exists() {
            return Ok(());
        }

        let content =
            tokio::fs::read_to_string(&auth_path)
                .await
                .map_err(|e| AppError::ConfigRead {
                    path: auth_path.to_string_lossy().to_string(),
                    source: e,
                })?;

        let mut auth_file: AuthFile = match serde_json::from_str(&content) {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        // Remove key
        auth_file.openai_api_key = None;

        let content = serde_json::to_string_pretty(&auth_file)?;
        secure_write(&auth_path, content.as_bytes())?;

        Ok(())
    }
}

impl Default for CodexSettingsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_codex_settings_flow() {
        let dir = tempdir().unwrap();
        let service = CodexSettingsService::new_with_root(dir.path().to_path_buf());
        let proxy_url = "http://127.0.0.1:18099";

        // 1. Initial status false
        assert!(!service.proxy_status().await.unwrap());

        // 2. Enable
        service.enable_proxy(proxy_url).await.unwrap();

        // Verify config
        let (config_path, _) = service.paths().unwrap();
        let content = tokio::fs::read_to_string(&config_path).await.unwrap();
        let config: CodexConfig = toml::from_str(&content).unwrap();
        assert_eq!(config.model_provider.unwrap(), CODEX_PROVIDER_KEY);

        let mp = config.model_providers.unwrap();
        let _provider_val = mp.get(CODEX_PROVIDER_KEY).unwrap();

        // Verify auth
        let (auth_path, _) = service.auth_paths().unwrap();
        assert!(auth_path.exists());
        let content = tokio::fs::read_to_string(&auth_path).await.unwrap();
        let auth: AuthFile = serde_json::from_str(&content).unwrap();
        assert_eq!(auth.openai_api_key.unwrap(), CODEX_TOKEN_VALUE);

        // Status true
        assert!(service.proxy_status().await.unwrap());

        // 3. Disable
        service.disable_proxy().await.unwrap();

        assert!(!service.proxy_status().await.unwrap());

        // Config file should still exist (if we created it) but without provider
        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await.unwrap();
            let config: CodexConfig = toml::from_str(&content).unwrap();
            assert!(
                config.model_provider.is_none()
                    || config.model_provider.unwrap() != CODEX_PROVIDER_KEY
            );
        }

        // Auth file should exist but no key
        if auth_path.exists() {
            let content = tokio::fs::read_to_string(&auth_path).await.unwrap();
            let auth: AuthFile = serde_json::from_str(&content).unwrap();
            assert!(auth.openai_api_key.is_none());
        }
    }

    #[tokio::test]
    async fn test_codex_preserve_other_settings() {
        let dir = tempdir().unwrap();
        let service = CodexSettingsService::new_with_root(dir.path().to_path_buf());
        let (config_path, _) = service.paths().unwrap();
        let (auth_path, _) = service.auth_paths().unwrap();

        ensure_dir(&config_path.parent().unwrap().to_path_buf()).unwrap();

        // Pre-create config
        secure_write(&config_path, b"existing_key = \"val\"\n").unwrap();
        // Pre-create auth
        secure_write(&auth_path, b"{\"other_key\": \"val\"}").unwrap();

        // Enable
        service.enable_proxy("url").await.unwrap();

        let content = tokio::fs::read_to_string(&config_path).await.unwrap();
        assert!(content.contains("existing_key"));
        let content = tokio::fs::read_to_string(&auth_path).await.unwrap();
        assert!(content.contains("other_key"));

        // Disable
        service.disable_proxy().await.unwrap();

        let config_content = tokio::fs::read_to_string(&config_path).await.unwrap();
        assert!(config_content.contains("existing_key"));
        let auth_content = tokio::fs::read_to_string(&auth_path).await.unwrap();
        assert!(auth_content.contains("other_key"));

        // Check removed
        let config: CodexConfig = toml::from_str(&config_content).unwrap();
        assert!(config.model_provider.is_none());
    }

    #[tokio::test]
    async fn test_codex_proxy_status_handles_invalid_toml() {
        let dir = tempdir().unwrap();
        let service = CodexSettingsService::new_with_root(dir.path().to_path_buf());
        let (config_path, _) = service.paths().unwrap();

        ensure_dir(&config_path.parent().unwrap().to_path_buf()).unwrap();
        secure_write(&config_path, b"not = [valid").unwrap();

        // 无法解析 TOML 时应返回 false，并且不 panic
        assert!(!service.proxy_status().await.unwrap());
    }

    #[tokio::test]
    async fn test_codex_enable_proxy_creates_auth_file_when_missing() {
        let dir = tempdir().unwrap();
        let service = CodexSettingsService::new_with_root(dir.path().to_path_buf());
        let proxy_url = "http://127.0.0.1:18099";
        let (auth_path, _) = service.auth_paths().unwrap();

        assert!(
            !auth_path.exists(),
            "auth.json should not exist before enabling in a clean tempdir"
        );

        service.enable_proxy(proxy_url).await.unwrap();

        let content = tokio::fs::read_to_string(&auth_path).await.unwrap();
        let auth: AuthFile = serde_json::from_str(&content).unwrap();
        assert_eq!(auth.openai_api_key.unwrap(), CODEX_TOKEN_VALUE);
    }

    #[tokio::test]
    async fn test_codex_enable_proxy_overrides_model_provider_but_preserves_existing_entries() {
        let dir = tempdir().unwrap();
        let service = CodexSettingsService::new_with_root(dir.path().to_path_buf());
        let (config_path, _) = service.paths().unwrap();

        ensure_dir(&config_path.parent().unwrap().to_path_buf()).unwrap();
        let initial = r#"
model_provider = "legacy"
[model_providers]
[model_providers.legacy]
name = "legacy"
base_url = "https://legacy.example.com"
wire_api = "responses"
requires_openai_auth = true
"#;
        secure_write(&config_path, initial.as_bytes()).unwrap();

        service
            .enable_proxy("http://local-proxy:9000")
            .await
            .unwrap();

        let contents = tokio::fs::read_to_string(&config_path).await.unwrap();
        let config_value: toml::Value = toml::from_str(&contents).unwrap();

        assert_eq!(
            config_value
                .get("model_provider")
                .and_then(|v| v.as_str())
                .unwrap(),
            CODEX_PROVIDER_KEY
        );

        let providers = config_value
            .get("model_providers")
            .and_then(|v| v.as_table())
            .expect("model_providers table should exist");

        assert!(
            providers.contains_key("legacy"),
            "legacy provider should remain"
        );
        assert!(
            providers.contains_key(CODEX_PROVIDER_KEY),
            "iswitch provider should be added"
        );

        let iswitch = providers.get(CODEX_PROVIDER_KEY).unwrap();
        assert_eq!(
            iswitch.get("base_url").and_then(|v| v.as_str()).unwrap(),
            "http://local-proxy:9000"
        );
    }
}
