//! [INPUT]:
//!   source: ../../../../code-switch/services/claudesettings.go ([POS]: 原 Go 实现参考)
//!
//! [OUTPUT]:
//!   - ClaudeSettingsService 结构体
//!   - proxy_status(), enable_proxy(), disable_proxy() API
//!
//! [POS]: Claude Code 代理设置管理
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{AppError, AppResult};
use crate::utils::paths::ensure_dir;
use serde::{Deserialize, Serialize};
use tracing::info;

const CLAUDE_SETTINGS_DIR: &str = ".claude";
const CLAUDE_SETTINGS_FILENAME: &str = "settings.json";
const CLAUDE_BACKUP_FILENAME: &str = "cc-studio.back.settings.json";
const CLAUDE_AUTH_TOKEN_VALUE: &str = "iswitch";

use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeSettingsFile {
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

/// Claude 设置服务
pub struct ClaudeSettingsService {
    root: Option<PathBuf>,
}

impl ClaudeSettingsService {
    pub fn new() -> Self {
        Self { root: None }
    }

    #[cfg(test)]
    pub fn new_with_root(root: PathBuf) -> Self {
        Self { root: Some(root) }
    }

    /// 获取 Claude 配置文件路径和备份路径
    fn paths(&self) -> AppResult<(PathBuf, PathBuf)> {
        let home = if let Some(ref r) = self.root {
            r.clone()
        } else {
            dirs::home_dir().ok_or_else(|| AppError::Internal("无法获取用户主目录".to_string()))?
        };

        let dir = home.join(CLAUDE_SETTINGS_DIR);
        Ok((
            dir.join(CLAUDE_SETTINGS_FILENAME),
            dir.join(CLAUDE_BACKUP_FILENAME),
        ))
    }

    /// 检测代理启用状态
    pub async fn proxy_status(&self) -> AppResult<bool> {
        let (settings_path, _) = self.paths()?;

        if !settings_path.exists() {
            return Ok(false);
        }

        let content = tokio::fs::read_to_string(&settings_path)
            .await
            .map_err(|e| AppError::ConfigRead {
                path: settings_path.to_string_lossy().to_string(),
                source: e,
            })?;

        let settings: ClaudeSettingsFile = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => return Ok(false), // 解析失败视为未启用
        };

        let token_match = settings
            .env
            .get("ANTHROPIC_AUTH_TOKEN")
            .map(|v| v.eq_ignore_ascii_case(CLAUDE_AUTH_TOKEN_VALUE))
            .unwrap_or(false);

        // 只要 token 匹配且 base_url 存在，就认为是启用的
        // 严格来说应该检查 base_url 指向本地代理，但端口可能变动，所以这里主要检查标记
        let base_url_exists = settings.env.contains_key("ANTHROPIC_BASE_URL");

        Ok(token_match && base_url_exists)
    }

    /// 启用代理
    pub async fn enable_proxy(&self, proxy_url: &str) -> AppResult<()> {
        let (settings_path, backup_path) = self.paths()?;

        // 确保目录存在
        if let Some(parent) = settings_path.parent() {
            ensure_dir(&parent.to_path_buf()).map_err(|e| AppError::DirCreate {
                path: parent.to_string_lossy().to_string(),
                source: e,
            })?;
        }

        // 读取现有配置或创建新配置
        let mut settings = if settings_path.exists() {
            // 备份现有配置（如果存在且未备份）
            // 仅作为安全网，不再用于恢复
            if !backup_path.exists() {
                if let Err(e) = tokio::fs::copy(&settings_path, &backup_path).await {
                    // 记录错误但继续
                    info!(error = %e, "备份 Claude 配置文件失败");
                } else {
                    info!(path = %backup_path.display(), "已备份 Claude 配置文件");
                }
            }

            let content = tokio::fs::read_to_string(&settings_path)
                .await
                .map_err(|e| AppError::ConfigRead {
                    path: settings_path.to_string_lossy().to_string(),
                    source: e,
                })?;
            serde_json::from_str(&content).unwrap_or_else(|_| ClaudeSettingsFile {
                env: HashMap::new(),
                other: HashMap::new(),
            })
        } else {
            ClaudeSettingsFile {
                env: HashMap::new(),
                other: HashMap::new(),
            }
        };

        // 更新 env
        settings.env.insert(
            "ANTHROPIC_AUTH_TOKEN".to_string(),
            CLAUDE_AUTH_TOKEN_VALUE.to_string(),
        );
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".to_string(), proxy_url.to_string());

        let content = serde_json::to_string_pretty(&settings)?;

        crate::utils::security::secure_write(&settings_path, content.as_bytes())?;

        info!(path = %settings_path.display(), proxy = %proxy_url, "Claude 代理已启用");

        Ok(())
    }

    /// 禁用代理
    pub async fn disable_proxy(&self) -> AppResult<()> {
        let (settings_path, _) = self.paths()?;

        if !settings_path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&settings_path)
            .await
            .map_err(|e| AppError::ConfigRead {
                path: settings_path.to_string_lossy().to_string(),
                source: e,
            })?;

        let mut settings: ClaudeSettingsFile = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => return Ok(()), //如果文件损坏或格式不对，不做处理直接返回
        };

        // 移除代理配置
        settings.env.remove("ANTHROPIC_AUTH_TOKEN");
        settings.env.remove("ANTHROPIC_BASE_URL");

        let content = serde_json::to_string_pretty(&settings)?;
        crate::utils::security::secure_write(&settings_path, content.as_bytes())?;

        info!("Claude 代理已禁用");

        Ok(())
    }
}

