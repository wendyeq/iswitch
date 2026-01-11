//! [INPUT]:
//!   source: ../services/claude_settings.rs ([POS]: Claude 设置服务)
//!
//! [OUTPUT]:
//!   - get_claude_proxy_status command
//!   - enable_claude_proxy command
//!   - disable_claude_proxy command
//!
//! [POS]: Claude Code 设置相关的 Tauri Commands
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::AppResult;
use crate::services::{AppSettingsService, ClaudeSettingsService};
use tauri::State;

/// 获取 Claude 代理状态
#[tauri::command]
pub async fn get_claude_proxy_status(state: State<'_, ClaudeSettingsService>) -> AppResult<bool> {
    state.proxy_status().await
}

/// 启用 Claude 代理
#[tauri::command]
pub async fn enable_claude_proxy(
    state: State<'_, ClaudeSettingsService>,
    settings_state: State<'_, AppSettingsService>,
) -> AppResult<()> {
    let settings = settings_state.get_settings().await?;
    let proxy_url = format!("http://127.0.0.1:{}", settings.proxy_port);
    state.enable_proxy(&proxy_url).await
}

/// 禁用 Claude 代理
#[tauri::command]
pub async fn disable_claude_proxy(state: State<'_, ClaudeSettingsService>) -> AppResult<()> {
    state.disable_proxy().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AppSettings;
    use tauri::Manager;
    use tempfile::tempdir;

    fn temp_services(
        root: &std::path::Path,
    ) -> (
        ClaudeSettingsService,
        AppSettingsService,
        std::path::PathBuf,
    ) {
        let claude_service = ClaudeSettingsService::new_with_root(root.to_path_buf());
        let settings_path = root.join("settings.json");
        let app_settings = AppSettingsService::with_path(settings_path.clone());
        (claude_service, app_settings, settings_path)
    }

    #[tokio::test]
    async fn test_enable_claude_proxy_uses_app_settings_port() {
        let dir = tempdir().unwrap();
        let (claude_service, app_settings, _settings_path) = temp_services(dir.path());

        // Persist custom proxy port in app settings.
        app_settings
            .save_settings(AppSettings {
                proxy_port: 19999,
                ..Default::default()
            })
            .await
            .unwrap();

        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(claude_service);
        app.manage(app_settings);

        let claude_state = app.state::<ClaudeSettingsService>();
        let settings_state = app.state::<AppSettingsService>();

        enable_claude_proxy(claude_state.clone(), settings_state.clone())
            .await
            .unwrap();

        let claude_settings_path = dir.path().join(".claude").join("settings.json");
        let content = tokio::fs::read_to_string(&claude_settings_path)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json["env"]["ANTHROPIC_BASE_URL"], "http://127.0.0.1:19999");
        assert_eq!(json["env"]["ANTHROPIC_AUTH_TOKEN"], "iswitch");
    }

    #[tokio::test]
    async fn test_get_and_disable_claude_proxy_commands() {
        let dir = tempdir().unwrap();
        let (claude_service, app_settings, _) = temp_services(dir.path());

        // Default settings use proxy_port = 18099.
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(claude_service);
        app.manage(app_settings);

        let claude_state = app.state::<ClaudeSettingsService>();
        let settings_state = app.state::<AppSettingsService>();

        enable_claude_proxy(claude_state.clone(), settings_state.clone())
            .await
            .unwrap();
        assert!(get_claude_proxy_status(claude_state.clone()).await.unwrap());

        disable_claude_proxy(claude_state.clone()).await.unwrap();
        assert!(
            !get_claude_proxy_status(app.state::<ClaudeSettingsService>())
                .await
                .unwrap()
        );
    }
}
