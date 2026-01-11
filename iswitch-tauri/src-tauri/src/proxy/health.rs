//! [INPUT]:
//!   source: ../models/provider.rs ([POS]: ProviderKind 枚举)
//!   source: ../../../openspec/changes/add-provider-auto-failover/design.md ([POS]: 健康追踪设计)
//!
//! [OUTPUT]:
//!   - ProviderHealthStatus
//!   - FailureReason
//!   - ProviderHealthTracker
//!
//! [POS]: 供应商健康状态追踪模块，实现基于阈值的智能降级机制
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{debug, info};

use crate::models::ProviderKind;

/// 供应商健康状态
#[derive(Debug, Clone)]
pub enum ProviderHealthStatus {
    /// 健康，可接收请求
    Healthy,
    /// 降级，暂时跳过（采用惰性恢复机制：在请求时检查是否超时，而非后台定时器）
    Degraded {
        /// 降级开始时间（用于计算是否已过 recovery_timeout）
        since: DateTime<Utc>,
        /// 连续失败次数
        failure_count: u32,
        /// 最后一次失败的错误类型
        last_error: FailureReason,
    },
}

impl ProviderHealthStatus {
    /// 检查是否为健康状态
    pub fn is_healthy(&self) -> bool {
        matches!(self, ProviderHealthStatus::Healthy)
    }

    /// 获取连续失败次数（健康状态返回 0）
    pub fn failure_count(&self) -> u32 {
        match self {
            ProviderHealthStatus::Healthy => 0,
            ProviderHealthStatus::Degraded { failure_count, .. } => *failure_count,
        }
    }
}

/// 失败原因分类
#[derive(Debug, Clone)]
pub enum FailureReason {
    /// 连接超时/失败
    ConnectionError,
    /// HTTP 5xx 服务器错误
    ServerError(u16),
    /// HTTP 429 限流
    RateLimited,
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for FailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureReason::ConnectionError => write!(f, "连接失败"),
            FailureReason::ServerError(code) => write!(f, "服务器错误 ({})", code),
            FailureReason::RateLimited => write!(f, "限流 (429)"),
            FailureReason::Other(msg) => write!(f, "其他错误: {}", msg),
        }
    }
}

/// 健康追踪器
///
/// 恢复机制说明：采用惰性检查（lazy recovery），在 is_available() 调用时检查
/// Degraded 状态是否已过 recovery_timeout，若是则自动重置为 Healthy。
/// 这避免了后台定时器的复杂性，且符合"请求驱动"的设计理念。
pub struct ProviderHealthTracker {
    /// 状态存储: (kind, provider_id) -> status
    /// - kind: ProviderKind 枚举 (Claude | Codex)
    /// - provider_id: 供应商数据库 ID
    states: RwLock<HashMap<(ProviderKind, i64), ProviderHealthStatus>>,
    /// 失败阈值：连续失败多少次后降级
    threshold: u32,
    /// 恢复超时：降级后多久自动尝试恢复（惰性检查）
    recovery_timeout: Duration,
}

