//! [INPUT]:
//!   source: ./.folder.md ([POS]: Commands 模块定义)
//!
//! [OUTPUT]:
//!   - 导出所有 Tauri Commands
//!
//! [POS]: Tauri Commands 模块入口，聚合导出所有前后端桥接函数
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

pub mod claude;
pub mod codex;
pub mod hud;
pub mod log;
pub mod mcp;
pub mod provider;
pub mod settings;
pub mod skill;

// === 重导出 Commands ===
pub use claude::{disable_claude_proxy, enable_claude_proxy, get_claude_proxy_status};
pub use codex::{disable_codex_proxy, enable_codex_proxy, get_codex_proxy_status};
pub use log::{
    get_heatmap_stats, get_log_stats, get_provider_daily_stats, list_log_providers,
    list_request_logs,
};
pub use mcp::{list_mcp_servers, save_mcp_servers};
pub use provider::{get_proxy_status, load_providers, save_providers};
pub use settings::{
    get_app_settings, get_code_switch_import_status, get_import_status, import_config,
    import_from_code_switch, open_logs_window, save_app_settings,
};
pub use skill::{
    add_skill_repo, install_skill, list_skill_repos, list_skills, remove_skill_repo,
    uninstall_skill,
};
