//! [INPUT]:
//!   source: ../models/log.rs ([POS]: RequestLog 模型)
//!
//! [OUTPUT]:
//!   - ensure_table()
//!   - insert_log()
//!   - list_logs()
//!   - list_providers()
//!   - get_heatmap_stats()
//!   - get_log_stats()
//!   - get_provider_daily_stats()
//!
//! [POS]: RequestLog 在 SQLite 中的 CRUD 实现
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::AppResult;
use crate::models::log::{HeatmapStat, LogStats, LogStatsSeries, ProviderDailyStat, RequestLog};
use sqlx::{Pool, Row, Sqlite};
use std::collections::HashMap;

pub async fn ensure_table(pool: &Pool<Sqlite>) -> AppResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS request_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            platform TEXT NOT NULL,
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            http_code INTEGER DEFAULT 200,
            input_tokens INTEGER DEFAULT 0,
            output_tokens INTEGER DEFAULT 0,
            reasoning_tokens INTEGER DEFAULT 0,
            cache_create_tokens INTEGER DEFAULT 0,
            cache_read_tokens INTEGER DEFAULT 0,
            total_cost REAL DEFAULT 0,
            duration_sec REAL DEFAULT 0,
            is_stream BOOLEAN DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Create index on created_at for faster stats query
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_created_at ON request_logs(created_at);")
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn insert_log(pool: &Pool<Sqlite>, log: &RequestLog) -> AppResult<i64> {
    // 如果 created_at 为空，不绑定该字段，让数据库使用 CURRENT_TIMESTAMP 默认值
    let use_default_time = log.created_at.is_empty();

    let query_str = if use_default_time {
        r#"
        INSERT INTO request_logs (
            platform, provider, model, http_code,
            input_tokens, output_tokens, reasoning_tokens,
            cache_create_tokens, cache_read_tokens,
            total_cost, duration_sec, is_stream
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    } else {
        r#"
        INSERT INTO request_logs (
            platform, provider, model, http_code,
            input_tokens, output_tokens, reasoning_tokens,
            cache_create_tokens, cache_read_tokens,
            total_cost, duration_sec, is_stream, created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    };

    let mut query = sqlx::query(query_str)
        .bind(&log.platform)
        .bind(&log.provider)
        .bind(&log.model)
        .bind(log.http_code)
        .bind(log.input_tokens)
        .bind(log.output_tokens)
        .bind(log.reasoning_tokens)
        .bind(log.cache_create_tokens)
        .bind(log.cache_read_tokens)
        .bind(log.total_cost)
        .bind(log.duration_sec)
        .bind(log.is_stream);

    if !use_default_time {
        query = query.bind(&log.created_at);
    }

    let result = query.execute(pool).await?;

    let id = result.last_insert_rowid();
    Ok(id)
}

pub async fn list_logs(
    pool: &Pool<Sqlite>,
    platform: Option<String>,
    provider: Option<String>,
    page: i64,
    page_size: i64,
) -> AppResult<(Vec<RequestLog>, i64)> {
    let offset = (page - 1) * page_size;

    let mut qb = sqlx::QueryBuilder::new("SELECT * FROM request_logs WHERE 1=1");
    if let Some(p) = &platform {
        qb.push(" AND platform = ");
        qb.push_bind(p);
    }
    if let Some(p) = &provider {
        qb.push(" AND provider = ");
        qb.push_bind(p);
    }
    qb.push(" ORDER BY created_at DESC LIMIT ");
    qb.push_bind(page_size);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let rows = qb.build().fetch_all(pool).await?;

    let logs = rows
        .iter()
        .map(|row| {
            RequestLog {
                id: row.get("id"),
                platform: row.get("platform"),
                provider: row.get("provider"),
                model: row.get("model"),
                http_code: row.get("http_code"),
                input_tokens: row.get("input_tokens"),
                output_tokens: row.get("output_tokens"),
                reasoning_tokens: row.get("reasoning_tokens"),
                cache_create_tokens: row.get("cache_create_tokens"),
                cache_read_tokens: row.get("cache_read_tokens"),
                total_cost: row.get("total_cost"),
                duration_sec: row.get("duration_sec"),
                is_stream: row.get("is_stream"),
                created_at: row.try_get::<String, _>("created_at").unwrap_or_default(), // SQLite datetime as string
                ..Default::default()
            }
        })
        .collect();

    let mut count_qb = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM request_logs WHERE 1=1");
    if let Some(p) = &platform {
        count_qb.push(" AND platform = ");
        count_qb.push_bind(p);
    }
    if let Some(p) = &provider {
        count_qb.push(" AND provider = ");
        count_qb.push_bind(p);
    }

    let count: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    Ok((logs, count))
}

pub async fn list_providers(
    pool: &Pool<Sqlite>,
    platform: Option<String>,
) -> AppResult<Vec<String>> {
    let mut qb = sqlx::QueryBuilder::new("SELECT DISTINCT provider FROM request_logs WHERE 1=1");

    if let Some(p) = &platform {
        qb.push(" AND platform = ");
        qb.push_bind(p);
    }

    qb.push(" ORDER BY provider");

    let providers: Vec<String> = qb.build_query_scalar().fetch_all(pool).await?;
    Ok(providers)
}

pub async fn get_heatmap_stats(pool: &Pool<Sqlite>) -> AppResult<Vec<HeatmapStat>> {
    let rows = sqlx::query(
        r#"
        SELECT 
            strftime('%Y-%m-%d %H', datetime(created_at, 'localtime')) as day,
            COUNT(*) as total_requests,
            SUM(input_tokens) as input_tokens,
            SUM(output_tokens) as output_tokens,
            SUM(reasoning_tokens) as reasoning_tokens,
            SUM(total_cost) as total_cost
        FROM request_logs
        GROUP BY day
        ORDER BY day DESC
        LIMIT 1000
        "#,
    )
    .fetch_all(pool)
    .await?;

    let stats = rows
        .iter()
        .map(|row| HeatmapStat {
            day: row.get::<String, _>("day"),
            total_requests: row.get::<i64, _>("total_requests"),
            input_tokens: row.try_get::<i64, _>("input_tokens").unwrap_or(0),
            output_tokens: row.try_get::<i64, _>("output_tokens").unwrap_or(0),
            reasoning_tokens: row.try_get::<i64, _>("reasoning_tokens").unwrap_or(0),
            total_cost: row.try_get::<f64, _>("total_cost").unwrap_or(0.0),
        })
        .collect();

    Ok(stats)
}

pub async fn get_log_stats(
    pool: &Pool<Sqlite>,
    platform: Option<String>,
    provider: Option<String>,
    since_days: i64,
) -> AppResult<LogStats> {
    // 1. Aggregate Totals
    let mut qb = sqlx::QueryBuilder::new(
        r#"
        SELECT
            COUNT(*) as total_requests,
            SUM(input_tokens) as input_tokens,
            SUM(output_tokens) as output_tokens,
            SUM(reasoning_tokens) as reasoning_tokens,
            SUM(cache_create_tokens) as cache_create_tokens,
            SUM(cache_read_tokens) as cache_read_tokens,
            SUM(total_cost) as total_cost
        FROM request_logs
        WHERE created_at >= datetime('now', 
        "#,
    );
    qb.push_bind(format!("-{} days", since_days));
    qb.push(")");

    if let Some(p) = &platform {
        qb.push(" AND platform = ");
        qb.push_bind(p);
    }
    if let Some(p) = &provider {
        qb.push(" AND provider = ");
        qb.push_bind(p);
    }

    let row = qb.build().fetch_one(pool).await?;

    let mut stats = LogStats {
        total_requests: row.try_get("total_requests").unwrap_or(0),
        input_tokens: row.try_get("input_tokens").unwrap_or(0),
        output_tokens: row.try_get("output_tokens").unwrap_or(0),
        reasoning_tokens: row.try_get("reasoning_tokens").unwrap_or(0),
        cache_create_tokens: row.try_get("cache_create_tokens").unwrap_or(0),
        cache_read_tokens: row.try_get("cache_read_tokens").unwrap_or(0),
        cost_total: row.try_get("total_cost").unwrap_or(0.0),
        ..Default::default()
    };

    // 2. Get Series Data
    // Determine granularity: Hourly for <= 1 day, Daily otherwise
    let date_format = if since_days <= 1 {
        "%Y-%m-%d %H:00"
    } else {
        "%Y-%m-%d"
    };

    let mut series_qb = sqlx::QueryBuilder::new("SELECT strftime(");
    series_qb.push_bind(date_format);
    // Use localtime for the grouping key so it matches frontend labels (which are local)
    series_qb.push(
        r#", datetime(created_at, 'localtime')) as day,
            COUNT(*) as total_requests,
            SUM(input_tokens) as input_tokens,
            SUM(output_tokens) as output_tokens,
            SUM(reasoning_tokens) as reasoning_tokens,
            SUM(cache_create_tokens) as cache_create_tokens,
            SUM(cache_read_tokens) as cache_read_tokens,
            SUM(total_cost) as total_cost
        FROM request_logs
        WHERE created_at >= datetime('now', "#,
    );
    series_qb.push_bind(format!("-{} days", since_days));
    series_qb.push(")");

    if let Some(p) = &platform {
        series_qb.push(" AND platform = ");
        series_qb.push_bind(p);
    }
    if let Some(p) = &provider {
        series_qb.push(" AND provider = ");
        series_qb.push_bind(p);
    }

    series_qb.push(" GROUP BY day ORDER BY day");

    let series_rows = series_qb.build().fetch_all(pool).await?;

    stats.series = series_rows
        .iter()
        .map(|row| LogStatsSeries {
            day: row.get("day"),
            total_requests: row.get("total_requests"),
            input_tokens: row.try_get("input_tokens").unwrap_or(0),
            output_tokens: row.try_get("output_tokens").unwrap_or(0),
            reasoning_tokens: row.try_get("reasoning_tokens").unwrap_or(0),
            cache_create_tokens: row.try_get("cache_create_tokens").unwrap_or(0),
            cache_read_tokens: row.try_get("cache_read_tokens").unwrap_or(0),
            total_cost: row.try_get("total_cost").unwrap_or(0.0),
        })
        .collect();

    Ok(stats)
}

pub async fn get_provider_daily_stats(
    pool: &Pool<Sqlite>,
    since_days: i64,
) -> AppResult<Vec<ProviderDailyStat>> {
    // 使用验证后的参数防止SQL注入
    let since_days_str = since_days.to_string();

    let rows = sqlx::query(&format!(
        r#"
        SELECT
            provider,
            COUNT(*) as total_requests,
            SUM(CASE WHEN http_code = 200 THEN 1 ELSE 0 END) as successful_requests,
            SUM(CASE WHEN http_code != 200 THEN 1 ELSE 0 END) as failed_requests,
            SUM(input_tokens) as input_tokens,
            SUM(output_tokens) as output_tokens,
            SUM(reasoning_tokens) as reasoning_tokens,
            SUM(cache_create_tokens) as cache_create_tokens,
            SUM(cache_read_tokens) as cache_read_tokens,
            SUM(total_cost) as total_cost
        FROM request_logs
        WHERE created_at >= datetime('now', '-{} days')
        GROUP BY provider
        ORDER BY total_requests DESC
        "#,
        since_days_str
    ))
    .fetch_all(pool)
    .await?;

    // --- Step 2: Fetch Hourly Trends (Sparkline Data) ---
    // Calculate hours_ago (0..23) for valid range.
    let hourly_rows = sqlx::query(
        r#"
        SELECT 
            provider,
            CAST((julianday('now') - julianday(created_at)) * 24 AS INTEGER) as hours_ago,
            COUNT(*) as cnt
        FROM request_logs
        WHERE created_at >= datetime('now', '-24 hours')
        GROUP BY provider, hours_ago
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut trend_map: HashMap<String, Vec<i64>> = HashMap::new();

    for row in hourly_rows {
        let provider: String = row.get("provider");
        let hours_ago: i64 = row.get("hours_ago");
        let count: i64 = row.get("cnt");

        let vec = trend_map.entry(provider).or_insert_with(|| vec![0; 24]);

        // Ensure index is within bounds (0..23)
        if hours_ago >= 0 && hours_ago < 24 {
            // Index logic:
            // hours_ago = 0  => Index 23 (Present)
            // hours_ago = 23 => Index 0  (Past)
            let idx = 23 - hours_ago as usize;
            if idx < 24 {
                vec[idx] = count;
            }
        }
    }

    let stats = rows
        .iter()
        .map(|row| {
            let total: i64 = row.get("total_requests");
            let success: i64 = row.get("successful_requests");
            let rate = if total > 0 {
                success as f64 / total as f64
            } else {
                0.0
            };

            ProviderDailyStat {
                provider: row.get("provider"),
                total_requests: total,
                successful_requests: success,
                failed_requests: row.get("failed_requests"),
                success_rate: rate,
                input_tokens: row.try_get("input_tokens").unwrap_or(0),
                output_tokens: row.try_get("output_tokens").unwrap_or(0),
                reasoning_tokens: row.try_get("reasoning_tokens").unwrap_or(0),
                cache_create_tokens: row.try_get("cache_create_tokens").unwrap_or(0),
                cache_read_tokens: row.try_get("cache_read_tokens").unwrap_or(0),
                cost_total: row.try_get("total_cost").unwrap_or(0.0),
                hourly_requests: trend_map
                    .get(&row.get::<String, _>("provider"))
                    .cloned()
                    .unwrap_or_else(|| vec![0; 24]),
            }
        })
        .collect();

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_db() -> Pool<Sqlite> {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        ensure_table(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_log_crud() {
        let pool = setup_db().await;
        let mut log = RequestLog::new("claude", "provider1", "claude-3-opus", false);
        log.input_tokens = 100;
        log.total_cost = 0.5;
        let id = insert_log(&pool, &log).await.unwrap();
        assert!(id > 0);

        let (logs, count) = list_logs(&pool, None, None, 1, 10).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].provider, "provider1");
        assert_eq!(logs[0].input_tokens, 100);
    }

    #[tokio::test]
    async fn test_stats() {
        let pool = setup_db().await;
        // Insert logs for different days/providers
        let mut log1 = RequestLog::new("claude", "p1", "m1", false);
        log1.total_cost = 1.0;
        insert_log(&pool, &log1).await.unwrap();

        let mut log2 = RequestLog::new("codex", "p2", "m2", false);
        log2.total_cost = 2.0;
        insert_log(&pool, &log2).await.unwrap();

        let stats = get_log_stats(&pool, None, None, 30).await.unwrap();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cost_total, 3.0);

        let daily = get_provider_daily_stats(&pool, 30).await.unwrap();
        assert!(daily.len() >= 2);
    }

    // ===== 边界值测试 =====

    /// 测试 since_days = 0 的情况
    /// 预期：只返回今天的数据
    #[tokio::test]
    async fn test_stats_since_zero_days() {
        let pool = setup_db().await;

        let mut log = RequestLog::new("claude", "test", "model", false);
        log.total_cost = 1.5;
        insert_log(&pool, &log).await.unwrap();

        // since_days = 0 应该只查询今天的数据
        let stats = get_log_stats(&pool, None, None, 0).await.unwrap();
        // 今天插入的数据应该被包含
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cost_total, 1.5);
    }

    /// 测试 since_days 为负数的情况
    /// 预期：不应 panic，返回空结果
    #[tokio::test]
    async fn test_stats_since_negative_days() {
        let pool = setup_db().await;

        let mut log = RequestLog::new("claude", "test", "model", false);
        log.total_cost = 1.0;
        insert_log(&pool, &log).await.unwrap();

        // 负数天数：查询"未来"，应该返回空结果
        let stats = get_log_stats(&pool, None, None, -1).await.unwrap();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.cost_total, 0.0);

        let daily = get_provider_daily_stats(&pool, -1).await.unwrap();
        assert!(daily.is_empty());
    }

    /// 测试 since_days 为大整数的情况
    /// 预期：不应溢出，返回所有数据
    #[tokio::test]
    async fn test_stats_since_large_days() {
        let pool = setup_db().await;

        let mut log = RequestLog::new("claude", "test", "model", false);
        log.total_cost = 2.5;
        insert_log(&pool, &log).await.unwrap();

        // 非常大的天数：应该包含所有历史数据
        let stats = get_log_stats(&pool, None, None, 999999).await.unwrap();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cost_total, 2.5);

        let daily = get_provider_daily_stats(&pool, 999999).await.unwrap();
        assert_eq!(daily.len(), 1);
    }

    /// 测试空数据库的统计查询
    /// 预期：返回零值，不抛错
    #[tokio::test]
    async fn test_stats_empty_database() {
        let pool = setup_db().await;

        // 空数据库应该返回零值
        let stats = get_log_stats(&pool, None, None, 30).await.unwrap();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.cost_total, 0.0);
        assert!(stats.series.is_empty());

        let daily = get_provider_daily_stats(&pool, 30).await.unwrap();
        assert!(daily.is_empty());

        let heatmap = get_heatmap_stats(&pool).await.unwrap();
        assert!(heatmap.is_empty());
    }

    /// 测试分页边界条件
    /// 预期：page=0 或 page_size=0 不应 panic
    #[tokio::test]
    async fn test_list_logs_pagination_boundary() {
        let pool = setup_db().await;

        // 插入 3 条日志
        for i in 0..3 {
            let mut log = RequestLog::new("claude", &format!("p{}", i), "model", false);
            log.total_cost = (i + 1) as f64;
            insert_log(&pool, &log).await.unwrap();
        }

        // page = 0 (边界情况，offset 为负数理论上)
        // 实际 offset = (0-1) * page_size = -page_size
        // SQLite 的 OFFSET 负数行为取决于具体实现
        // 但代码应该不 panic
        let result = list_logs(&pool, None, None, 0, 10).await;
        assert!(result.is_ok());

        // page_size = 0 (边界情况)
        let (logs, count) = list_logs(&pool, None, None, 1, 0).await.unwrap();
        assert_eq!(count, 3); // count 应该正确
        assert!(logs.is_empty()); // LIMIT 0 返回空

        // 正常分页
        let (logs, count) = list_logs(&pool, None, None, 1, 2).await.unwrap();
        assert_eq!(count, 3);
        assert_eq!(logs.len(), 2);

        // 第二页
        let (logs, _) = list_logs(&pool, None, None, 2, 2).await.unwrap();
        assert_eq!(logs.len(), 1);

        // 超出范围的页码
        let (logs, _) = list_logs(&pool, None, None, 100, 10).await.unwrap();
        assert!(logs.is_empty());
    }

    /// 测试 list_providers 返回去重结果
    #[tokio::test]
    async fn test_list_providers_distinct() {
        let pool = setup_db().await;

        // 插入多条相同 provider 的日志
        for _ in 0..3 {
            let log = RequestLog::new("claude", "same_provider", "model", false);
            insert_log(&pool, &log).await.unwrap();
        }

        // 插入不同 provider
        let log2 = RequestLog::new("codex", "another_provider", "model", false);
        insert_log(&pool, &log2).await.unwrap();

        let providers = list_providers(&pool, None).await.unwrap();
        assert_eq!(providers.len(), 2);
        assert!(providers.contains(&"same_provider".to_string()));
        assert!(providers.contains(&"another_provider".to_string()));

        // Test filtering
        let claude_providers = list_providers(&pool, Some("claude".to_string()))
            .await
            .unwrap();
        assert_eq!(claude_providers.len(), 1);
        assert_eq!(claude_providers[0], "same_provider");
    }

    /// 测试 heatmap 统计的小时粒度
    #[tokio::test]
    async fn test_heatmap_stats_hourly_granularity() {
        let pool = setup_db().await;

        // 插入多条日志
        for i in 0..5 {
            let mut log = RequestLog::new("claude", "p1", "model", false);
            log.input_tokens = (i + 1) * 100;
            log.output_tokens = (i + 1) * 50;
            log.total_cost = (i + 1) as f64 * 0.1;
            insert_log(&pool, &log).await.unwrap();
        }

        let stats = get_heatmap_stats(&pool).await.unwrap();
        assert!(!stats.is_empty());

        // 验证聚合结果
        let first = &stats[0];
        // 5 条日志应该聚合在同一小时
        assert_eq!(first.total_requests, 5);
        // input_tokens: 100+200+300+400+500 = 1500
        assert_eq!(first.input_tokens, 1500);
        // output_tokens: 50+100+150+200+250 = 750
        assert_eq!(first.output_tokens, 750);
        // total_cost: 0.1+0.2+0.3+0.4+0.5 = 1.5
        assert!((first.total_cost - 1.5).abs() < 0.001);
    }

    /// 测试 Provider 日统计的成功率计算
    #[tokio::test]
    async fn test_provider_daily_stats_success_rate() {
        let pool = setup_db().await;

        // 插入 3 条成功日志
        for _ in 0..3 {
            let mut log = RequestLog::new("claude", "test_provider", "model", false);
            log.http_code = 200;
            insert_log(&pool, &log).await.unwrap();
        }

        // 插入 2 条失败日志
        for _ in 0..2 {
            let mut log = RequestLog::new("claude", "test_provider", "model", false);
            log.http_code = 500;
            insert_log(&pool, &log).await.unwrap();
        }

        let daily = get_provider_daily_stats(&pool, 30).await.unwrap();
        assert_eq!(daily.len(), 1);

        let stat = &daily[0];
        assert_eq!(stat.provider, "test_provider");
        assert_eq!(stat.total_requests, 5);
        assert_eq!(stat.successful_requests, 3);
        assert_eq!(stat.failed_requests, 2);
        // 成功率: 3/5 = 0.6
        assert!((stat.success_rate - 0.6).abs() < 0.001);
    }

    /// 测试带有 created_at 的日志插入
    #[tokio::test]
    async fn test_insert_log_with_custom_created_at() {
        let pool = setup_db().await;

        let mut log = RequestLog::new("claude", "test", "model", false);
        log.created_at = "2025-12-01 10:00:00".to_string();
        log.total_cost = 1.0;

        let id = insert_log(&pool, &log).await.unwrap();
        assert!(id > 0);

        // 验证插入的日期
        let (logs, _) = list_logs(&pool, None, None, 1, 10).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].created_at.contains("2025-12-01"));
    }

    /// 测试空 created_at 使用默认值
    #[tokio::test]
    async fn test_insert_log_with_empty_created_at() {
        let pool = setup_db().await;

        let mut log = RequestLog::new("claude", "test", "model", false);
        log.created_at = String::new(); // 空字符串
        log.total_cost = 1.0;

        let id = insert_log(&pool, &log).await.unwrap();
        assert!(id > 0);

        // 验证使用了默认时间
        let (logs, _) = list_logs(&pool, None, None, 1, 10).await.unwrap();
        assert_eq!(logs.len(), 1);
        // created_at 不应该为空
        assert!(!logs[0].created_at.is_empty());
    }
}