impl ProviderHealthTracker {
    /// 创建新的健康追踪器
    ///
    /// # 参数
    /// - `threshold`: 连续失败多少次后降级（建议 3）
    /// - `recovery_timeout`: 降级后多久自动尝试恢复（建议 5 分钟）
    pub fn new(threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            threshold,
            recovery_timeout,
        }
    }

    /// 使用默认配置创建健康追踪器
    ///
    /// 默认值：
    /// - threshold: 3 次连续失败
    /// - recovery_timeout: 5 分钟
    pub fn with_defaults() -> Self {
        Self::new(3, Duration::seconds(300))
    }

    /// 检查供应商是否可用
    ///
    /// 惰性恢复机制：如果状态为 Degraded 且超时已过，自动重置为 Healthy
    /// 只有当失败次数 >= threshold 时才视为不可用
    pub fn is_available(&self, kind: ProviderKind, provider_id: i64, provider_name: &str) -> bool {
        let key = (kind, provider_id);

        // 读取当前状态
        let (should_recover, is_truly_degraded) = {
            let states = self.states.read().unwrap();
            match states.get(&key) {
                None | Some(ProviderHealthStatus::Healthy) => return true,
                Some(ProviderHealthStatus::Degraded {
                    since,
                    failure_count,
                    ..
                }) => {
                    // 只有达到阈值时才真正降级
                    if *failure_count < self.threshold {
                        return true; // 未达阈值，仍然可用
                    }
                    // 检查是否已过恢复超时
                    let should_recover = Utc::now() - *since > self.recovery_timeout;
                    (should_recover, true)
                }
            }
        };

        // 如果需要恢复，更新状态
        if should_recover && is_truly_degraded {
            let mut states = self.states.write().unwrap();
            // 双重检查，防止竞态条件
            if let Some(ProviderHealthStatus::Degraded {
                since,
                failure_count,
                ..
            }) = states.get(&key)
            {
                if *failure_count >= self.threshold && Utc::now() - *since > self.recovery_timeout {
                    info!(
                        "[ProviderHealth] Provider \"{}\" (id={}, kind={}) 状态变更: Degraded -> Healthy, 原因: 恢复超时到期",
                        provider_name, provider_id, kind
                    );
                    states.insert(key, ProviderHealthStatus::Healthy);
                    return true;
                }
            }
        }

        false
    }

    /// 记录请求失败
    ///
    /// 当连续失败次数达到阈值时，状态变为 Degraded
    pub fn record_failure(
        &self,
        kind: ProviderKind,
        provider_id: i64,
        provider_name: &str,
        reason: FailureReason,
    ) {
        let key = (kind, provider_id);
        let mut states = self.states.write().unwrap();

        let current_count = match states.get(&key) {
            None | Some(ProviderHealthStatus::Healthy) => 0,
            Some(ProviderHealthStatus::Degraded { failure_count, .. }) => *failure_count,
        };

        let new_count = current_count + 1;

        if new_count >= self.threshold {
            // 达到阈值，降级
            let was_healthy =
                matches!(states.get(&key), None | Some(ProviderHealthStatus::Healthy));

            states.insert(
                key,
                ProviderHealthStatus::Degraded {
                    since: Utc::now(),
                    failure_count: new_count,
                    last_error: reason.clone(),
                },
            );

            if was_healthy {
                info!(
                    "[ProviderHealth] Provider \"{}\" (id={}, kind={}) 状态变更: Healthy -> Degraded, 原因: 连续失败 {} 次 ({})",
                    provider_name, provider_id, kind, new_count, reason
                );
            } else {
                debug!(
                    "[ProviderHealth] Provider \"{}\" 连续失败 {} 次 ({})",
                    provider_name, new_count, reason
                );
            }
        } else {
            // 未达阈值，只更新计数
            states.insert(
                key,
                ProviderHealthStatus::Degraded {
                    since: Utc::now(),
                    failure_count: new_count,
                    last_error: reason,
                },
            );
            debug!(
                "[ProviderHealth] Provider \"{}\" 失败计数: {}/{}",
                provider_name, new_count, self.threshold
            );
        }
    }

    /// 记录请求成功（重置状态）
    ///
    /// 成功请求后，状态重置为 Healthy，连续失败计数清零
    pub fn record_success(&self, kind: ProviderKind, provider_id: i64, provider_name: &str) {
        let key = (kind, provider_id);
        let mut states = self.states.write().unwrap();

        let was_degraded = matches!(
            states.get(&key),
            Some(ProviderHealthStatus::Degraded { .. })
        );

        if was_degraded {
            info!(
                "[ProviderHealth] Provider \"{}\" (id={}, kind={}) 请求成功，健康状态已确认",
                provider_name, provider_id, kind
            );
        }

        states.insert(key, ProviderHealthStatus::Healthy);
    }

    /// 获取所有降级的供应商（用于日志/调试）
    pub fn get_degraded_providers(&self) -> Vec<(ProviderKind, i64, ProviderHealthStatus)> {
        let states = self.states.read().unwrap();
        states
            .iter()
            .filter_map(|((kind, id), status)| {
                if matches!(status, ProviderHealthStatus::Degraded { .. }) {
                    Some((*kind, *id, status.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// 检查是否所有供应商都处于降级状态
    ///
    /// 用于启用保底策略
    pub fn all_degraded(&self, providers: &[(ProviderKind, i64)]) -> bool {
        if providers.is_empty() {
            return false;
        }

        let states = self.states.read().unwrap();
        providers.iter().all(|(kind, id)| {
            matches!(
                states.get(&(*kind, *id)),
                Some(ProviderHealthStatus::Degraded { .. })
            )
        })
    }

    /// 获取供应商当前的连续失败次数
    pub fn get_failure_count(&self, kind: ProviderKind, provider_id: i64) -> u32 {
        let states = self.states.read().unwrap();
        match states.get(&(kind, provider_id)) {
            Some(status) => status.failure_count(),
            None => 0,
        }
    }
}

// 发送+同步安全性
unsafe impl Send for ProviderHealthTracker {}
unsafe impl Sync for ProviderHealthTracker {}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试: 连续失败 N 次后状态变为 Degraded
    #[test]
    fn test_degradation_after_threshold() {
        let tracker = ProviderHealthTracker::new(3, Duration::seconds(300));
        let kind = ProviderKind::Claude;
        let provider_id = 1;
        let name = "Test Provider";

        // 初始状态应该可用
        assert!(tracker.is_available(kind, provider_id, name));

        // 第 1 次失败
        tracker.record_failure(kind, provider_id, name, FailureReason::ConnectionError);
        assert!(tracker.is_available(kind, provider_id, name)); // 未达阈值

        // 第 2 次失败
        tracker.record_failure(kind, provider_id, name, FailureReason::ServerError(500));
        assert!(tracker.is_available(kind, provider_id, name)); // 未达阈值

        // 第 3 次失败 - 达到阈值
        tracker.record_failure(kind, provider_id, name, FailureReason::RateLimited);
        assert!(!tracker.is_available(kind, provider_id, name)); // 已降级
    }

    /// 测试: 成功请求后状态重置为 Healthy
    #[test]
    fn test_success_resets_status() {
        let tracker = ProviderHealthTracker::new(2, Duration::seconds(300));
        let kind = ProviderKind::Claude;
        let provider_id = 1;
        let name = "Test Provider";

        // 连续失败 2 次触发降级
        tracker.record_failure(kind, provider_id, name, FailureReason::ConnectionError);
        tracker.record_failure(kind, provider_id, name, FailureReason::ConnectionError);
        assert!(!tracker.is_available(kind, provider_id, name));

        // 成功请求重置状态
        tracker.record_success(kind, provider_id, name);
        assert!(tracker.is_available(kind, provider_id, name));

        // 验证计数已重置
        assert_eq!(tracker.get_failure_count(kind, provider_id), 0);
    }

    /// 测试: recovery_timeout 到期后状态重置为 Healthy
    #[test]
    fn test_recovery_timeout() {
        // 使用极短的超时以便测试
        let tracker = ProviderHealthTracker::new(1, Duration::milliseconds(1));
        let kind = ProviderKind::Codex;
        let provider_id = 2;
        let name = "Test Provider 2";

        // 触发降级
        tracker.record_failure(kind, provider_id, name, FailureReason::RateLimited);
        assert!(!tracker.is_available(kind, provider_id, name));

        // 等待超时
        std::thread::sleep(std::time::Duration::from_millis(10));

        // 应该已恢复
        assert!(tracker.is_available(kind, provider_id, name));
    }

    /// 测试: 所有供应商降级时的检测
    #[test]
    fn test_all_degraded() {
        let tracker = ProviderHealthTracker::new(1, Duration::seconds(300));
        let kind = ProviderKind::Claude;

        let providers = vec![(kind, 1), (kind, 2), (kind, 3)];

        // 初始状态：不是全部降级
        assert!(!tracker.all_degraded(&providers));

        // 降级部分
        tracker.record_failure(kind, 1, "P1", FailureReason::ConnectionError);
        tracker.record_failure(kind, 2, "P2", FailureReason::ConnectionError);
        assert!(!tracker.all_degraded(&providers));

        // 全部降级
        tracker.record_failure(kind, 3, "P3", FailureReason::ConnectionError);
        assert!(tracker.all_degraded(&providers));
    }

    /// 测试: 获取降级供应商列表
    #[test]
    fn test_get_degraded_providers() {
        let tracker = ProviderHealthTracker::new(1, Duration::seconds(300));

        tracker.record_failure(ProviderKind::Claude, 1, "P1", FailureReason::RateLimited);
        tracker.record_failure(
            ProviderKind::Codex,
            2,
            "P2",
            FailureReason::ServerError(500),
        );

        let degraded = tracker.get_degraded_providers();
        assert_eq!(degraded.len(), 2);
    }

    /// 测试: 失败原因的显示
    #[test]
    fn test_failure_reason_display() {
        assert_eq!(format!("{}", FailureReason::ConnectionError), "连接失败");
        assert_eq!(
            format!("{}", FailureReason::ServerError(503)),
            "服务器错误 (503)"
        );
        assert_eq!(format!("{}", FailureReason::RateLimited), "限流 (429)");
        assert_eq!(
            format!("{}", FailureReason::Other("unknown".to_string())),
            "其他错误: unknown"
        );
    }
}
