//! [INPUT]:
//!   source: ./.folder.md ([POS]: Proxy 模块定义)
//!
//! [OUTPUT]:
//!   - 导出代理服务器相关功能
//!
//! [POS]: HTTP 代理模块入口，聚合导出代理服务器功能
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

pub mod events;
pub mod handler;
pub mod health;
pub mod monitor;
pub mod router;
pub mod server;
// pub mod middleware; // Phase 7 安全审计时可能需要独立出来
