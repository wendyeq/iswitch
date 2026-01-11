//! [INPUT]:
//!   source: ../../../../code-switch/services/mcpservice.go ([POS]: 原 Go 实现参考)
//!   source: ../models/mcp.rs ([POS]: MCP 数据模型)
//!
//! [OUTPUT]:
//!   - MCPService 结构体
//!   - list_servers(), save_servers() API
//!   - sync_claude_servers(), sync_codex_servers() 同步逻辑
//!   - import_from_claude() 导入逻辑
//!
//! [POS]: MCP Server 配置管理，支持双平台同步和占位符检测
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::{AppError, AppResult};
use crate::models::{builtin_servers, ClaudeDesktopServer, MCPServer, MCPServerType, RawMCPServer};
use crate::utils::paths::{claude_mcp_config_path, codex_config_path, mcp_servers_path};
use crate::utils::security::secure_write;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use tokio::fs;
use toml::Table;
use tracing::{info, warn};

use std::path::PathBuf;

/// MCP 服务
pub struct MCPService {
    mcp_config_path: PathBuf,
    claude_config_path: PathBuf,
    codex_config_path: PathBuf,
}

impl MCPService {
    pub fn new() -> Self {
        Self {
            mcp_config_path: mcp_servers_path(),
            claude_config_path: claude_mcp_config_path(),
            codex_config_path: codex_config_path(),
        }
    }

    /// 用于测试的构造函数，允许注入自定义路径
    #[cfg(test)]
    pub fn with_paths(mcp_path: PathBuf, claude_path: PathBuf, codex_path: PathBuf) -> Self {
        Self {
            mcp_config_path: mcp_path,
            claude_config_path: claude_path,
            codex_config_path: codex_path,
        }
    }

    /// 列出所有 MCP Server
    pub async fn list_servers(&self) -> AppResult<Vec<MCPServer>> {
        let path = &self.mcp_config_path;

        let raw_servers = if path.exists() {
            let content = fs::read_to_string(&path)
                .await
                .map_err(|e| AppError::ConfigRead {
                    path: path.display().to_string(),
                    source: e,
                })?;

            // Allow broken files to return defaults or empty?
            // Better to error if format is wrong, but default to empty if file is empty.
            if content.trim().is_empty() {
                HashMap::new()
            } else {
                serde_json::from_str::<HashMap<String, RawMCPServer>>(&content)?
            }
        } else {
            // 如果不存在，返回内置默认配置
            builtin_servers()
        };

        let mut servers = Vec::new();
        for (name, raw) in raw_servers {
            let missing_placeholders = self.check_missing_placeholders(&raw);

            servers.push(MCPServer {
                name,
                server_type: raw.server_type,
                command: raw.command,
                args: raw.args,
                env: raw.env,
                url: raw.url,
                website: raw.website,
                tips: raw.tips,
                enable_platform: raw.enable_platform.clone(),
                enabled_in_claude: raw.enable_platform.iter().any(|p| p == "claude-code"),
                enabled_in_codex: raw.enable_platform.iter().any(|p| p == "codex"),
                missing_placeholders,
            });
        }

        // 按名称排序
        servers.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(servers)
    }

