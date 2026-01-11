//! [INPUT]:
//!   source: ../services/log_service.rs ([POS]: Log 服务)
//!   source: ../models/log.rs ([POS]: Log 数据模型)
//!
//! [OUTPUT]:
//!   - list_request_logs, list_log_providers commands
//!   - get_heatmap_stats, get_log_stats, get_provider_daily_stats commands
//!
//! [POS]: 日志相关的 Tauri Commands
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::AppResult;
use crate::models::{HeatmapStat, LogStats, ProviderDailyStat, RequestLog};

use crate::services::LogService;
use tauri::State;

/// 列出请求日志
#[tauri::command]
pub async fn list_request_logs(
    state: State<'_, LogService>,
    platform: Option<String>,
    provider: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
    limit: Option<i64>,
) -> AppResult<Vec<RequestLog>> {
    let p = page.unwrap_or(1);
    let s = page_size.or(limit).unwrap_or(50);
    // Treat empty string as None (no filter)
    let platform_filter = platform.filter(|s| !s.is_empty());
    let provider_filter = provider.filter(|s| !s.is_empty());
    let (logs, _) = state
        .list_request_logs(platform_filter, provider_filter, p, s)
        .await?;
    Ok(logs)
}

/// 列出日志中的供应商
#[tauri::command]
pub async fn list_log_providers(
    state: State<'_, LogService>,
    platform: Option<String>,
) -> AppResult<Vec<String>> {
    let platform_filter = platform.filter(|s| !s.is_empty());
    state.list_providers(platform_filter).await
}

/// 获取热力图统计
#[tauri::command]
pub async fn get_heatmap_stats(state: State<'_, LogService>) -> AppResult<Vec<HeatmapStat>> {
    state.heatmap_stats().await
}

/// 获取日志统计
#[tauri::command]
pub async fn get_log_stats(
    state: State<'_, LogService>,
    platform: Option<String>,
    provider: Option<String>,
    days: Option<i64>,
) -> AppResult<LogStats> {
    let platform_filter = platform.filter(|s| !s.is_empty());
    let provider_filter = provider.filter(|s| !s.is_empty());
    state
        .stats_since(platform_filter, provider_filter, days.unwrap_or(30))
        .await
}

/// 获取供应商每日统计
#[tauri::command]
pub async fn get_provider_daily_stats(
    state: State<'_, LogService>,
    days: Option<i64>,
) -> AppResult<Vec<ProviderDailyStat>> {
    state.provider_daily_stats(days.unwrap_or(30)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::request_log;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use tauri::Manager;

    async fn build_service() -> (LogService, Arc<SqlitePool>) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("create sqlite pool");
        request_log::ensure_table(&pool).await.unwrap();
        let arc_pool = Arc::new(pool);
        (LogService::new(arc_pool.clone()), arc_pool)
    }

    fn sample_log(platform: &str, provider: &str, created_at: &str) -> RequestLog {
        RequestLog {
            platform: platform.to_string(),
            provider: provider.to_string(),
            model: "model".to_string(),
            created_at: created_at.to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_list_request_logs_command_respects_limit_and_filter() {
        let (service, pool) = build_service().await;
        request_log::insert_log(&pool, &sample_log("claude", "a", ""))
            .await
            .unwrap();
        request_log::insert_log(&pool, &sample_log("codex", "b", ""))
            .await
            .unwrap();

        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(service);

        let state = app.state::<LogService>();

        // Empty string platform should behave as None.
        let logs = list_request_logs(
            state.clone(),
            Some(String::new()),
            None,
            None,
            None,
            Some(1),
        )
        .await
        .unwrap();
        assert_eq!(logs.len(), 1);

        // Filtering by platform returns only matching entries.
        let claude_logs =
            list_request_logs(state, Some("claude".into()), None, Some(1), Some(10), None)
                .await
                .unwrap();
        assert_eq!(claude_logs.len(), 1);
        assert_eq!(claude_logs[0].platform, "claude");
    }

    #[tokio::test]
    async fn test_get_log_stats_command_defaults_to_30_days() {
        let (service, pool) = build_service().await;
        // Old log outside default 30 day window.
        let mut old_log = sample_log("claude", "legacy", "2020-01-01 00:00:00");
        old_log.total_cost = 1.0;
        request_log::insert_log(&pool, &old_log).await.unwrap();
        // Recent log should be counted by default.
        request_log::insert_log(&pool, &sample_log("claude", "recent", ""))
            .await
            .unwrap();

        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(service);
        let state = app.state::<LogService>();

        let stats = get_log_stats(state.clone(), None, None, None)
            .await
            .unwrap();
        assert_eq!(
            stats.total_requests, 1,
            "only recent entries should be counted"
        );

        let stats_with_range = get_log_stats(state, None, None, Some(9999)).await.unwrap();
        assert_eq!(stats_with_range.total_requests, 2);
    }

    #[tokio::test]
    async fn test_get_provider_daily_stats_command_uses_default_days() {
        let (service, pool) = build_service().await;
        request_log::insert_log(&pool, &sample_log("claude", "provider-a", ""))
            .await
            .unwrap();

        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        app.manage(service);
        let state = app.state::<LogService>();

        let stats = get_provider_daily_stats(state.clone(), None).await.unwrap();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].provider, "provider-a");

        let providers = list_log_providers(state, None).await.unwrap();
        assert_eq!(providers, vec!["provider-a".to_string()]);
    }
}
