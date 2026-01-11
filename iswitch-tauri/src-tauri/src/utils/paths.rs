//! [INPUT]:
//!   source: ../../../code-switch/services/*.go ([POS]: 配置文件路径定义)
//!
//! [OUTPUT]:
//!   - 各种配置文件路径获取函数
//!
//! [POS]: 配置文件路径解析工具，使用独立的 ~/.iswitch/ 目录避免与旧版冲突
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use once_cell::sync::Lazy;
use std::path::PathBuf;
use tracing::debug;

/// 获取用户主目录
fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

/// iSwitch 配置目录 (~/.iswitch/)
pub fn iswitch_dir() -> PathBuf {
    let path = home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".iswitch");

    debug!(path = %path.display(), "iSwitch 配置目录");
    path
}

/// Provider 配置文件路径
///
/// # 参数
/// - `kind`: "claude" 或 "codex"
pub fn providers_config_path(kind: &str) -> PathBuf {
    iswitch_dir().join(format!("providers-{}.json", kind))
}

/// 日志数据库路径 (~/.iswitch/logs.db)
pub fn logs_db_path() -> PathBuf {
    iswitch_dir().join("logs.db")
}

/// 应用设置配置文件路径 (~/.iswitch/settings.json)
pub fn app_settings_path() -> PathBuf {
    iswitch_dir().join("settings.json")
}

/// MCP Servers 配置文件路径 (~/.iswitch/mcp-servers.json)
pub fn mcp_servers_path() -> PathBuf {
    iswitch_dir().join("mcp-servers.json")
}

/// Skill Repos 配置文件路径 (~/.iswitch/skill-repos.json)
pub fn skill_repos_path() -> PathBuf {
    iswitch_dir().join("skill-repos.json")
}

// === Claude 相关路径 ===

/// Claude 配置目录 (~/.claude/)
pub fn claude_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}

/// Claude settings.json 路径
pub fn claude_settings_path() -> PathBuf {
    claude_dir().join("settings.json")
}

/// Claude settings.json 备份路径
pub fn claude_settings_backup_path() -> PathBuf {
    claude_dir().join("settings.json.backup")
}

/// Claude MCP 配置路径 (~/.claude/claude_desktop_config.json)
pub fn claude_mcp_config_path() -> PathBuf {
    claude_dir().join("claude_desktop_config.json")
}

// === Codex 相关路径 ===

/// Codex 配置目录 (~/.codex/)
pub fn codex_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
}

/// Codex config.toml 路径
pub fn codex_config_path() -> PathBuf {
    codex_dir().join("config.toml")
}

/// Codex config.toml 备份路径
pub fn codex_config_backup_path() -> PathBuf {
    codex_dir().join("config.toml.backup")
}

/// Codex auth.json 路径
pub fn codex_auth_path() -> PathBuf {
    codex_dir().join("auth.json")
}

// === 旧版配置路径 (用于导入) ===

/// 旧版 cc-switch 配置目录 (~/.cc-switch/)
pub fn old_cc_switch_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cc-switch")
}

/// 旧版 code-switch 配置目录 (~/.code-switch/)
/// 用于从 Go 版本 code-switch 导入配置
pub fn old_code_switch_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".code-switch")
}

/// Model Pricing JSON 路径
///
/// 返回应用资源目录下的 model-pricing.json
pub static MODEL_PRICING_PATH: Lazy<PathBuf> = Lazy::new(|| {
    // 运行时从资源目录获取
    // Tauri 会在打包时处理资源文件
    PathBuf::from("resources/model-pricing.json")
});

/// 确保目录存在
pub fn ensure_dir(path: &PathBuf) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 确保 iSwitch 配置目录存在
pub fn ensure_iswitch_dir() -> std::io::Result<()> {
    ensure_dir(&iswitch_dir())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_iswitch_dir() {
        let path = iswitch_dir();
        assert!(path.to_string_lossy().contains(".iswitch"));
    }

    #[test]
    fn test_providers_config_path() {
        let claude_path = providers_config_path("claude");
        assert!(claude_path
            .to_string_lossy()
            .contains("providers-claude.json"));

        let codex_path = providers_config_path("codex");
        assert!(codex_path
            .to_string_lossy()
            .contains("providers-codex.json"));
    }

    #[test]
    fn test_claude_paths() {
        let settings = claude_settings_path();
        assert!(settings.to_string_lossy().contains(".claude/settings.json"));

        let backup = claude_settings_backup_path();
        assert!(backup.to_string_lossy().contains("settings.json.backup"));
    }

    #[test]
    fn test_codex_paths() {
        let config = codex_config_path();
        assert!(config.to_string_lossy().contains(".codex/config.toml"));
    }

    #[test]
    fn test_other_paths_and_logs() {
        assert!(logs_db_path().to_string_lossy().contains("logs.db"));
        assert!(app_settings_path()
            .to_string_lossy()
            .contains("settings.json"));
        assert!(mcp_servers_path()
            .to_string_lossy()
            .contains("mcp-servers.json"));
        assert!(skill_repos_path()
            .to_string_lossy()
            .contains("skill-repos.json"));
        assert!(old_cc_switch_dir().to_string_lossy().contains(".cc-switch"));
        assert!(old_code_switch_dir()
            .to_string_lossy()
            .contains(".code-switch"));
        assert_eq!(
            MODEL_PRICING_PATH.as_path(),
            PathBuf::from("resources/model-pricing.json")
        );
    }

    #[test]
    fn test_ensure_dir_helpers() {
        let tmp = tempdir().unwrap();
        let nested = tmp.path().join("nested").join("deep");
        ensure_dir(&nested).unwrap();
        assert!(nested.exists());

        ensure_iswitch_dir().unwrap();
        assert!(iswitch_dir().exists());
    }
}
