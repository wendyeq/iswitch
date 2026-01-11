//! [INPUT]:
//!   source: ./router.rs ([POS]: Router Factory)
//!   source: ./health.rs ([POS]: Health Tracker)
//!   source: ../services/provider_service.rs ([POS]: Service Injection)
//!   source: ../models/settings.rs ([POS]: AppSettings 配置)
//!
//! [OUTPUT]:
//!   - start_proxy_server
//!
//! [POS]: Proxy server lifecycle entry point
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::proxy::health::ProviderHealthTracker;
use crate::proxy::{handler::ProxyState, router};
use crate::services::provider_service::ProviderService;
use chrono::Duration;
use sqlx::{Pool, Sqlite};
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::process::Command;
use tracing::{error, info, warn};

/// 默认代理端口
pub const DEFAULT_PROXY_PORT: u16 = 18099;
/// 默认故障转移阈值
pub const DEFAULT_FAILOVER_THRESHOLD: u32 = 5;
/// 默认恢复超时时间（秒）
pub const DEFAULT_RECOVERY_TIMEOUT_SECS: u64 = 300;

/// 占用端口的进程信息
#[derive(Debug, Clone)]
pub struct BlockerProcessInfo {
    pub name: Option<String>,
    pub pid: Option<u32>,
}

/// 端口冲突详情
#[derive(Debug, Clone)]
pub struct PortConflictInfo {
    pub port: u16,
    pub blocker: Option<BlockerProcessInfo>,
}

/// 健康追踪器配置
pub struct HealthTrackerConfig {
    /// 连续失败多少次后降级
    pub threshold: u32,
    /// 降级后多久自动尝试恢复（秒）
    pub recovery_timeout_secs: u64,
}

impl Default for HealthTrackerConfig {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_FAILOVER_THRESHOLD,
            recovery_timeout_secs: DEFAULT_RECOVERY_TIMEOUT_SECS,
        }
    }
}

/// 检测端口冲突，如果端口被占用则返回冲突详情
pub async fn detect_port_conflict(port: u16) -> Option<PortConflictInfo> {
    if port == 0 {
        return None;
    }

    let addr = format!("127.0.0.1:{}", port);
    match TcpListener::bind(&addr).await {
        Ok(listener) => {
            drop(listener);
            None
        }
        Err(e) => {
            if e.kind() != ErrorKind::AddrInUse {
                warn!(port, error = %e, "端口检测失败");
                return None;
            }

            let blocker = resolve_blocking_process(port).await;
            if let Some(info) = &blocker {
                warn!(
                    port,
                    blocker_pid = info.pid.unwrap_or_default(),
                    blocker_name = info.name.as_deref().unwrap_or("unknown"),
                    "代理端口被其他进程占用"
                );
            } else {
                warn!(port, "代理端口被占用，但无法识别占用进程");
            }

            Some(PortConflictInfo { port, blocker })
        }
    }
}

/// 启动代理服务器 (后台运行)
///
/// 返回绑定后的端口号。如果端口为 0，则由系统分配随机端口。
///
/// # 参数
/// - `provider_service`: 供应商服务
/// - `pool`: 数据库连接池
/// - `port`: 监听端口
/// - `health_config`: 健康追踪器配置（可选，默认使用 threshold=3, recovery=300s）
pub async fn start_proxy_server(
    provider_service: Arc<ProviderService>,
    pool: Arc<Pool<Sqlite>>,
    port: u16,
    health_config: Option<HealthTrackerConfig>,
) -> std::io::Result<u16> {
    // 创建健康追踪器（使用提供的配置或默认配置）
    let config = health_config.unwrap_or_default();
    let health_tracker = Arc::new(ProviderHealthTracker::new(
        config.threshold,
        Duration::seconds(config.recovery_timeout_secs as i64),
    ));

    info!(
        threshold = config.threshold,
        recovery_timeout_secs = config.recovery_timeout_secs,
        "健康追踪器配置"
    );

    // 构建应用状态
    let state = ProxyState {
        provider_service,
        pool,
        http_client: reqwest::Client::new(),
        health_tracker,
    };

    // 创建路由
    let app = router::create_router(state);

    // 绑定端口
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    let bound_port = listener.local_addr()?.port();

    info!("代理服务器正在启动, 监听: 127.0.0.1:{}", bound_port);

    // 异步启动服务器
    tokio::spawn(async move {
        info!("代理服务器启动成功: 127.0.0.1:{}", bound_port);
        if let Err(e) = axum::serve(listener, app).await {
            error!("代理服务器运行时发生错误: {}", e);
        }
    });

    Ok(bound_port)
}

