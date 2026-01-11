# iSwitch 项目 Bug 扫描报告

**扫描日期**: 2026-01-06
**扫描范围**: 完整项目代码（Rust 后端 + TypeScript 前端）
**总文件数**: 60+ 源文件
**发现问题**: 11 个（高优先级 3 个，中优先级 3 个，低优先级 5 个）

---

## 📊 执行摘要

### 代码质量评分
- **整体评分**: ⭐⭐⭐⭐ (4/5)
- **安全性**: ⚠️ 需要改进
- **可维护性**: ✅ 良好
- **测试覆盖**: ✅ 较好

### 关键发现
- ✅ **优点**: 类型安全、错误处理完善、并发控制正确、文档规范
- ⚠️ **缺点**: 存在SQL注入风险、路径遍历漏洞、错误日志缺失

---

## 🔴 高优先级问题（需立即修复）

### 1. SQL注入风险
**文件**: `iswitch-tauri/src-tauri/src/db/request_log.rs`
**行号**: 182-196, 213-229
**严重程度**: 🔴 高
**类别**: 安全漏洞

#### 问题描述
使用字符串拼接构建SQL查询，存在SQL注入风险。

#### 当前代码
```rust
let start_date_expr = format!("date('now', '-{} days')", since_days);

let row = sqlx::query(&format!(
    r#"
    SELECT ...
    FROM request_logs
    WHERE datetime(created_at, 'localtime') >= {}
    "#,
    start_date_expr
))
```

#### 安全风险
- 如果 `since_days` 参数被恶意控制，可能执行任意SQL命令
- 虽然当前调用来源可信，但这是潜在的安全隐患

#### 修复方案
使用参数化查询：

```rust
pub async fn stats_since(&self, since_days: i64) -> AppResult<LogStats> {
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*) as total,
            SUM(input_tokens) as input_tokens,
            SUM(output_tokens) as output_tokens,
            SUM(cache_create_tokens) as cache_create_tokens,
            SUM(cache_read_tokens) as cache_read_tokens,
            SUM(reasoning_tokens) as reasoning_tokens,
            SUM(total_cost) as total_cost,
            AVG(duration_sec) as avg_duration
        FROM request_logs
        WHERE datetime(created_at, 'localtime') >= date('now', '-' || ? || ' days')
        "#
    )
    .bind(since_days)
    .fetch_one(self.pool())
    .await?;

    // ... 其余处理逻辑
}

pub async fn heatmap_stats(&self, since_days: i64) -> AppResult<Vec<HeatmapStat>> {
    let rows = sqlx::query(
        r#"
        SELECT
            date(created_at, 'localtime') as day,
            COUNT(*) as count
        FROM request_logs
        WHERE datetime(created_at, 'localtime') >= date('now', '-' || ? || ' days')
        GROUP BY day
        ORDER BY day
        "#
    )
    .bind(since_days)
    .fetch_all(self.pool())
    .await?;

    // ... 其余处理逻辑
}
```

#### 测试建议
- 添加单元测试验证参数化查询
- 测试边界值（负数、0、大整数）

---

### 2. 路径遍历风险
**文件**: `iswitch-tauri/src-tauri/src/commands/settings.rs`
**行号**: 167
**严重程度**: 🔴 高
**类别**: 安全漏洞

#### 问题描述
用户提供的文件路径未经验证直接使用，可能导致路径遍历攻击。

#### 当前代码
```rust
#[tauri::command]
pub async fn import_from_file(_app: AppHandle, path: String) -> AppResult<ConfigImportResult> {
    info!(path = %path, "从自定义文件导入配置");
    let import_service = ImportService::new(provider_service, mcp_service);
    import_service.import_from_file(&path).await
}
```

#### 安全风险
- 攻击者可以传入 `../../etc/passwd` 等路径读取系统文件
- 可能读取用户任意文件内容

#### 修复方案
添加严格的路径验证：

```rust
#[tauri::command]
pub async fn import_from_file(_app: AppHandle, path: String) -> AppResult<ConfigImportResult> {
    use crate::error::AppError;

    // 1. 验证路径格式
    let path_obj = std::path::Path::new(&path);

    // 2. 规范化路径，解析所有 `..` 和 `.`
    let canonical_path = path_obj
        .canonicalize()
        .map_err(|_| AppError::InvalidInput("Invalid file path".into()))?;

    // 3. 验证文件存在且是普通文件
    if !canonical_path.exists() || !canonical_path.is_file() {
        return Err(AppError::InvalidInput("File does not exist".into()));
    }

    // 4. 验证文件扩展名（只允许.json）
    if canonical_path.extension().and_then(|s| s.to_str()) != Some("json") {
        return Err(AppError::InvalidInput(
            "Only JSON files are allowed".into()
        ));
    }

    // 5. 可选：限制文件大小（防止DoS）
    let metadata = std::fs::metadata(&canonical_path)
        .map_err(|_| AppError::InvalidInput("Cannot read file metadata".into()))?;

    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    if metadata.len() > MAX_FILE_SIZE {
        return Err(AppError::InvalidInput(
            "File too large (max 10MB)".into()
        ));
    }

    info!(path = %canonical_path.display(), "从自定义文件导入配置");
    let import_service = ImportService::new(provider_service, mcp_service);
    import_service.import_from_file(&canonical_path).await
}
```

