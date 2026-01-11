//! [INPUT]:
//!   source: ../../../../code-switch/services/appsettings.go ([POS]: 原 Go 实现参考)
//!   source: ../models/settings.rs ([POS]: Settings 数据模型)
//!
//! [OUTPUT]:
//!   - AppSettingsService 结构体
//!   - get_settings(), save_settings() API
//!
//! [POS]: 应用设置管理服务，处理设置持久化和开机自启动控制
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::{AppError, AppResult};
use crate::models::AppSettings;
use crate::utils::paths::app_settings_path;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

/// 应用设置服务
///
/// 管理应用设置的加载、保存和开机自启动控制
pub struct AppSettingsService {
    /// 设置文件路径 (可注入用于测试)
    path: PathBuf,
}

impl AppSettingsService {
    /// 创建新的应用设置服务实例
    pub fn new() -> Self {
        Self {
            path: app_settings_path(),
        }
    }

    /// 创建带有自定义路径的应用设置服务 (用于测试)
    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// 获取应用设置
    ///
    /// 如果配置文件不存在，返回默认设置
    pub async fn get_settings(&self) -> AppResult<AppSettings> {
        debug!(path = %self.path.display(), "加载应用设置");

        // 如果文件不存在，返回默认设置
        if !self.path.exists() {
            info!("应用设置文件不存在，使用默认设置");
            return Ok(AppSettings::default());
        }

        // 读取并解析设置文件
        let content = tokio::fs::read_to_string(&self.path).await.map_err(|e| {
            error!(error = %e, path = %self.path.display(), "读取设置文件失败");
            AppError::ConfigRead {
                path: self.path.display().to_string(),
                source: e,
            }
        })?;

        // 空文件返回默认设置
        if content.trim().is_empty() {
            warn!("设置文件为空，使用默认设置");
            return Ok(AppSettings::default());
        }

        // 解析 JSON
        let settings: AppSettings = serde_json::from_str(&content).map_err(|e| {
            error!(error = %e, "解析设置文件失败");
            AppError::ConfigParse(e)
        })?;

        debug!(?settings, "已加载应用设置");
        Ok(settings)
    }

    /// 保存应用设置
    ///
    /// 同时处理开机自启动状态的同步
    pub async fn save_settings(&self, settings: AppSettings) -> AppResult<AppSettings> {
        debug!(?settings, "保存应用设置");

        // 确保父目录存在
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                error!(error = %e, path = %parent.display(), "创建设置目录失败");
                AppError::ConfigWrite {
                    path: parent.display().to_string(),
                    source: e,
                }
            })?;
        }

        // 序列化设置为 JSON
        let content = serde_json::to_string_pretty(&settings).map_err(|e| {
            error!(error = %e, "序列化设置失败");
            AppError::Serialize(e.to_string())
        })?;

        // 写入文件
        tokio::fs::write(&self.path, content).await.map_err(|e| {
            error!(error = %e, path = %self.path.display(), "写入设置文件失败");
            AppError::ConfigWrite {
                path: self.path.display().to_string(),
                source: e,
            }
        })?;

        info!(path = %self.path.display(), "已保存应用设置");
        Ok(settings)
    }
}

impl Default for AppSettingsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_get_settings_default_when_file_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("settings.json");
        let service = AppSettingsService::with_path(path);

        let settings = service.get_settings().await.unwrap();

        assert!(settings.show_heatmap);
        assert!(settings.auto_start);
    }

    #[tokio::test]
    async fn test_save_and_get_settings() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("settings.json");
        let service = AppSettingsService::with_path(path);

        // 保存设置
        let settings = AppSettings {
            show_heatmap: false,
            auto_start: true,
            proxy_port: 18099,
            failover_threshold: 5,
            recovery_timeout_secs: 300,
            hud: None,
        };
        service.save_settings(settings.clone()).await.unwrap();

        // 读取设置
        let loaded = service.get_settings().await.unwrap();
        assert!(!loaded.show_heatmap);
        assert!(loaded.auto_start);
    }

    #[tokio::test]
    async fn test_get_settings_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("settings.json");

        // 创建空文件
        tokio::fs::write(&path, "").await.unwrap();

        let service = AppSettingsService::with_path(path);
        let settings = service.get_settings().await.unwrap();

        // 应该返回默认设置
        assert!(settings.show_heatmap);
    }

    #[tokio::test]
    async fn test_get_settings_partial_json() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("settings.json");

        // 创建只有部分字段的 JSON
        tokio::fs::write(&path, r#"{"show_heatmap": false}"#)
            .await
            .unwrap();

        let service = AppSettingsService::with_path(path);
        let settings = service.get_settings().await.unwrap();

        // show_heatmap 应该是 false，其他使用默认值
        assert!(!settings.show_heatmap);
        assert!(settings.auto_start); // 默认值
    }
}
