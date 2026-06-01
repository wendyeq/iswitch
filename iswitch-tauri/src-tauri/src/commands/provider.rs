//! [INPUT]:
//!   source: ../services/provider_service.rs ([POS]: Provider 服务)
//!   source: ../models/provider.rs ([POS]: Provider 数据模型)
//!
//! [OUTPUT]:
//!   - load_providers command
//!   - save_providers command
//!   - get_proxy_status command
//!
//! [POS]: Provider 相关的 Tauri Commands
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::AppResult;
use crate::models::Provider;
use crate::services::{AppSettingsService, ProviderService};
use std::sync::Arc;
use tauri::State;

/// 加载 Provider 列表
#[tauri::command]
pub async fn load_providers(
    kind: String,
    service: State<'_, Arc<ProviderService>>,
) -> AppResult<Vec<Provider>> {
    service.load_providers(&kind).await
}

/// 保存 Provider 列表
#[tauri::command]
pub async fn save_providers(
    kind: String,
    providers: Vec<Provider>,
    service: State<'_, Arc<ProviderService>>,
) -> AppResult<()> {
    service.save_providers(&kind, providers).await
}

/// 获取代理状态
#[tauri::command]
pub async fn get_proxy_status(
    settings_state: State<'_, AppSettingsService>,
) -> AppResult<ProxyStatus> {
    let settings = settings_state.get_settings().await?;
    // Phase 2: 代理服务器随应用启动常驻运行
    // TODO: 结合实际运行状态（从 ProxyService 获取）
    Ok(ProxyStatus {
        running: true,
        address: "127.0.0.1".to_string(),
        port: settings.proxy_port,
    })
}

/// 代理状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ProxyStatus {
    pub running: bool,
    pub address: String,
    pub port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppSettings, Provider};
    use crate::services::{AppSettingsService, ProviderService};
    use std::sync::Arc;
    use tauri::Manager;
    use tempfile::tempdir;

    fn build_mock_app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("failed to construct mock app")
    }

    #[tokio::test]
    async fn test_load_providers_command_returns_saved_entries() {
        let dir = tempdir().unwrap();
        let service = ProviderService::with_root(dir.path().to_path_buf());

        // Pre-populate provider data so the command exercises real IO
        service
            .save_providers(
                "claude",
                vec![Provider {
                    id: 42,
                    name: "Test Claude".to_string(),
                    api_key: "sk-test".to_string(),
                    ..Default::default()
                }],
            )
            .await
            .unwrap();

        let app = build_mock_app();
        app.manage(Arc::new(service));
        let state = app.state::<Arc<ProviderService>>();

        let providers = load_providers("claude".to_string(), state)
            .await
            .expect("command should load providers");

        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, 42);
        assert_eq!(providers[0].name, "Test Claude");
        assert_eq!(providers[0].api_key, "sk-test");
    }

    #[tokio::test]
    async fn test_get_proxy_status_uses_saved_app_settings() {
        let dir = tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        let service = AppSettingsService::with_path(settings_path.clone());

        service
            .save_settings(AppSettings {
                proxy_port: 19001,
                ..Default::default()
            })
            .await
            .unwrap();

        let app = build_mock_app();
        app.manage(service);
        let state = app.state::<AppSettingsService>();

        let status = get_proxy_status(state)
            .await
            .expect("command should return proxy status");

        assert!(status.running);
        assert_eq!(status.address, "127.0.0.1");
        assert_eq!(status.port, 19001);
    }
}