impl Default for ClaudeSettingsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_claude_settings_flow() {
        let dir = tempdir().unwrap();
        let service = ClaudeSettingsService::new_with_root(dir.path().to_path_buf());
        let proxy_url = "http://127.0.0.1:18099";

        // 1. Initial status should be false
        assert!(!service.proxy_status().await.unwrap());

        // 2. Enable proxy
        service.enable_proxy(proxy_url).await.unwrap();

        // Verify file content
        let (settings_path, _) = service.paths().unwrap();
        let content = tokio::fs::read_to_string(&settings_path).await.unwrap();
        let settings: ClaudeSettingsFile = serde_json::from_str(&content).unwrap();
        assert_eq!(settings.env.get("ANTHROPIC_BASE_URL").unwrap(), proxy_url);

        // Status should be true
        assert!(service.proxy_status().await.unwrap());

        // 3. Disable proxy
        service.disable_proxy().await.unwrap();

        // Status should be false
        assert!(!service.proxy_status().await.unwrap());

        // File should still exist but keys removed
        let content = tokio::fs::read_to_string(&settings_path).await.unwrap();
        let settings: ClaudeSettingsFile = serde_json::from_str(&content).unwrap();
        assert!(settings.env.get("ANTHROPIC_BASE_URL").is_none());
    }

    #[tokio::test]
    async fn test_claude_preserve_other_settings() {
        let dir = tempdir().unwrap();
        let service = ClaudeSettingsService::new_with_root(dir.path().to_path_buf());
        let (settings_path, _) = service.paths().unwrap();

        // Create initial settings with other fields
        ensure_dir(&settings_path.parent().unwrap().to_path_buf()).unwrap();
        tokio::fs::write(
            &settings_path,
            r#"{"existing_key": "existing_value", "env": {"EXISTING_ENV": "true"}}"#,
        )
        .await
        .unwrap();

        // Enable proxy
        service.enable_proxy("url").await.unwrap();

        let content = tokio::fs::read_to_string(&settings_path).await.unwrap();
        let settings: ClaudeSettingsFile = serde_json::from_str(&content).unwrap();

        // Check preserved values
        assert_eq!(
            settings.other.get("existing_key").unwrap(),
            "existing_value"
        );
        assert_eq!(settings.env.get("EXISTING_ENV").unwrap(), "true");
        // Check new values
        assert_eq!(
            settings.env.get("ANTHROPIC_AUTH_TOKEN").unwrap(),
            CLAUDE_AUTH_TOKEN_VALUE
        );

        // Disable proxy
        service.disable_proxy().await.unwrap();

        let content = tokio::fs::read_to_string(&settings_path).await.unwrap();
        let settings: ClaudeSettingsFile = serde_json::from_str(&content).unwrap();

        // Check preserved values again
        assert_eq!(
            settings.other.get("existing_key").unwrap(),
            "existing_value"
        );
        assert_eq!(settings.env.get("EXISTING_ENV").unwrap(), "true");
        // Check removed values
        assert!(settings.env.get("ANTHROPIC_AUTH_TOKEN").is_none());
    }

    #[tokio::test]
    async fn test_claude_proxy_status_handles_invalid_json() {
        let dir = tempdir().unwrap();
        let service = ClaudeSettingsService::new_with_root(dir.path().to_path_buf());
        let (settings_path, _) = service.paths().unwrap();

        ensure_dir(&settings_path.parent().unwrap().to_path_buf()).unwrap();
        tokio::fs::write(&settings_path, b"not valid json")
            .await
            .unwrap();

        assert!(
            !service.proxy_status().await.unwrap(),
            "invalid JSON should be treated as disabled"
        );
    }

    #[tokio::test]
    async fn test_claude_enable_proxy_creates_backup_for_existing_file() {
        let dir = tempdir().unwrap();
        let service = ClaudeSettingsService::new_with_root(dir.path().to_path_buf());
        let (settings_path, backup_path) = service.paths().unwrap();

        ensure_dir(&settings_path.parent().unwrap().to_path_buf()).unwrap();
        tokio::fs::write(
            &settings_path,
            r#"{"env": {"EXISTING": "VALUE"}, "other": {"key": 1}}"#,
        )
        .await
        .unwrap();

        service.enable_proxy("http://localhost:1234").await.unwrap();

        assert!(
            backup_path.exists(),
            "enabling proxy should create a backup of existing settings"
        );
        let backup_content = tokio::fs::read_to_string(&backup_path).await.unwrap();
        assert!(backup_content.contains("EXISTING"));
    }

    #[tokio::test]
    async fn test_claude_disable_proxy_is_noop_when_file_missing() {
        let dir = tempdir().unwrap();
        let service = ClaudeSettingsService::new_with_root(dir.path().to_path_buf());
        assert!(service.disable_proxy().await.is_ok());
    }
}