#### 测试建议
- 测试正常文件路径
- 测试包含 `..` 的路径
- 测试不存在的文件
- 测试非JSON文件
- 测试超大文件

---

### 3. 前端错误被静默吞掉
**文件**: `iswitch-tauri/src/services/tauri.ts`
**行号**: 72-76, 207-212, 246-251, 262-267
**严重程度**: 🔴 高
**类别**: 错误处理

#### 问题描述
多个函数的catch块为空或只返回默认值，不记录错误日志。

#### 当前代码
```typescript
export const fetchAppSettings = async (): Promise<AppSettings> => {
    try {
        return await invoke<AppSettings>('get_app_settings');
    } catch {
        return DEFAULT_SETTINGS;  // 错误被完全忽略
    }
};

export const fetchProxyStatus = async (platform: Platform): Promise<boolean> => {
    try {
        const command = platform === 'claude' ? 'get_claude_proxy_status' : 'get_codex_proxy_status';
        return await invoke<boolean>(command);
    } catch {
        return false;
    }
};

// 类似问题在多处出现
```

#### 问题影响
- 无法诊断问题根源
- 用户体验差（静默失败）
- 难以追踪和调试

#### 修复方案
添加错误日志记录：

```typescript
export const fetchAppSettings = async (): Promise<AppSettings> => {
    try {
        return await invoke<AppSettings>('get_app_settings');
    } catch (error) {
        console.error('[fetchAppSettings] Failed to fetch app settings:', error);
        // 可选：发送到错误追踪服务
        // Sentry.captureException(error);
        return DEFAULT_SETTINGS;
    }
};

export const fetchProxyStatus = async (platform: Platform): Promise<boolean> => {
    try {
        const command = platform === 'claude' ? 'get_claude_proxy_status' : 'get_codex_proxy_status';
        return await invoke<boolean>(command);
    } catch (error) {
        console.error(`[fetchProxyStatus] Failed to fetch ${platform} proxy status:`, error);
        return false;
    }
};

export const fetchProviders = async (platform: Platform): Promise<Provider[]> => {
    try {
        return await invoke<Provider[]>('load_providers', { kind: platform });
    } catch (error) {
        console.error(`[fetchProviders] Failed to load ${platform} providers:`, error);
        return [];
    }
};
```

#### 增强建议
考虑实现统一的错误处理中间件：

```typescript
// utils/errorHandler.ts
export async function withErrorLogging<T>(
    operation: string,
    fn: () => Promise<T>,
    fallback: T
): Promise<T> {
    try {
        return await fn();
    } catch (error) {
        console.error(`[${operation}] Error:`, error);
        // 可选：上报到监控系统
        return fallback;
    }
}

// 使用示例
export const fetchAppSettings = async (): Promise<AppSettings> => {
    return withErrorLogging(
        'fetchAppSettings',
        () => invoke<AppSettings>('get_app_settings'),
        DEFAULT_SETTINGS
    );
};
```

---

## 🟡 中优先级问题

### 4. 边界条件检查缺失
**文件**: `iswitch-tauri/src-tauri/src/proxy/handler.rs`
**行号**: 84-89
**严重程度**: 🟡 中
**类别**: 逻辑错误

#### 问题描述
检查model字段存在性，但未验证是否为空字符串。

#### 当前代码
```rust
let model = match body.get("model").and_then(|v| v.as_str()) {
    Some(m) => m.to_string(),
    None => {
        return (StatusCode::BAD_REQUEST, "Missing model field").into_response();
    }
};
```

#### 问题场景
- 请求体：`{"model": "", "messages": [...]}`
- 空字符串会通过验证，可能导致后续逻辑错误

#### 修复方案
```rust
let model = match body.get("model").and_then(|v| v.as_str()) {
    Some(m) if !m.is_empty() => m.to_string(),
    Some(_) => {
        return (
            StatusCode::BAD_REQUEST,
            "Model field cannot be empty"
        ).into_response();
    }
    None => {
        return (
            StatusCode::BAD_REQUEST,
            "Missing model field"
        ).into_response();
    }
};
```