    /// 保存 MCP Server 列表
    pub async fn save_servers(&self, servers: Vec<MCPServer>) -> AppResult<()> {
        let mut raw_map = HashMap::new();

        for server in servers {
            let mut platform = Vec::new();
            if server.enabled_in_claude {
                platform.push("claude-code".to_string());
            }
            if server.enabled_in_codex {
                platform.push("codex".to_string());
            }

            raw_map.insert(
                server.name,
                RawMCPServer {
                    server_type: server.server_type,
                    command: server.command,
                    args: server.args,
                    env: server.env,
                    url: server.url,
                    website: server.website,
                    tips: server.tips,
                    enable_platform: platform,
                },
            );
        }

        let path = &self.mcp_config_path;

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| AppError::DirCreate {
                        path: parent.display().to_string(),
                        source: e,
                    })?;
            }
        }
        let content = serde_json::to_string_pretty(&raw_map)?;
        secure_write(&path, content.as_bytes())?;

        info!("MCP Servers 配置已保存");
        Ok(())
    }

    /// 同步到 Claude Desktop
    pub async fn sync_claude_servers(&self) -> AppResult<()> {
        let servers = self.list_servers().await?;
        let claude_servers: HashMap<String, ClaudeDesktopServer> = servers
            .into_iter()
            .filter(|s| s.enabled_in_claude && s.missing_placeholders.is_empty())
            .map(|s| (s.name.clone(), s.to_claude_desktop()))
            .collect();

        let path = &self.claude_config_path;

        // 读取现有配置（如果有）以保留其他设置
        let mut config_json = if path.exists() {
            let content = fs::read_to_string(&path)
                .await
                .unwrap_or_else(|_| "{}".to_string());
            serde_json::from_str::<serde_json::Value>(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // 更新 mcpServers 字段
        if let Some(obj) = config_json.as_object_mut() {
            obj.insert(
                "mcpServers".to_string(),
                serde_json::to_value(claude_servers)?,
            );
        }

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| AppError::DirCreate {
                        path: parent.display().to_string(),
                        source: e,
                    })?;
            }
        }

        let content = serde_json::to_string_pretty(&config_json)?;
        secure_write(&path, content.as_bytes())?;

        info!("MCP Servers 已同步到 Claude Desktop");
        Ok(())
    }

    /// 同步到 Codex (config.toml)
    pub async fn sync_codex_servers(&self) -> AppResult<()> {
        let servers = self.list_servers().await?;
        let codex_servers: Vec<MCPServer> = servers
            .into_iter()
            .filter(|s| s.enabled_in_codex && s.missing_placeholders.is_empty())
            .collect();

        let path = &self.codex_config_path;
        if !path.exists() {
            warn!("Codex 配置文件不存在，跳过同步");
            return Ok(());
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| AppError::ConfigRead {
                path: path.display().to_string(),
                source: e,
            })?;

        let mut config: Table = toml::from_str(&content).unwrap_or_else(|_| Table::new());

        // 构建 experimental.mcp_servers
        let mut mcp_servers_map = Table::new();
        for s in codex_servers {
            let mut server_table = Table::new();
            match s.server_type {
                MCPServerType::Stdio => {
                    server_table.insert("command".to_string(), toml::Value::String(s.command));

                    let args_val = s.args.into_iter().map(toml::Value::String).collect();
                    server_table.insert("args".to_string(), toml::Value::Array(args_val));

                    let mut env_table = Table::new();
                    for (k, v) in s.env {
                        env_table.insert(k, toml::Value::String(v));
                    }
                    if !env_table.is_empty() {
                        server_table.insert("env".to_string(), toml::Value::Table(env_table));
                    }
                }
                MCPServerType::Http => {
                    // Codex 目前 experimental 可能不完全支持 HTTP 类型，但按照 spec 生成
                    // 假设 Codex 支持 url 字段
                    server_table.insert("url".to_string(), toml::Value::String(s.url));
                }
            }
            mcp_servers_map.insert(s.name, toml::Value::Table(server_table));
        }

        // 更新 experimental 表
        let experimental = config
            .entry("experimental")
            .or_insert(toml::Value::Table(Table::new()));

        if let toml::Value::Table(ref mut exp_table) = experimental {
            exp_table.insert(
                "mcp_servers".to_string(),
                toml::Value::Table(mcp_servers_map),
            );
        }

        let new_content = toml::to_string_pretty(&config)
            .map_err(|e| AppError::Internal(format!("TOML serialization failed: {}", e)))?;
        secure_write(&path, new_content.as_bytes())?;

        info!("MCP Servers 已同步到 Codex");
        Ok(())
    }

    pub async fn import_from_claude(&self) -> AppResult<Vec<MCPServer>> {
        let path = &self.claude_config_path;
        if !path.exists() {
            warn!("Claude 配置文件不存在");
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| AppError::ConfigRead {
                path: path.display().to_string(),
                source: e,
            })?;

        let config_json: serde_json::Value = serde_json::from_str(&content)?;
        let mut params_servers = Vec::new();

        // 读取 mcpServers
        if let Some(mcp_servers) = config_json.get("mcpServers").and_then(|v| v.as_object()) {
            for (name, value) in mcp_servers {
                let claude_server: ClaudeDesktopServer = serde_json::from_value(value.clone())?;

                // 转换为内部格式
                let server_type = if !claude_server.url.is_empty() {
                    MCPServerType::Http
                } else {
                    MCPServerType::Stdio
                };

                let server = MCPServer {
                    name: name.clone(),
                    server_type,
                    command: claude_server.command,
                    args: claude_server.args,
                    env: claude_server.env,
                    url: claude_server.url,
                    website: "".to_string(),
                    tips: "Imported from Claude Desktop".to_string(),
                    enable_platform: vec!["claude-code".to_string()], // 默认只启用 Claude
                    enabled_in_claude: true,
                    enabled_in_codex: false,
                    missing_placeholders: Vec::new(),
                };
                params_servers.push(server);
            }
        }

        Ok(params_servers)
    }

    /// 检查并返回缺失的占位符列表
    ///
    /// 检查 args, env 值, url 中是否包含 `{VAR}` 格式的占位符
    pub fn check_missing_placeholders(&self, server: &RawMCPServer) -> Vec<String> {
        let mut placeholders = HashSet::new();
        // Lazily compile regex or use strict definition
        // Just recreate it, it's cheap enough here
        let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}").expect("Invalid regex");

        // 检查 args
        for arg in &server.args {
            for cap in re.captures_iter(arg) {
                if let Some(m) = cap.get(1) {
                    placeholders.insert(m.as_str().to_string());
                }
            }
        }

        // 检查 env
        for (_, val) in &server.env {
            for cap in re.captures_iter(val) {
                if let Some(m) = cap.get(1) {
                    placeholders.insert(m.as_str().to_string());
                }
            }
        }

        // 检查 url
        for cap in re.captures_iter(&server.url) {
            if let Some(m) = cap.get(1) {
                placeholders.insert(m.as_str().to_string());
            }
        }

        let mut list: Vec<String> = placeholders.into_iter().collect();
        list.sort();
        list
    }
}

