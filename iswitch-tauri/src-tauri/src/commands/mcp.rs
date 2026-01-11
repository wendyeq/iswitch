//! [INPUT]:
//!   source: ../../../openspec/changes/migrate-to-tauri-stack/design.md ([POS]: Command 定义)
//!   source: ../services/mcp_service.rs ([POS]: 调用 Service)
//!
//! [OUTPUT]:
//!   - list_mcp_servers
//!   - save_mcp_servers
//!
//! [POS]: MCP 模块的前端调用接口
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::{AppError, AppResult};
use crate::models::MCPServer;
use crate::services::MCPService;
use tauri::State;
use tracing::info;

/// 列出所有 MCP Servers
#[tauri::command]
pub async fn list_mcp_servers(service: State<'_, MCPService>) -> AppResult<Vec<MCPServer>> {
    service.list_servers().await
}

/// 保存 MCP Servers 并同步到各平台
#[tauri::command]
pub async fn save_mcp_servers(
    servers: Vec<MCPServer>,
    service: State<'_, MCPService>,
) -> AppResult<()> {
    info!(count = servers.len(), "保存 MCP Servers");

    // 1. 保存到本地配置
    service.save_servers(servers).await?;

    // 2. 同步到 Claude Desktop
    if let Err(e) = service.sync_claude_servers().await {
        tracing::error!("同步到 Claude Desktop 失败: {}", e);
        return Err(AppError::Internal(format!(
            "保存成功，但在同步到 Claude 时失败: {}",
            e
        )));
    }

    // 3. 同步到 Codex
    if let Err(e) = service.sync_codex_servers().await {
        tracing::error!("同步到 Codex 失败: {}", e);
        return Err(AppError::Internal(format!(
            "保存成功，但在同步到 Codex 时失败: {}",
            e
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MCPServerType;
    use std::fs::File;
    use std::io::Write;
    use tauri::Manager;
    use tempfile::tempdir;

    fn create_dummy_file(path: &std::path::Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[tokio::test]
    async fn test_list_mcp_servers_command() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let service = MCPService::with_paths(
            mcp_path.clone(),
            dir.path().join("claude.json"),
            dir.path().join("codex.toml"),
        );

        // Mock data
        let mut raw_map = std::collections::HashMap::new();
        raw_map.insert(
            "test".to_string(),
            crate::models::RawMCPServer {
                command: "ls".to_string(),
                ..Default::default()
            },
        );
        create_dummy_file(&mcp_path, &serde_json::to_string(&raw_map).unwrap());

        // We use State::new for testing if possible, but tauri::State doesn't have a public constructor easily usable this way.
        // However, we can use tauri::test utilities if we want to be official.
        // For simplicity, we can also test the logic by calling the service, but the user wants to test the COMMAND.

        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(service);
        let state = app.state::<MCPService>();

        let result = list_mcp_servers(state).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "test");
    }

    #[tokio::test]
    async fn test_save_mcp_servers_command_success() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude.json");
        let codex_path = dir.path().join("codex.toml");

        // Setup Codex config file (it must exist for sync to NOT warn/skip, although current implementation skips if missing)
        create_dummy_file(&codex_path, "[experimental]");

        let service =
            MCPService::with_paths(mcp_path.clone(), claude_path.clone(), codex_path.clone());

        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(service);
        let state = app.state::<MCPService>();

        let servers = vec![MCPServer {
            name: "new-server".to_string(),
            server_type: MCPServerType::Stdio,
            command: "echo".to_string(),
            enabled_in_claude: true,
            enabled_in_codex: true,
            ..Default::default()
        }];

        save_mcp_servers(servers, state).await.unwrap();

        // Verify files
        assert!(mcp_path.exists());
        assert!(claude_path.exists());
        assert!(codex_path.exists());
    }
}
