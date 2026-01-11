//! [INPUT]:
//!   source: ./.folder.md ([POS]: Models 模块定义)
//!
//! [OUTPUT]:
//!   - 导出所有数据模型
//!
//! [POS]: Models 模块入口，聚合导出所有数据结构定义
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

pub mod log;
pub mod mcp;
pub mod provider;
pub mod settings;
pub mod skill;

// === 重导出常用类型 ===
pub use log::{HeatmapStat, LogStats, ProviderDailyStat, RequestLog};
pub use mcp::{builtin_servers, ClaudeDesktopServer, MCPServer, MCPServerType, RawMCPServer};
pub use provider::{Provider, ProviderEnvelope, ProviderKind};
pub use settings::{AppSettings, HudSettings};
pub use skill::{
    default_skill_repos, Skill, SkillInstallRequest, SkillMetadata, SkillRepoConfig, SkillStore,
};
