# iSwitch 安全最佳实践指南

**版本**: 1.0
**适用范围**: 全体开发人员
**更新日期**: 2026-01-06

---

## 🎯 目标

本文档提供 iSwitch 项目开发过程中应遵循的安全最佳实践，确保代码安全性和系统可靠性。

---

## 📚 目录

1. [Rust安全编程](#1-rust安全编程)
2. [TypeScript安全编程](#2-typescript安全编程)
3. [数据库安全](#3-数据库安全)
4. [API安全](#4-api安全)
5. [密钥管理](#5-密钥管理)
6. [日志和监控](#6-日志和监控)
7. [依赖管理](#7-依赖管理)
8. [安全开发生命周期](#8-安全开发生命周期)

---

## 1. Rust安全编程

### 1.1 错误处理

#### ✅ DO - 使用 Result 类型

```rust
// 好的做法
pub fn read_config(path: &Path) -> AppResult<Config> {
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(Into::into)
}

// 避免
pub fn read_config(path: &Path) -> Config {
    let content = std::fs::read_to_string(path).unwrap();  // ❌ 可能panic
    serde_json::from_str(&content).unwrap()
}
```

#### ✅ DO - 使用 unwrap_or() 处理默认值

```rust
// 好的做法
let port = std::env::var("PROXY_PORT")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(18099);

// 避免
let port = std::env::var("PROXY_PORT")
    .unwrap()
    .parse()
    .unwrap();  // ❌ 可能panic
```

#### ✅ DO - 避免在生产代码使用 panic!/expect!

```rust
// 好的做法
pub fn connect_to_database(url: &str) -> AppResult<Connection> {
    Connection::connect(url).map_err(|e| {
        AppError::DatabaseConnection {
            url: url.to_string(),
            source: e,
        }
    })
}

// 避免
pub fn connect_to_database(url: &str) -> Connection {
    Connection::connect(url).expect("Failed to connect")  // ❌ 生产panic
}
```

### 1.2 内存安全

#### ✅ DO - 使用有界channel

```rust
// 好的做法
use tokio::sync::mpsc;

let (tx, rx) = mpsc::channel::<Bytes>(1000);  // 容量1000

// 发送时使用try_send
if let Err(_) = tx.try_send(data) {
    // channel满，记录日志
    warn!("Log channel full, dropping entry");
}

// 避免
let (tx, rx) = mpsc::unbounded_channel::<Bytes>();  // ❌ 可能内存泄漏
let _ = tx.send(data);  // 无限增长
```

#### ✅ DO - 避免内存泄漏

```rust
// 好的做法 - 使用 RAII
struct ConnectionGuard {
    conn: Option<Connection>,
}

impl ConnectionGuard {
    fn new(conn: Connection) -> Self {
        Self { conn: Some(conn) }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            let _ = conn.close();  // 自动清理
        }
    }
}
```

### 1.3 并发安全

#### ✅ DO - 正确使用 Mutex 和 RwLock

```rust
use std::sync::{Arc, RwLock};

struct SafeState {
    data: Arc<RwLock<Vec<String>>>,
}

impl SafeState {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn add_item(&self, item: String) {
        let mut data = self.data.write().unwrap();  // 写锁
        data.push(item);
    }

    fn get_items(&self) -> Vec<String> {
        let data = self.data.read().unwrap();  // 读锁
        data.clone()
    }
}
```

#### ✅ DO - 避免死锁

```rust
// 好的做法 - 按固定顺序获取锁
fn update_data(state1: &Arc<Mutex<Data1>>, state2: &Arc<Mutex<Data2>>) {
    // 总是先锁 state1，再锁 state2
    let s1 = state1.lock().unwrap();
    let s2 = state2.lock().unwrap();
    // ...
}

// 避免 - 可能死锁
fn update_data_unsafe(state1: &Arc<Mutex<Data1>>, state2: &Arc<Mutex<Data2>>) {
    let s1 = state1.lock().unwrap();
    let s2 = state2.lock().unwrap();
    // 如果另一个线程先锁 state2，会死锁
}
```

---

## 2. TypeScript安全编程

### 2.1 类型安全

#### ✅ DO - 避免使用 any

```typescript
// 好的做法
interface Provider {
    name: string;
    baseUrl: string;
    apiKey: string;
}

function loadProvider(name: string): Provider {
    // ...
}

// 避免
function loadProvider(name: string): any {  // ❌ 失去类型安全
    return { name, apiKey: 'xxx' };
}
```

#### ✅ DO - 验证输入类型

```typescript
// 好的做法
function validateModel(model: unknown): string {
    if (typeof model !== 'string') {
        throw new Error('Model must be a string');
    }
    if (model.trim().length === 0) {
        throw new Error('Model cannot be empty');
    }
    return model;
}

// 使用
try {
    const model = validateModel(body.model);
} catch (error) {
    console.error('[validateModel] Invalid model:', error);
}
```

### 2.2 错误处理

#### ✅ DO - 始终处理错误

```typescript
// 好的做法
export const fetchProviders = async (kind: string): Promise<Provider[]> => {
    try {
        return await invoke<Provider[]>('load_providers', { kind });
    } catch (error) {
        console.error('[fetchProviders] Failed to load providers:', error);
        return [];  // 返回空数组作为fallback
    }
};

// 避免
export const fetchProviders = async (kind: string): Promise<Provider[]> => {
    return await invoke<Provider[]>('load_providers', { kind });
    // ❌ 如果失败，用户看不到错误
};
```

---

## 3. 数据库安全

### 3.1 SQL注入防护

#### ✅ DO - 使用参数化查询

```rust
// 好的做法 - 参数化
let row = sqlx::query(
    "SELECT * FROM users WHERE id = ?"
)
.bind(user_id)
.fetch_one(pool)
.await?;

// 避免 - 字符串拼接
let sql = format!("SELECT * FROM users WHERE id = {}", user_id);  // ❌ SQL注入风险
let row = sqlx::query(&sql).fetch_one(pool).await?;
```

### 3.2 数据加密

#### ✅ DO - 加密敏感数据

```rust
// 好的做法 - 使用 AES-256-GCM
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::Rng;

fn encrypt_data(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::from_slice(key));
    let nonce = Nonce::rand(&mut rand::thread_rng());
    let ciphertext = cipher.encrypt(&nonce, data)?;
    Ok(ciphertext)
}
```

---

## 4. API安全

### 4.1 输入验证

#### ✅ DO - 验证所有输入

```rust
// 好的做法
fn validate_model(model: &str) -> AppResult<String> {
    if model.is_empty() {
        return Err(AppError::InvalidInput("Model cannot be empty".into()));
    }

    if model.trim().is_empty() {
        return Err(AppError::InvalidInput("Model cannot be whitespace".into()));
    }

    if model.len() > 100 {
        return Err(AppError::InvalidInput("Model too long".into()));
    }

    Ok(model.to_string())
}
```

### 4.2 输出过滤

#### ✅ DO - 过滤敏感信息

```rust
// 好的做法 - 不暴露上游错误
pub async fn handle_request() -> Response {
    match provider.call().await {
        Ok(resp) => resp.into_response(),
        Err(e) => {
            error!("Provider error: {}", e);  // 详细日志
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Request failed"  // 通用错误消息
            ).into_response()
        }
    }
}
```

---

## 5. 密钥管理

### 5.1 密钥存储

#### ✅ DO - 使用系统Keychain

```rust
#[cfg(target_os = "macos")]
use security_framework::passwords::{
    get_generic_password, set_generic_password,
};

pub async fn save_api_key(account: &str, password: &str) -> AppResult<()> {
    set_generic_password("iswitch", account, password)
        .map_err(|e| AppError::KeychainError {
            operation: "save".to_string(),
            source: e.into(),
        })?;
    Ok(())
}

pub async fn load_api_key(account: &str) -> AppResult<String> {
    get_generic_password("iswitch", account)
        .map_err(|e| AppError::KeychainError {
            operation: "load".to_string(),
            source: e.into(),
        })?
        .password
        .ok_or_else(|| AppError::KeyNotFound)
}
```

### 5.2 密钥脱敏

#### ✅ DO - 脱敏日志中的密钥

```rust
// 好的做法
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 4 {
        return "****".to_string();
    }
    format!("{}****", &key[..4])
}

// 使用
info!("Using API key: {}", mask_api_key(&api_key));
// 输出: Using API key: sk-1****
```

---

## 6. 日志和监控

### 6.1 结构化日志

#### ✅ DO - 使用 tracing 库

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(pool))]
pub async fn process_request(pool: &Pool<Sqlite>, request: &Request) -> AppResult<Response> {
    info!(
        request_id = %request.id,
        model = %request.model,
        provider = %request.provider,
        "Processing request"
    );

    match execute_request(pool, request).await {
        Ok(response) => {
            info!(
                duration_ms = response.duration_ms,
                status = %response.status,
                "Request completed"
            );
            Ok(response)
        }
        Err(e) => {
            error!(
                error = %e,
                "Request failed"
            );
            Err(e)
        }
    }
}
```

### 6.2 审计日志

#### ✅ DO - 记录关键操作

```rust
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: AuditAction,
    pub resource: String,
    pub result: AuditResult,
}

pub enum AuditAction {
    ApiKeyAdded,
    ApiKeyRemoved,
    ProviderEnabled,
    ProviderDisabled,
    ConfigImported,
}

pub fn log_audit_event(event: AuditEvent) {
    info!(
        timestamp = %event.timestamp,
        actor = %event.actor,
        action = ?event.action,
        resource = %event.resource,
        result = ?event.result,
        "AUDIT_EVENT"
    );
}
```

---

## 7. 依赖管理

### 7.1 定期更新

```bash
# 每周检查一次
cd iswitch-tauri/src-tauri
cargo outdated

# 每月更新一次
cargo update

# 审计依赖
cargo audit
```

### 7.2 依赖审查

```toml
# Cargo.toml - 指定可信来源
[dependencies]
tokio = { version = "1.35", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }

# 只选择必要的features
[dependencies.reqwest]
version = "0.11"
default-features = false
features = ["rustls-tls", "json"]  # 禁用默认的native-tls
```

---

## 8. 安全开发生命周期

### 8.1 设计阶段

#### ✅ 安全设计原则
1. **最小权限原则**: 只请求必要的权限
2. **纵深防御**: 多层安全控制
3. **失败安全**: 默认拒绝，明确允许
4. **最小惊讶**: 行为符合用户预期

### 8.2 开发阶段

#### ✅ 安全编码实践
1. **代码审查**: 所有PR必须经过安全审查
2. **单元测试**: 包含安全测试用例
3. **静态分析**: 使用clippy和semgrep
4. **同行评审**: 至少一人审查安全相关代码

### 8.3 测试阶段

#### ✅ 安全测试清单
- [ ] 输入验证测试
- [ ] 认证测试
- [ ] 授权测试
- [ ] DoS测试
- [ ] 渗透测试

### 8.4 部署阶段

#### ✅ 部署前检查
- [ ] 移除调试代码
- [ ] 配置生产环境日志级别
- [ ] 启用HTTPS/TLS
- [ ] 配置防火墙
- [ ] 设置监控告警

### 8.5 维护阶段

#### ✅ 持续维护
- [ ] 定期更新依赖
- [ ] 监控安全公告
- [ ] 定期安全审计
- [ ] 漏洞响应流程

---

## 🚀 安全工具推荐

### Rust工具
```bash
# 安装
cargo install cargo-audit
cargo install cargo-outdated
cargo install cargo-machete

# 使用
cargo audit              # 审计依赖
cargo outdated           # 检查更新
cargo clippy             # Lint检查
cargo machete            # 未使用依赖
```

### TypeScript工具
```bash
npm install -g npm-audit-resolver
npm install -g snyk
npm install -g depcheck

npm audit                # 审计依赖
npm audit fix            # 自动修复
snyk test                # 深度扫描
depcheck                 # 检查未使用依赖
```

### 通用工具
```bash
# 密钥扫描
gitleaks detect --source .
trufflehog filesystem .

# 容器扫描
docker scan iswitch:latest

# 端口扫描
nmap -sV -sC localhost
```

---

## 📚 学习资源

### Rust安全
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust Unsafe Guidelines](https://doc.rust-lang.org/nomicon/unsafe-guidelines.html)
- [Rust Security Best Practices](https://github.com/RustSec/community)

### Web安全
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [OWASP Cheat Sheet Series](https://cheatsheetseries.owasp.org/)
- [CWE Top 25](https://cwe.mitre.org/top25/)

### 加密
- [Mozilla Web Security Guidelines](https://info.mozilla.org/en-US/docs/Web/Security)
- [Google Application Security Best Practices](https://security.googleblog.com/)

---

## 📋 安全审查清单

### 提交代码前检查

```bash
# 1. 运行所有测试
cargo test
npm test

# 2. 运行lint检查
cargo clippy
npm run lint

# 3. 审计依赖
cargo audit
npm audit

# 4. 检查密钥泄露
gitleaks detect

# 5. 自动化测试
./test/security/run-all-tests.sh
```

### PR审查检查清单

**安全相关**:
- [ ] 是否引入新的外部输入？
- [ ] 是否验证所有输入？
- [ ] 是否处理所有错误？
- [ ] 是否记录安全日志？
- [ ] 是否暴露敏感信息？

**代码质量**:
- [ ] 代码是否清晰易懂？
- [ ] 是否有足够的注释？
- [ ] 是否有单元测试？
- [ ] 是否通过所有lint检查？

---

## 🎓 培训资源

### 推荐课程
- [Rust Security](https://www.pluralsight.com/courses/rust-security)
- [Web Application Security](https://www.coursera.org/learn/web-application-security)
- [Secure Coding in Rust](https://www.udemy.com/course/secure-coding-rust/)

### 内部培训
每月安全会议：
- 1月: 代码安全最佳实践
- 2月: Web应用安全
- 3月: 密码学和加密
- 4月: 渗透测试基础
- 5月: 事故响应流程
- 6月: 安全工具使用

---

## 📞 报告安全问题

### 发现漏洞？
1. **不要**在公开issue中报告
2. **通过私有渠道**发送报告
3. **提供详细复现步骤**
4. **等待确认**再公开

### 联系方式
- Email: security@iswitch.example.com
- PGP Key: [待添加]
- HackerOne: [待添加]

---

**文档维护**: 安全团队
**更新频率**: 季度
**反馈**: 提交PR或Issue