#### 测试建议
```rust
#[tokio::test]
async fn test_empty_model_rejected() {
    // 测试空字符串被拒绝
    let body = json!({"model": "", "messages": []});
    let response = handle_claude_messages(State(state), HeaderMap::new(), Json(body)).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

---

### 5. 敏感信息泄露
**文件**: `iswitch-tauri/src-tauri/src/proxy/handler.rs`
**行号**: 325-330
**严重程度**: 🟡 中
**类别**: 信息泄露

#### 问题描述
将上游Provider的错误响应原样返回给客户端，可能泄露敏感信息。

#### 当前代码
```rust
let error_text = match resp.text().await {
    Ok(text) => text,
    Err(_) => "无法读取错误响应体".to_string(),
};
warn!("Provider {} 请求失败: {}, 响应内容: {}", provider.name, status, error_text);
return (status, error_text).into_response();  // 暴露上游错误详情
```

#### 安全风险
- 上游错误可能包含内部API路径、参数等敏感信息
- 攻击者可以利用这些信息进行进一步攻击

#### 修复方案
```rust
// 方案1：返回通用错误消息
let error_text = match resp.text().await {
    Ok(text) => {
        warn!("Provider {} 请求失败: {}, 响应: {}", provider.name, status, text);
        "Provider request failed".to_string()  // 不暴露详细信息
    }
    Err(_) => {
        warn!("Provider {} 请求失败: {}", provider.name, status);
        "Provider request failed".to_string()
    }
};

return (status, error_text).to_string();

// 方案2：仅在开发环境返回详细信息
#[cfg(debug_assertions)]
{
    return (status, error_text).into_response();
}
#[cfg(not(debug_assertions))]
{
    return (status, "Provider request failed").into_response();
}
```

---

### 6. 无界Channel内存泄漏风险
**文件**: `iswitch-tauri/src-tauri/src/proxy/monitor.rs`
**行号**: 93-98
**严重程度**: 🟡 中
**类别**: 资源管理

#### 问题描述
使用 `mpsc::unbounded_channel()` 创建无界channel，如果生产速度远大于消费速度，可能导致内存溢出。

#### 当前代码
```rust
pub fn monitor_response(
    mut response: Response,
    ctx: MonitorContext,
) -> Response {
    let (tx, rx) = mpsc::unbounded_channel();  // 无界channel
    // ...
}
```

#### 问题场景
- 高并发场景下，大量请求同时到达
- 日志处理速度跟不上请求速度
- 内存持续增长，最终OOM

#### 修复方案
```rust
pub fn monitor_response(
    mut response: Response,
    ctx: MonitorContext,
) -> Response {
    // 使用有界channel，容量设为1000
    let (tx, rx) = mpsc::channel(1000);

    tokio::spawn(async move {
        process_log_entry(rx, ctx, status).await;
    });

    // 当channel满时，丢弃最旧的日志
    axum::body::Body::from_stream(
        rx_stream.map(move |chunk| {
            if tx.capacity() == 0 || tx.try_send(chunk).is_err() {
                // Channel满，记录警告并丢弃
                warn!("Log channel full, dropping log entry");
            }
            chunk
        })
    )
}
```

#### 监控建议
- 添加channel容量监控指标
- 记录丢包日志
- 考虑使用采样策略（如只记录10%的请求）

---

## 🟢 低优先级问题

### 7. 配置目录创建失败未通知用户
**文件**: `iswitch-tauri/src-tauri/src/lib.rs`
**行号**: 206-208
**严重程度**: 🟢 低

#### 当前代码
```rust
if let Err(e) = utils::paths::ensure_iswitch_dir() {
    error!(error = %e, "创建配置目录失败");
}
```

#### 改进建议
```rust
if let Err(e) = utils::paths::ensure_iswitch_dir() {
    error!(error = %e, "创建配置目录失败");

    // 尝试使用临时目录
    let temp_dir = std::env::temp_dir().join("iswitch");
    if let Err(temp_err) = std::fs::create_dir_all(&temp_dir) {
        error!(error = %temp_err, "无法创建临时目录，应用可能无法正常工作");
    } else {
        info!("使用临时目录: {}", temp_dir.display());
    }
}
```

---

### 8. 数据库连接池配置固定
**文件**: `iswitch-tauri/src-tauri/src/db/mod.rs`
**行号**: 36-39
**严重程度**: 🟢 低

#### 当前代码
```rust
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect_with(options)
    .await?;
```

#### 改进建议
```rust
// 从配置读取连接池大小，或根据系统资源动态调整
let max_connections = std::env::var("DB_MAX_CONNECTIONS")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(5);

let pool = SqlitePoolOptions::new()
    .max_connections(max_connections)
    .acquire_timeout(Duration::from_secs(30))
    .connect_with(options)
    .await?;
