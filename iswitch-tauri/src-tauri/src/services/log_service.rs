//! [INPUT]:
//!   source: ../../../../code-switch/services/logservice.go ([POS]: 原 Go 实现参考)
//!   source: ../models/log.rs ([POS]: Log 数据模型)
//!
//! [OUTPUT]:
//!   - LogService 结构体
//!   - list_request_logs(), stats 相关 API
//!
//! [POS]: 请求日志记录和统计服务
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::AppResult;
use crate::models::{HeatmapStat, LogStats, ProviderDailyStat, RequestLog};

/// 日志服务
#[derive(Clone)]
pub struct LogService {
    pool: std::sync::Arc<sqlx::Pool<sqlx::Sqlite>>,
}

impl LogService {
    pub fn new(pool: std::sync::Arc<sqlx::Pool<sqlx::Sqlite>>) -> Self {
        Self { pool }
    }

    /// 列出请求日志
    pub async fn list_request_logs(
        &self,
        platform: Option<String>,
        provider: Option<String>,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<RequestLog>, i64)> {
        use crate::db::request_log;
        request_log::list_logs(&self.pool, platform, provider, page, page_size).await
    }

    /// 列出已记录的供应商
    pub async fn list_providers(&self, platform: Option<String>) -> AppResult<Vec<String>> {
        use crate::db::request_log;
        request_log::list_providers(&self.pool, platform).await
    }

    /// 热力图统计
    pub async fn heatmap_stats(&self) -> AppResult<Vec<HeatmapStat>> {
        use crate::db::request_log;
        request_log::get_heatmap_stats(&self.pool).await
    }

    /// 汇总统计
    pub async fn stats_since(
        &self,
        platform: Option<String>,
        provider: Option<String>,
        days: i64,
    ) -> AppResult<LogStats> {
        use crate::db::request_log;
        request_log::get_log_stats(&self.pool, platform, provider, days).await
    }

    /// 供应商每日统计
    pub async fn provider_daily_stats(&self, days: i64) -> AppResult<Vec<ProviderDailyStat>> {
        use crate::db::request_log;
        request_log::get_provider_daily_stats(&self.pool, days).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::request_log;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use std::sync::Arc;

    async fn build_service() -> (LogService, Arc<SqlitePool>) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create in-memory sqlite pool");

        request_log::ensure_table(&pool)
            .await
            .expect("failed to create tables");

        let arc_pool = Arc::new(pool);
        (LogService::new(arc_pool.clone()), arc_pool)
    }

    fn sample_log(platform: &str, provider: &str, model: &str, http_code: i32) -> RequestLog {
        RequestLog {
            platform: platform.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            http_code,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_list_request_logs_returns_data_and_count() {
        let (service, pool) = build_service().await;

        // Insert two logs
        request_log::insert_log(
            pool.as_ref(),
            &sample_log("claude", "provider-a", "model-1", 200),
        )
        .await
        .unwrap();
        request_log::insert_log(
            pool.as_ref(),
            &sample_log("codex", "provider-b", "model-2", 500),
        )
        .await
        .unwrap();

        let (logs, total) = service
            .list_request_logs(None, None, 1, 10)
            .await
            .expect("should fetch logs");

        assert_eq!(total, 2);
        assert_eq!(logs.len(), 2);
        let models: Vec<_> = logs.iter().map(|l| l.model.as_str()).collect();
        assert!(models.contains(&"model-1"));
        assert!(models.contains(&"model-2"));

        // Filtering by platform should reduce count
        let (claude_logs, claude_total) = service
            .list_request_logs(Some("claude".to_string()), None, 1, 10)
            .await
            .expect("should fetch claude logs");
        assert_eq!(claude_total, 1);
        assert_eq!(claude_logs[0].platform, "claude");
    }

    #[tokio::test]
    async fn test_list_providers_returns_unique_sorted_names() {
        let (service, pool) = build_service().await;

        request_log::insert_log(
            pool.as_ref(),
            &sample_log("claude", "c-provider", "model", 200),
        )
        .await
        .unwrap();
        request_log::insert_log(
            pool.as_ref(),
            &sample_log("claude", "a-provider", "model", 200),
        )
        .await
        .unwrap();
        request_log::insert_log(
            pool.as_ref(),
            &sample_log("claude", "a-provider", "model", 200),
        )
        .await
        .unwrap(); // duplicate, should be deduped

        let providers = service.list_providers(None).await.unwrap();
        assert_eq!(
            providers,
            vec!["a-provider".to_string(), "c-provider".to_string()]
        );
    }

    #[tokio::test]
    async fn test_heatmap_stats_exposes_aggregated_counts() {
        let (service, pool) = build_service().await;

        for i in 1..=3 {
            let mut log = sample_log("claude", "provider", "model", 200);
            log.input_tokens = i * 10;
            log.output_tokens = i * 5;
            log.total_cost = i as f64 * 0.1;
            request_log::insert_log(pool.as_ref(), &log).await.unwrap();
        }

        let stats = service.heatmap_stats().await.unwrap();
        assert!(!stats.is_empty());
        let first = &stats[0];
        assert_eq!(first.total_requests, 3);
        assert_eq!(first.input_tokens, 60);
        assert_eq!(first.output_tokens, 30);
        assert!((first.total_cost - 0.6).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_stats_since_returns_expected_totals() {
        let (service, pool) = build_service().await;

        let mut log_a = sample_log("claude", "p1", "model", 200);
        log_a.input_tokens = 100;
        log_a.output_tokens = 50;
        log_a.total_cost = 1.0;
        request_log::insert_log(pool.as_ref(), &log_a)
            .await
            .unwrap();

        let mut log_b = sample_log("codex", "p2", "model", 502);
        log_b.input_tokens = 40;
        log_b.output_tokens = 20;
        log_b.total_cost = 0.5;
        request_log::insert_log(pool.as_ref(), &log_b)
            .await
            .unwrap();

        let stats = service.stats_since(None, None, 30).await.unwrap();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.input_tokens, 140);
        assert_eq!(stats.output_tokens, 70);
        assert!((stats.cost_total - 1.5).abs() < f64::EPSILON);
        assert_eq!(stats.series.len(), 1);
    }

    #[tokio::test]
    async fn test_provider_daily_stats_combines_success_and_failure_counts() {
        let (service, pool) = build_service().await;

        for _ in 0..2 {
            let log = sample_log("claude", "primary", "model", 200);
            request_log::insert_log(pool.as_ref(), &log).await.unwrap();
        }

        let failed = sample_log("claude", "primary", "model", 500);
        request_log::insert_log(pool.as_ref(), &failed)
            .await
            .unwrap();

        let other = sample_log("claude", "secondary", "model", 200);
        request_log::insert_log(pool.as_ref(), &other)
            .await
            .unwrap();

        let stats = service.provider_daily_stats(7).await.unwrap();
        assert_eq!(stats.len(), 2);

        let primary = stats.iter().find(|s| s.provider == "primary").unwrap();
        assert_eq!(primary.total_requests, 3);
        assert_eq!(primary.successful_requests, 2);
        assert_eq!(primary.failed_requests, 1);
        assert!((primary.success_rate - (2.0 / 3.0)).abs() < 1e-6);

        let secondary = stats.iter().find(|s| s.provider == "secondary").unwrap();
        assert_eq!(secondary.total_requests, 1);
    }
}
