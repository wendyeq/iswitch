//! [INPUT]:
//!   source: ./.folder.md ([POS]: DB 模块定义)
//!
//! [OUTPUT]:
//!   - 导出数据库操作功能
//!
//! [POS]: 数据库模块入口，聚合导出 SQLite 操作封装
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

pub mod request_log;

use crate::error::AppResult;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

pub async fn init_db(db_path: PathBuf) -> AppResult<SqlitePool> {
    info!("Initializing database at: {:?}", db_path);

    if !db_path.exists() {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::File::create(&db_path)?;
    }

    let options = SqliteConnectOptions::from_str(&format!(
        "sqlite://{}",
        db_path.to_str().unwrap_or(":memory:")
    ))
    .unwrap_or_else(|_| SqliteConnectOptions::new())
    .create_if_missing(true);

    // 从环境变量读取连接池配置，或使用默认值
    let max_connections = std::env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    let acquire_timeout_secs = std::env::var("DB_ACQUIRE_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(std::time::Duration::from_secs(acquire_timeout_secs))
        .connect_with(options)
        .await?;

    request_log::ensure_table(&pool).await?;
    info!(
        max_connections,
        acquire_timeout_secs, "Database initialized successfully"
    );

    Ok(pool)
}