```

---

### 9. 前后端默认值重复定义
**文件**: `iswitch-tauri/src/services/tauri.ts`
**行号**: 62-69
**严重程度**: 🟢 低

#### 当前代码
```typescript
// 前端定义
const DEFAULT_SETTINGS: AppSettings = {
    show_heatmap: true,
    show_home_title: true,
    auto_start: false,
    proxy_port: 18099,
    failover_threshold: 3,
    recovery_timeout_secs: 300,
};
```

#### 问题
- 与Rust后端的 `impl Default for AppSettings` 重复
- 可能出现不一致

#### 改进建议
```typescript
// 从后端获取默认设置
const DEFAULT_SETTINGS: AppSettings = await invoke<AppSettings>('get_default_settings');
```

---

### 10. 硬编码的代理端口
**文件**: `iswitch-tauri/src-tauri/src/proxy/server.rs`
**行号**: 23
**严重程度**: 🟢 低

#### 当前代码
```rust
pub const DEFAULT_PROXY_PORT: u16 = 18099;
```

#### 改进建议
已经合理，但可以考虑：
- 检查端口是否被占用
- 如果被占用，自动尝试其他端口

---

### 11. 任务错误未记录
**文件**: `iswitch-tauri/src-tauri/src/proxy/handler.rs`
**行号**: 300-307, 351-357
**严重程度**: 🟢 低

#### 当前代码
```rust
tokio::spawn(async move {
    let mut log = RequestLog::new(&log_platform, &log_provider, &log_model, is_stream);
    log.http_code = status.as_u16() as i32;
    log.duration_sec = duration;
    log.created_at = request_start.format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = request_log::insert_log(&pool, &log).await;  // 忽略错误
});
```

#### 改进建议
```rust
tokio::spawn(async move {
    let mut log = RequestLog::new(&log_platform, &log_provider, &log_model, is_stream);
    log.http_code = status.as_u16() as i32;
    log.duration_sec = duration;
    log.created_at = request_start.format("%Y-%m-%d %H:%M:%S").to_string();

    if let Err(e) = request_log::insert_log(&pool, &log).await {
        error!("Failed to insert log: {}", e);
        // 考虑将失败的日志写入队列，稍后重试
    }
});
```

---

## ✅ 代码优点总结

### 1. 类型安全
- ✅ Rust使用强类型系统，避免运行时类型错误
- ✅ TypeScript配置严格，减少类型错误
- ✅ 使用 `Result<T, E>` 和 `?` 操作符正确处理错误

### 2. 并发安全
- ✅ 正确使用 `AtomicBool` 处理并发状态
- ✅ 使用 `RwLock` 保护共享状态，并实现双重检查
- ✅ 使用 `Arc` 实现线程安全的共享所有权

### 3. 资源管理
- ✅ RAII模式确保资源自动释放
- ✅ 使用 `SqlitePool` 复用数据库连接
- ✅ 大部分 `unwrap()` 都有 `unwrap_or()` 保护

### 4. 测试覆盖
- ✅ 核心业务逻辑有完善的单元测试
- ✅ 使用 `wiremock` 进行HTTP mock测试
- ✅ 测试覆盖边界条件和错误场景

### 5. 文档规范
- ✅ 所有文件遵循 FractalFlow 规范
- ✅ 注释清晰，说明设计意图
- ✅ README 和设计文档完整

---

## 📈 修复优先级路线图

### 第一周（紧急修复）
- [ ] 修复SQL注入风险（request_log.rs）
- [ ] 修复路径遍历漏洞（settings.rs）
- [ ] 添加前端错误日志（tauri.ts）

### 第二周（重要改进）
- [ ] 添加边界条件检查
- [ ] 过滤敏感信息
- [ ] 替换为有界Channel

### 第三-四周（持续改进）
- [ ] 统一错误处理策略
- [ ] 添加集成测试
- [ ] 性能监控和告警
- [ ] 文档更新

---

## 🎯 长期建议

### 1. 安全加固
- 实施内容安全策略（CSP）
- 添加请求速率限制
- 实施API密钥轮换机制
- 定期进行安全审计

### 2. 可观测性
- 集成APM监控（如Sentry）
- 添加性能指标采集
- 实施分布式追踪
- 建立告警机制

### 3. 开发流程
- 引入pre-commit hooks
- 添加CI/CD流水线
- 自动化安全扫描
- 代码审查清单

### 4. 测试策略
- 增加E2E测试覆盖
- 添加负载测试
- 混沌工程测试
- 自动化回归测试

---

## 📚 参考资料

### 安全编码规范
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://doc.rust-lang.org/nomicon/unsafe-guidelines.html)
- [TypeScript Security Best Practices](https://typescript-eslint.io/rules/)

### 相关工具
- `cargo-audit`: Rust依赖安全扫描
- `npm audit`: Node.js依赖安全扫描
- `semgrep`: 静态代码分析
- `sonarqube`: 代码质量分析

---

**报告生成时间**: 2026-01-06
**下次扫描建议**: 修复高优先级问题后重新扫描
**联系方式**: 如有问题，请提交Issue或联系维护团队
