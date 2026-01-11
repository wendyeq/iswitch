//! [INPUT]:
//!   source: ../../../../code-switch/services/mcpservice.go ([POS]: 原 Go MCP 数据模型)
//!   source: ../../../openspec/changes/migrate-to-tauri-stack/design.md ([POS]: Rust 数据结构规范)
//!
//! [OUTPUT]:
//!   - MCPServer 结构体
//!   - MCPServerType 枚举
//!   - 相关辅助类型
//!
//! [POS]: MCP Server 数据模型定义，用于管理 Model Context Protocol 服务器配置
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Server 类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MCPServerType {
    /// 标准 I/O 类型（命令行进程）
    #[default]
    Stdio,
    /// HTTP 类型（远程服务）
    Http,
}

impl MCPServerType {
    /// 从字符串解析 MCPServerType
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "http" => Self::Http,
            _ => Self::Stdio,
        }
    }
}

impl std::fmt::Display for MCPServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdio => write!(f, "stdio"),
            Self::Http => write!(f, "http"),
        }
    }
}

/// MCP Server 平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MCPPlatform {
    /// Claude Code 平台
    #[serde(rename = "claude-code")]
    ClaudeCode,
    /// Codex 平台
    Codex,
}

impl MCPPlatform {
    /// 从字符串解析平台
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" | "claude_code" | "claude-code" => Some(Self::ClaudeCode),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }
}

impl std::fmt::Display for MCPPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClaudeCode => write!(f, "claude-code"),
            Self::Codex => write!(f, "codex"),
        }
    }
}

/// MCP Server 配置
///
/// 对应 Go 版本的 MCPServer 结构体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MCPServer {
    /// 服务器名称
    pub name: String,

    /// 服务器类型 (stdio/http)
    #[serde(rename = "type")]
    pub server_type: MCPServerType,

    /// 命令（stdio 类型使用）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,

    /// 命令参数（stdio 类型使用）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    /// 环境变量
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// 服务 URL（http 类型使用）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,

    /// 官方网站
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub website: String,

    /// 使用提示
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tips: String,

    /// 启用的平台列表
    #[serde(default)]
    pub enable_platform: Vec<String>,

    /// 是否在 Claude 中启用
    #[serde(default)]
    pub enabled_in_claude: bool,

    /// 是否在 Codex 中启用
    #[serde(default)]
    pub enabled_in_codex: bool,

    /// 未填充的占位符列表
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_placeholders: Vec<String>,
}

/// 原始 MCP Server 配置（用于文件存储）
///
/// 对应 Go 版本的 rawMCPServer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawMCPServer {
    /// 服务器类型
    #[serde(rename = "type")]
    pub server_type: MCPServerType,

    /// 命令
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,

    /// 命令参数
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    /// 环境变量
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// 服务 URL
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,

    /// 官方网站
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub website: String,

    /// 使用提示
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tips: String,

    /// 启用的平台列表
    #[serde(default)]
    pub enable_platform: Vec<String>,
}

/// Claude Desktop MCP Server 配置格式
///
/// 用于写入 ~/.claude.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeDesktopServer {
    #[serde(rename = "type", default, skip_serializing_if = "String::is_empty")]
    pub server_type: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
}

impl MCPServer {
    /// 检查是否启用了指定平台
    pub fn is_platform_enabled(&self, platform: &str) -> bool {
        self.enable_platform
            .iter()
            .any(|p| p.eq_ignore_ascii_case(platform))
    }

    /// 转换为 Claude Desktop 格式
    pub fn to_claude_desktop(&self) -> ClaudeDesktopServer {
        ClaudeDesktopServer {
            server_type: self.server_type.to_string(),
            command: if self.server_type == MCPServerType::Stdio {
                self.command.clone()
            } else {
                String::new()
            },
            args: if self.server_type == MCPServerType::Stdio {
                self.args.clone()
            } else {
                Vec::new()
            },
            env: if self.server_type == MCPServerType::Stdio {
                self.env.clone()
            } else {
                HashMap::new()
            },
            url: if self.server_type == MCPServerType::Http {
                self.url.clone()
            } else {
                String::new()
            },
        }
    }
}

/// 内置 MCP Server 配置
///
/// 提供一些预定义的 MCP Server 模板
pub fn builtin_servers() -> HashMap<String, RawMCPServer> {
    let mut servers = HashMap::new();

    servers.insert(
        "reftools".to_string(),
        RawMCPServer {
            server_type: MCPServerType::Http,
            url: "https://api.ref.tools/mcp?apiKey={apiKey}".to_string(),
            website: "https://ref.tools".to_string(),
            tips: "Visit ref.tools to claim your API key.".to_string(),
            ..Default::default()
        },
    );

    servers.insert(
        "chrome-devtools".to_string(),
        RawMCPServer {
            server_type: MCPServerType::Stdio,
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "chrome-devtools-mcp@latest".to_string()],
            tips: "Needs Node.js. Run once to install dependencies.".to_string(),
            ..Default::default()
        },
    );

    servers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_type() {
        assert_eq!(MCPServerType::from_str("http"), MCPServerType::Http);
        assert_eq!(MCPServerType::from_str("HTTP"), MCPServerType::Http);
        assert_eq!(MCPServerType::from_str("stdio"), MCPServerType::Stdio);
        assert_eq!(MCPServerType::from_str("unknown"), MCPServerType::Stdio);
    }

    #[test]
    fn test_mcp_platform() {
        assert_eq!(
            MCPPlatform::from_str("claude"),
            Some(MCPPlatform::ClaudeCode)
        );
        assert_eq!(
            MCPPlatform::from_str("claude-code"),
            Some(MCPPlatform::ClaudeCode)
        );
        assert_eq!(MCPPlatform::from_str("codex"), Some(MCPPlatform::Codex));
        assert_eq!(MCPPlatform::from_str("unknown"), None);
    }

    #[test]
    fn test_mcp_server_json() {
        let json = r#"{
            "name": "test-server",
            "type": "stdio",
            "command": "npx",
            "args": ["-y", "test-mcp"],
            "enable_platform": ["claude-code"]
        }"#;

        let server: MCPServer = serde_json::from_str(json).unwrap();
        assert_eq!(server.name, "test-server");
        assert_eq!(server.server_type, MCPServerType::Stdio);
        assert_eq!(server.command, "npx");
        assert!(server.is_platform_enabled("claude-code"));
    }

    #[test]
    fn test_builtin_servers() {
        let servers = builtin_servers();
        assert!(servers.contains_key("reftools"));
        assert!(servers.contains_key("chrome-devtools"));

        let reftools = &servers["reftools"];
        assert_eq!(reftools.server_type, MCPServerType::Http);
        assert!(reftools.url.contains("ref.tools"));
    }
}
