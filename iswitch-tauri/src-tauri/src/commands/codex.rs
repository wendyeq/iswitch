//! [INPUT]:
//!   source: ../services/codex_settings.rs ([POS]: Codex 设置服务)
//!
//! [OUTPUT]:
//!   - get_codex_proxy_status command
//!   - enable_codex_proxy command
//!   - disable_codex_proxy command
//!
//! [POS]: Codex 设置相关的 Tauri Commands
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::AppResult;
use crate::services::{AppSettingsService, CodexSettingsService};
use tauri::State;

/// 获取 Codex 代理状态
#[tauri::command]
pub async fn get_codex_proxy_status(state: State<'_, CodexSettingsService>) -> AppResult<bool> {
    state.proxy_status().await
}

/// 启用 Codex 代理
#[tauri::command]
pub async fn enable_codex_proxy(
    state: State<'_, CodexSettingsService>,
    settings_state: State<'_, AppSettingsService>,
) -> AppResult<()> {
    let settings = settings_state.get_settings().await?;
    let proxy_url = format!("http://127.0.0.1:{}", settings.proxy_port);
    state.enable_proxy(&proxy_url).await
}

/// 禁用 Codex 代理
#[tauri::command]
pub async fn disable_codex_proxy(state: State<'_, CodexSettingsService>) -> AppResult<()> {
    state.disable_proxy().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AppSettings;
    use serde_json::Value as JsonValue;
    use tauri::Manager;
    use tempfile::tempdir;
    use toml::Value as TomlValue;

    fn temp_services(
        root: &std::path::Path,
    ) -> (CodexSettingsService, AppSettingsService, std::path::PathBuf) {
        let codex_service = CodexSettingsService::new_with_root(root.to_path_buf());
        let settings_path = root.join("settings.json");
        let app_settings = AppSettingsService::with_path(settings_path.clone());
        (codex_service, app_settings, settings_path)
    }

    #[tokio::test]
    async fn test_enable_codex_proxy_command_updates_config_and_auth() {
        let dir = tempdir().unwrap();
        let (codex_service, app_settings, _) = temp_services(dir.path());

        app_settings
            .save_settings(AppSettings {
                proxy_port: 18888,
                ..Default::default()
            })
            .await
            .unwrap();

        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(codex_service);
        app.manage(app_settings);

        let codex_state = app.state::<CodexSettingsService>();
        let settings_state = app.state::<AppSettingsService>();

        enable_codex_proxy(codex_state.clone(), settings_state.clone())
            .await
            .unwrap();

        let config_path = dir.path().join(".codex").join("config.toml");
        let config_content = tokio::fs::read_to_string(&config_path).await.unwrap();
        let config: TomlValue = toml::from_str(&config_content).unwrap();
        assert_eq!(
            config
                .get("model_provider")
                .and_then(|v| v.as_str())
                .unwrap(),
            "iswitch"
        );

        let auth_path = dir.path().join(".codex").join("auth.json");
        let auth_content = tokio::fs::read_to_string(&auth_path).await.unwrap();
        let auth: JsonValue = serde_json::from_str(&auth_content).unwrap();
        assert_eq!(auth["OPENAI_API_KEY"], "iswitch");
    }

    #[tokio::test]
    async fn test_disable_codex_proxy_command_removes_provider() {
        let dir = tempdir().unwrap();
        let (codex_service, app_settings, _) = temp_services(dir.path());

        // Enable first to create files.
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(codex_service);
        app.manage(app_settings);
        let codex_state = app.state::<CodexSettingsService>();
        let settings_state = app.state::<AppSettingsService>();

        enable_codex_proxy(codex_state.clone(), settings_state.clone())
            .await
            .unwrap();
        disable_codex_proxy(codex_state.clone()).await.unwrap();

        let config_path = dir.path().join(".codex").join("config.toml");
        let config_content = tokio::fs::read_to_string(&config_path).await.unwrap();
        let config: TomlValue = toml::from_str(&config_content).unwrap();
        assert!(
            config
                .get("model_provider")
                .and_then(|v| v.as_str())
                .is_none(),
            "model_provider should be removed after disabling proxy"
        );

        let auth_path = dir.path().join(".codex").join("auth.json");
        let auth_content = tokio::fs::read_to_string(&auth_path).await.unwrap();
        let auth: JsonValue = serde_json::from_str(&auth_content).unwrap();
        assert!(auth["OPENAI_API_KEY"].is_null());
    }
}