#[cfg(target_family = "unix")]
async fn resolve_blocking_process(port: u16) -> Option<BlockerProcessInfo> {
    let port_flag = format!("TCP:{}", port);
    let output = Command::new("lsof")
        .args(["-nP", "-i", &port_flag, "-sTCP:LISTEN"])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    lines.next()?; // 跳过表头
    let line = lines.find(|line| !line.trim().is_empty())?;
    let mut parts = line.split_whitespace();

    let name = parts.next().map(|s| s.to_string());
    let pid = parts.next().and_then(|pid| pid.parse::<u32>().ok());

    Some(BlockerProcessInfo { name, pid })
}

#[cfg(target_family = "windows")]
async fn resolve_blocking_process(port: u16) -> Option<BlockerProcessInfo> {
    let output = Command::new("netstat")
        .args(["-ano", "-p", "TCP"])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let port_suffix = format!(":{}", port);
    let mut blocker_pid: Option<u32> = None;

    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("TCP") {
            continue;
        }
        let cols: Vec<_> = trimmed.split_whitespace().collect();
        if cols.len() < 5 {
            continue;
        }
        if !cols[1].ends_with(&port_suffix) {
            continue;
        }
        if !cols[3].eq_ignore_ascii_case("LISTENING") {
            continue;
        }
        blocker_pid = cols.last().and_then(|pid| pid.parse::<u32>().ok());
        break;
    }

    let pid = blocker_pid?;
    let task_output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
        .output()
        .await
        .ok()?;

    if !task_output.status.success() {
        return Some(BlockerProcessInfo {
            name: None,
            pid: Some(pid),
        });
    }

    let stdout = String::from_utf8_lossy(&task_output.stdout);
    let name = stdout.lines().next().and_then(|line| {
        let trimmed = line.trim_matches('\r').trim_matches('\n').trim_matches('"');
        if trimmed.is_empty() || trimmed.contains("No tasks") {
            None
        } else {
            Some(trimmed.split("\",\"").next().unwrap_or(trimmed).to_string())
        }
    });

    Some(BlockerProcessInfo {
        name,
        pid: Some(pid),
    })
}

#[cfg(not(any(target_family = "unix", target_family = "windows")))]
async fn resolve_blocking_process(_port: u16) -> Option<BlockerProcessInfo> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_port_conflict_none_when_free() {
        let listener = match TcpListener::bind("127.0.0.1:0").await {
            Ok(listener) => listener,
            Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                eprintln!("Skipping test_detect_port_conflict_none_when_free: {e}");
                return;
            }
            Err(e) => panic!("failed to bind temp port: {e}"),
        };
        let port = listener.local_addr().unwrap().port();
        drop(listener); // release before detection
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let conflict = detect_port_conflict(port).await;
        assert!(
            conflict.is_none(),
            "Port {port} should be available but reported conflict: {conflict:?}"
        );
    }

    #[tokio::test]
    async fn test_detect_port_conflict_detects_in_use_port() {
        let listener = match TcpListener::bind("127.0.0.1:0").await {
            Ok(listener) => listener,
            Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                eprintln!("Skipping test_detect_port_conflict_detects_in_use_port: {e}");
                return;
            }
            Err(e) => panic!("failed to bind temp port: {e}"),
        };
        let port = listener.local_addr().unwrap().port();

        let conflict = detect_port_conflict(port).await;
        assert!(conflict.is_some(), "Expected conflict on port {port}");

        drop(listener);
    }
}