impl Default for MCPService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    // Helper to create a dummy config file
    fn create_dummy_file(path: &std::path::Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_check_missing_placeholders() {
        let service = MCPService::new();
        let mut server = RawMCPServer::default();

        server.args = vec!["--key", "{API_KEY}"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        server
            .env
            .insert("TOKEN".to_string(), "Basic {AUTH_TOKEN}".to_string());
        server.url = "https://api.example.com/v1/{MODEL_ID}".to_string();

        let placeholders = service.check_missing_placeholders(&server);
        assert_eq!(placeholders, vec!["API_KEY", "AUTH_TOKEN", "MODEL_ID"]);

        // Test no placeholders
        server.args = vec!["--noflag"].iter().map(|s| s.to_string()).collect();
        server.env.clear();
        server.url = "https://example.com".to_string();

        let placeholders = service.check_missing_placeholders(&server);
        assert!(placeholders.is_empty());
    }

    #[tokio::test]
    async fn test_list_servers_defaults() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude_config.json");
        let codex_path = dir.path().join("codex_config.toml");

        let service = MCPService::with_paths(mcp_path.clone(), claude_path, codex_path);

        // When file doesn't exist, should return builtin servers
        let servers = service.list_servers().await.unwrap();
        assert!(!servers.is_empty());
        assert!(servers.iter().any(|s| s.name == "chrome-devtools"));
    }

    #[tokio::test]
    async fn test_sync_claude_servers() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude_config.json");
        let codex_path = dir.path().join("codex_config.toml");

        let service = MCPService::with_paths(mcp_path.clone(), claude_path.clone(), codex_path);

        // 1. Setup MCP servers
        let mut raw_map = HashMap::new();
        raw_map.insert(
            "test-server".to_string(),
            RawMCPServer {
                command: "node".to_string(),
                args: vec!["index.js".to_string()],
                enable_platform: vec!["claude-code".to_string()],
                ..Default::default()
            },
        );
        let content = serde_json::to_string(&raw_map).unwrap();
        create_dummy_file(&mcp_path, &content);

        // 2. Setup existing Claude config to ensure it's preserved
        let initial_claude_config = serde_json::json!({
            "globalShortcut": "Cmd+Shift+O",
            "mcpServers": {}
        });
        create_dummy_file(&claude_path, &initial_claude_config.to_string());

        // 3. Sync
        service.sync_claude_servers().await.unwrap();

        // 4. Verify
        let claude_content = fs::read_to_string(&claude_path).await.unwrap();
        let claude_json: serde_json::Value = serde_json::from_str(&claude_content).unwrap();

        assert_eq!(claude_json["globalShortcut"], "Cmd+Shift+O");
        let servers = claude_json["mcpServers"].as_object().unwrap();
        assert!(servers.contains_key("test-server"));
        assert_eq!(servers["test-server"]["command"], "node");
    }

    #[tokio::test]
    async fn test_sync_codex_servers() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude_config.json");
        let codex_path = dir.path().join("codex_config.toml");

        let service = MCPService::with_paths(mcp_path.clone(), claude_path, codex_path.clone());

        // 1. Setup MCP servers
        let mut raw_map = HashMap::new();
        raw_map.insert(
            "codex-server".to_string(),
            RawMCPServer {
                command: "python".to_string(),
                args: vec!["server.py".to_string()],
                enable_platform: vec!["codex".to_string()],
                ..Default::default()
            },
        );
        let content = serde_json::to_string(&raw_map).unwrap();
        create_dummy_file(&mcp_path, &content);

        // 2. Setup existing Codex config
        let initial_codex_config = r#"
            [ui]
            theme = "dark"
        "#;
        create_dummy_file(&codex_path, initial_codex_config);

        // 3. Sync
        service.sync_codex_servers().await.unwrap();

        // 4. Verify
        let codex_content = fs::read_to_string(&codex_path).await.unwrap();
        let codex_toml: Table = toml::from_str(&codex_content).unwrap();

        let experimental = codex_toml.get("experimental").unwrap().as_table().unwrap();
        let servers = experimental.get("mcp_servers").unwrap().as_table().unwrap();

        assert!(servers.contains_key("codex-server"));
        let server_config = servers.get("codex-server").unwrap().as_table().unwrap();
        assert_eq!(
            server_config.get("command").unwrap().as_str().unwrap(),
            "python"
        );
    }

    // ===== save_servers 测试 =====

    /// 测试保存和加载 servers 往返
    #[tokio::test]
    async fn test_save_and_load_servers() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude_config.json");
        let codex_path = dir.path().join("codex_config.toml");

        let service = MCPService::with_paths(mcp_path.clone(), claude_path, codex_path);

        // 创建测试数据
        let servers = vec![
            MCPServer {
                name: "test-server-1".to_string(),
                server_type: MCPServerType::Stdio,
                command: "node".to_string(),
                args: vec!["index.js".to_string()],
                env: HashMap::from([("KEY".to_string(), "value".to_string())]),
                url: "".to_string(),
                website: "https://example.com".to_string(),
                tips: "Test tip".to_string(),
                enable_platform: vec!["claude-code".to_string()],
                enabled_in_claude: true,
                enabled_in_codex: false,
                missing_placeholders: vec![],
            },
            MCPServer {
                name: "test-server-2".to_string(),
                server_type: MCPServerType::Http,
                command: "".to_string(),
                args: vec![],
                env: HashMap::new(),
                url: "https://mcp.example.com".to_string(),
                website: "".to_string(),
                tips: "".to_string(),
                enable_platform: vec!["codex".to_string()],
                enabled_in_claude: false,
                enabled_in_codex: true,
                missing_placeholders: vec![],
            },
        ];

        // 保存
        service.save_servers(servers.clone()).await.unwrap();

        // 验证文件存在
        assert!(mcp_path.exists());

        // 重新加载
        let loaded = service.list_servers().await.unwrap();

        // 验证数量
        assert_eq!(loaded.len(), 2);

        // 验证内容（按名称排序）
        let s1 = loaded.iter().find(|s| s.name == "test-server-1").unwrap();
        assert_eq!(s1.command, "node");
        assert!(s1.enabled_in_claude);
        assert!(!s1.enabled_in_codex);

        let s2 = loaded.iter().find(|s| s.name == "test-server-2").unwrap();
        assert_eq!(s2.url, "https://mcp.example.com");
        assert!(!s2.enabled_in_claude);
        assert!(s2.enabled_in_codex);
    }

    /// 测试保存空列表
    #[tokio::test]
    async fn test_save_empty_servers() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude_config.json");
        let codex_path = dir.path().join("codex_config.toml");

        let service = MCPService::with_paths(mcp_path.clone(), claude_path, codex_path);

        // 保存空列表
        service.save_servers(vec![]).await.unwrap();

        // 加载
        let loaded = service.list_servers().await.unwrap();
        assert!(loaded.is_empty());
    }

    // ===== import_from_claude 测试 =====

    /// 测试从 Claude Desktop 导入
    #[tokio::test]
    async fn test_import_from_claude() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude_config.json");
        let codex_path = dir.path().join("codex_config.toml");

        let service = MCPService::with_paths(mcp_path, claude_path.clone(), codex_path);

        // 创建模拟 Claude Desktop 配置
        let claude_config = serde_json::json!({
            "mcpServers": {
                "imported-server": {
                    "type": "stdio",
                    "command": "npx",
                    "args": ["-y", "@example/mcp-server"],
                    "env": {"API_KEY": "test-key"}
                }
            }
        });
        create_dummy_file(&claude_path, &claude_config.to_string());

        // 导入
        let imported = service.import_from_claude().await.unwrap();

        assert_eq!(imported.len(), 1);
        let server = &imported[0];
        assert_eq!(server.name, "imported-server");
        assert_eq!(server.command, "npx");
        assert_eq!(server.args, vec!["-y", "@example/mcp-server"]);
        assert_eq!(server.env.get("API_KEY"), Some(&"test-key".to_string()));
        assert!(server.enabled_in_claude);
        assert!(!server.enabled_in_codex);
    }

    /// 测试 Claude 配置不存在时导入
    #[tokio::test]
    async fn test_import_from_claude_not_exists() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("nonexistent.json");
        let codex_path = dir.path().join("codex_config.toml");

        let service = MCPService::with_paths(mcp_path, claude_path, codex_path);

        let imported = service.import_from_claude().await.unwrap();
        assert!(imported.is_empty());
    }

    // ===== 边界条件测试 =====

    /// 测试空文件处理
    #[tokio::test]
    async fn test_list_servers_empty_file() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude_config.json");
        let codex_path = dir.path().join("codex_config.toml");

        // 创建空文件
        create_dummy_file(&mcp_path, "");

        let service = MCPService::with_paths(mcp_path, claude_path, codex_path);

        let servers = service.list_servers().await.unwrap();
        assert!(servers.is_empty());
    }

    /// 测试占位符多种格式
    #[test]
    fn test_check_missing_placeholders_edge_cases() {
        let service = MCPService::new();
        let mut server = RawMCPServer::default();

        // 测试重复占位符
        server.args = vec!["{KEY}".to_string(), "--config={KEY}".to_string()];
        let placeholders = service.check_missing_placeholders(&server);
        assert_eq!(placeholders.len(), 1);
        assert_eq!(placeholders[0], "KEY");

        // 测试下划线变量名
        server.args = vec!["{API_KEY_2}".to_string()];
        let placeholders = service.check_missing_placeholders(&server);
        assert_eq!(placeholders[0], "API_KEY_2");

        // 测试不完整的占位符（应该不匹配）
        server.args = vec!["{UNCLOSED".to_string(), "NO_BRACES".to_string()];
        let placeholders = service.check_missing_placeholders(&server);
        assert!(placeholders.is_empty());
    }

    /// 测试 Codex 配置不存在时同步
    #[tokio::test]
    async fn test_sync_codex_servers_file_not_exists() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude_config.json");
        let codex_path = dir.path().join("nonexistent.toml");

        let service = MCPService::with_paths(mcp_path, claude_path, codex_path);

        // 应该成功但跳过
        let result = service.sync_codex_servers().await;
        assert!(result.is_ok());
    }

    /// 测试带占位符的 server 不被同步
    #[tokio::test]
    async fn test_sync_skips_servers_with_placeholders() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude_config.json");
        let codex_path = dir.path().join("codex_config.toml");

        let service = MCPService::with_paths(mcp_path.clone(), claude_path.clone(), codex_path);

        // 创建带占位符的 server
        let mut raw_map = HashMap::new();
        raw_map.insert(
            "server-with-placeholder".to_string(),
            RawMCPServer {
                command: "node".to_string(),
                args: vec!["{MISSING_VAR}".to_string()],
                enable_platform: vec!["claude-code".to_string()],
                ..Default::default()
            },
        );
        let content = serde_json::to_string(&raw_map).unwrap();
        create_dummy_file(&mcp_path, &content);

        // 同步
        service.sync_claude_servers().await.unwrap();

        // 验证：带占位符的 server 不应该被同步到 Claude
        let claude_content = fs::read_to_string(&claude_path).await.unwrap();
        let claude_json: serde_json::Value = serde_json::from_str(&claude_content).unwrap();
        let servers = claude_json["mcpServers"].as_object().unwrap();

        // 不应该包含该 server
        assert!(!servers.contains_key("server-with-placeholder"));
    }

    /// 测试 HTTP 类型 server
    #[tokio::test]
    async fn test_http_server_type() {
        let dir = tempdir().unwrap();
        let mcp_path = dir.path().join("mcp_servers.json");
        let claude_path = dir.path().join("claude_config.json");
        let codex_path = dir.path().join("codex_config.toml");

        let service = MCPService::with_paths(mcp_path.clone(), claude_path, codex_path);

        // 创建 HTTP 类型 server
        let servers = vec![MCPServer {
            name: "http-server".to_string(),
            server_type: MCPServerType::Http,
            command: "".to_string(),
            args: vec![],
            env: HashMap::new(),
            url: "https://mcp.example.com/sse".to_string(),
            website: "".to_string(),
            tips: "".to_string(),
            enable_platform: vec!["codex".to_string()],
            enabled_in_claude: false,
            enabled_in_codex: true,
            missing_placeholders: vec![],
        }];

        service.save_servers(servers).await.unwrap();

        let loaded = service.list_servers().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].server_type, MCPServerType::Http);
        assert_eq!(loaded[0].url, "https://mcp.example.com/sse");
    }
}
