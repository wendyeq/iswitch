# iSwitch 安全测试指南

**文档版本**: 1.0
**适用版本**: iSwitch 0.1.0+
**测试人员**: 开发团队 + 安全团队

---

## 📚 目录

1. [测试环境准备](#1-测试环境准备)
2. [自动化安全扫描](#2-自动化安全扫描)
3. [手工安全测试](#3-手工安全测试)
4. [渗透测试](#4-渗透测试)
5. [代码审计](#5-代码审计)
6. [持续安全监控](#6-持续安全监控)

---

## 1. 测试环境准备

### 1.1 环境要求

```bash
# 安装必要的工具
# Rust工具链
rustup update
cargo install cargo-audit
cargo install cargo-outdated

# Node.js工具
npm install -g npm-audit-resolver
npm install -g snyk

# 安全扫描工具
# macOS
brew install nmap sqlmap

# Linux
sudo apt install nmap sqlmap

# Python安全工具
pip3 install bandit safety
```

### 1.2 测试数据

```bash
# 准备测试配置
mkdir -p test/security/test-data

# 创建测试Provider配置
cat > test/security/test-data/test-provider.json << 'EOF'
{
  "providers": [
    {
      "name": "test-provider",
      "base_url": "https://api.example.com",
      "api_key": "sk-test-dummy-key-for-testing-12345"
    }
  ]
}
EOF
```

---

## 2. 自动化安全扫描

### 2.1 Rust依赖安全扫描

```bash
cd iswitch-tauri/src-tauri

# 1. Cargo Audit - 检查已知漏洞
cargo audit

# 预期输出示例:
# Fetching advisory database from https://github.com/RustSec/advisory-db
# Found 0 vulnerabilities
# ✅ 所有依赖安全

# 2. 检查过时的依赖
cargo outdated

# 3. 生成依赖树
cargo tree > test/security/reports/cargo-tree.txt

# 4. 检查未使用的依赖
cargo machete
```

**修复漏洞**:
```bash
# 如果发现漏洞，更新依赖
cargo update -p package_name

# 或更新所有
cargo update

# 验证修复
cargo audit
```

---

### 2.2 TypeScript/JavaScript依赖扫描

```bash
cd iswitch-tauri

# 1. npm audit - 检查已知漏洞
npm audit

# 2. 自动修复可修复的漏洞
npm audit fix

# 3. 检查过时的包
npm outdated

# 4. 使用Snyk深度扫描
npx snyk test
```

**配置 .npmrc**:
```bash
# 在项目根目录创建 .npmrc
cat > .npmrc << 'EOF'
audit=true
audit-level=moderate
fund=false
EOF
```

---

### 2.3 代码静态分析

#### Rust代码扫描

```bash
# 1. 使用 clippy 进行lint检查
cd iswitch-tauri/src-tauri
cargo clippy --all-targets --all-features -- -D warnings

# 2. 生成clippy报告
cargo clippy --message-format=json > test/security/reports/clippy-report.json

# 3. 使用cargo-outdated检查更新
cargo outdated > test/security/reports/outdated.txt
```

#### TypeScript代码扫描

```bash
# 1. ESLint安全检查
cd iswitch-tauri
npm run lint || npm run lint:security

# 2. TypeScript编译检查
npm run type-check

# 3. 使用npm audit检查安全
npm audit --production
```

---

### 2.4 密钥和凭证扫描

```bash
# 使用 truffleHog 扫描密钥泄露
docker run --rm -v "$PWD:/pwd" trufflesecurity/trufflehog:latest filesystem /pwd

# 或使用 gitleaks
gitleaks detect --source . --report-path test/security/reports/gitleaks-report.json

# 扫描环境变量
env | grep -i "key\|secret\|token\|password"
```

**预防措施**:
```bash
# 添加 .gitignore 规则
echo "*.key
*.pem
.env
.env.local
test-data/secrets/" >> .gitignore

# 预提交hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# 检测是否有密钥被提交
if git diff --cached --name-only | xargs grep -l "sk-\|api_key\|secret"; then
    echo "警告: 检测到可能的密钥！"
    exit 1
fi
EOF
chmod +x .git/hooks/pre-commit
```

---

## 3. 手工安全测试

### 3.1 API安全测试

#### 测试1: SQL注入测试

```bash
#!/bin/bash
# test/security/tests/sql-injection-test.sh

echo "=== SQL注入测试 ==="

# 测试端点
ENDPOINT="http://127.0.0.1:18099/v1/messages"

# 测试用例
test_cases=(
    # 基础SQL注入
    '{"model": "test'"'"'; DROP TABLE request_logs; --", "messages": []}'

    # UNION注入
    '{"model": "test'"'"' UNION SELECT * FROM request_logs--", "messages": []}'

    # 布尔盲注
    '{"model": "test'"'"' AND 1=1--", "messages": []}'

    # 时间盲注
    '{"model": "test'"'"'; WAITFOR DELAY '00:00:05'--", "messages": []}'
)

for i in "${!test_cases[@]}"; do
    echo "测试用例 $((i+1)):"
    curl -X POST "$ENDPOINT" \
        -H "Content-Type: application/json" \
        -d "${test_cases[$i]}" \
        -w "\nHTTP状态: %{http_code}\n"
    echo "---"
done

echo "✅ 所有测试应返回 400 Bad Request"
```

**预期结果**:
- 所有测试应返回 `400 Bad Request`
- 不应出现数据库错误
- 不应导致应用崩溃

---

#### 测试2: 路径遍历测试

```typescript
// test/security/tests/path-traversal.test.ts
import { expect } from 'chai';
import { invoke } from '@tauri-apps/api/core';

describe('路径遍历测试', () => {
    const maliciousPaths = [
        // 相对路径遍历
        '../../../etc/passwd',
        '..\\..\\..\\..\\windows\\system32\\config\\sam',

        // 绝对路径
        '/etc/passwd',
        '/etc/shadow',
        'C:\\Windows\\System32\\config\\SAM',

        // URL编码
        '%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd',
        '..%2f..%2f..%2fetc%2fpasswd',

        // Unicode编码
        '..%c0%af..%c0%af..%c0%afetc%c0%afpasswd',

        // 空字节注入
        '../../../etc/passwd%00.jpg',
    ];

    maliciousPaths.forEach((path, index) => {
        it(`应该拒绝恶意路径 ${index + 1}: ${path}`, async () => {
            try {
                await invoke('import_from_file', { path });
                expect.fail('应该抛出错误');
            } catch (error) {
                expect(error).to.include('Invalid file path')
                          .or.to.include('文件不存在或不是有效文件')
                          .or.to.include('只允许导入 JSON 格式的配置文件');
            }
        });
    });

    it('应该接受合法路径', async () => {
        const legalPath = '/tmp/test-config.json';
        try {
            // 创建合法测试文件
            await invoke('import_from_file', { path: legalPath });
        } catch (error) {
            // 文件不存在是可接受的
            expect(error).to.not.include('Invalid');
        }
    });
});
```

---

#### 测试3: DoS防护测试

```bash
#!/bin/bash
# test/security/tests/dos-test.sh

echo "=== DoS防护测试 ==="

ENDPOINT="http://127.0.0.1:18099/v1/messages"

# 测试1: 大payload攻击
echo "测试1: 大payload攻击"
LARGE_PAYLOAD=$(python3 -c "import json; print(json.dumps({'model': 'x'*10000, 'messages': []}))")
curl -X POST "$ENDPOINT" -H "Content-Type: application/json" -d "$LARGE_PAYLOAD"
echo "✅ 应该被拒绝或超时"

# 测试2: 快速请求
echo "测试2: 快速请求（100次/秒）"
for i in {1..100}; do
    curl -s "$ENDPOINT" -X POST -H "Content-Type: application/json" -d '{"model": "test", "messages": []}' &
done
wait
echo "✅ 应该有速率限制或队列控制"

# 测试3: 长连接
echo "测试3: 长连接保持"
timeout 30 curl -N "$ENDPOINT" -X POST \
    -H "Content-Type: application/json" \
    -d '{"model": "test", "messages": [{"role": "user", "content": "x"*1000000}]}'
echo "✅ 应该有超时机制"

echo "=== DoS测试完成 ==="
```

---

### 3.2 内存安全测试

```bash
# 1. Valgrind内存泄漏检测
cargo install valgrind

valgrind --leak-check=full \
         --show-leak-kinds=all \
         --track-origins=yes \
         ./target/debug/iswitch

# 2. 使用 ASan（Address Sanitizer）
RUSTFLAGS=-Zsanitizer=address cargo test

# 3. 使用 TSan（Thread Sanitizer）
RUSTFLAGS=-Zsanitizer=thread cargo test
```

---

### 3.3 并发安全测试

```rust
// test/security/tests/concurrent-test.rs
use std::sync::Arc;
use std::thread;

#[tokio::test]
async fn test_concurrent_provider_access() {
    let provider_service = Arc::new(ProviderService::new());
    let mut handles = vec![];

    // 模拟100个并发请求
    for i in 0..100 {
        let service = provider_service.clone();
        let handle = tokio::spawn(async move {
            service.load_providers("claude").await
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "并发访问应该成功");
    }
}
```

---

## 4. 渗透测试

### 4.1 端口扫描

```bash
# 使用nmap扫描开放端口
nmap -sV -sC localhost

# 预期结果:
# PORT      STATE SERVICE  VERSION
# 18099/tcp open  http?
# 其他端口应该关闭或过滤
```

---

### 4.2 Web漏洞扫描

```bash
# 使用 Nikto 扫描Web应用
nikto -h http://127.0.0.1:18099 \
      -output test/security/reports/nikto-report.txt

# 预期结果:
# 应该没有高危漏洞
```

---

### 4.3 中间人攻击测试

```bash
# 使用Burp Suite测试HTTPS
# 1. 配置Burp Proxy
# 2. 设置系统代理 127.0.0.1:8080
# 3. 拦截HTTPS请求
# 4. 验证证书是否有效

# 或使用curl测试
curl -k https://example.com --proxy http://127.0.0.1:8080
```

---

## 5. 代码审计

### 5.1 自动化代码审计工具

```bash
# 1. Semgrep - 自定义规则
pip install semgrep

# 创建安全规则
cat > test/security/semgrep-rules.yaml << 'EOF'
rules:
  - id: rust-sql-injection
    pattern: sqlx::query($FORMAT, ...)
    message: "使用参数化查询避免SQL注入"
    languages: [rust]
    severity: ERROR

  - id: rust-unwrap
    pattern: $EXPR.unwrap()
    message: "避免使用unwrap()，使用unwrap_or()"
    languages: [rust]
    severity: WARNING

  - id: rust-assert
    pattern: assert!($EXPR)
    message: "避免在生产代码使用assert!"
    languages: [rust]
    severity: WARNING
EOF

# 运行扫描
semgrep --config test/security/semgrep-rules.yaml src/
```

---

### 5.2 手工代码审计清单

#### 审计要点

**1. 数据访问**
- [ ] 所有数据库查询使用参数化？
- [ ] 文件操作有路径验证？
- [ ] 敏感数据加密存储？

**2. 输入验证**
- [ ] 所有用户输入都验证？
- [ ] 类型检查完整？
- [ ] 边界条件处理？

**3. 错误处理**
- [ ] 错误消息不泄露敏感信息？
- [ ] 异常被正确捕获？
- [ ] Panic处理合理？

**4. 加密**
- [ ] 使用最新的加密算法？
- [ ] 密钥管理安全？
- [ ] 证书验证有效？

---

## 6. 持续安全监控

### 6.1 CI/CD集成

#### GitHub Actions示例

```yaml
# .github/workflows/security-scan.yml
name: Security Scan

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0'  # 每周日

jobs:
  rust-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run cargo audit
        run: |
          cd iswitch-tauri/src-tauri
          cargo install cargo-audit
          cargo audit

      - name: Run cargo clippy
        run: |
          cd iswitch-tauri/src-tauri
          cargo clippy --all-targets --all-features -- -D warnings

  npm-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Run npm audit
        run: |
          cd iswitch-tauri
          npm audit --audit-level=moderate

      - name: Run npm audit fix
        run: |
          cd iswitch-tauri
          npm audit fix || true

  secrets-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # 完整历史

      - name: Run TruffleHog
        uses: trufflesecurity/trufflehog@main
        with:
          path: ./
          base: ${{ github.ref }}
```

---

### 6.2 监控和告警

#### 应用监控指标

```rust
// src-tauri/src/monitoring.rs
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SecurityMetrics {
    // 认证失败次数
    pub auth_failures: AtomicU64,

    // 畸形路径尝试
    pub path_traversal_attempts: AtomicU64,

    // SQL注入尝试
    pub sql_injection_attempts: AtomicU64,

    // 速率限制触发
    pub rate_limit_triggers: AtomicU64,
}

impl SecurityMetrics {
    pub fn report_auth_failure(&self) {
        let count = self.auth_failures.fetch_add(1, Ordering::Relaxed);

        // 告警阈值
        if count > 100 {
            eprintln!("⚠️ 安全告警: 认证失败次数过高 ({})", count);
            // 发送到监控系统
        }
    }
}
```

---

### 6.3 定期安全审查

#### 审查周期

| 活动频率 | 活动类型 | 责任人 |
|---------|---------|--------|
| 每周 | 自动化安全扫描 | CI/CD系统 |
| 每月 | 依赖更新 | 开发团队 |
| 每季度 | 代码审计 | 安全团队 |
| 每半年 | 渗透测试 | 第三方 |
| 每年 | 全面安全评估 | 安全团队 |

---

## 📊 测试报告模板

```markdown
# 安全测试报告

**项目**: iSwitch
**测试日期**: YYYY-MM-DD
**测试人员**: ___________
**测试版本**: 0.1.0

---

## 测试执行摘要

| 测试类别 | 测试数量 | 通过 | 失败 | 覆盖率 |
|---------|---------|------|------|--------|
| 自动化扫描 | 10 | 10 | 0 | 100% |
| 手工测试 | 25 | 25 | 0 | 95% |
| 渗透测试 | 15 | 14 | 1 | 90% |
| **总计** | **50** | **49** | **1** | **95%** |

---

## 发现的问题

### 高危 (0)
无

### 中危 (1)
1. 缺少速率限制机制
   - **位置**: `src-tauri/src/proxy/server.rs`
   - **建议**: 添加速率限制中间件
   - **优先级**: P1

### 低危 (3)
...

---

## 测试环境
- 操作系统: macOS 14.0
- Rust版本: 1.75.0
- Node版本: v18.19.0
- 工具版本: cargo-audit 0.20.0

---

## 附录
- 详细测试日志: `test/security/logs/`
- 扫描报告: `test/security/reports/`
```

---

## 🎯 快速开始

```bash
# 1. 克隆仓库并进入目录
cd /path/to/iswitch

# 2. 运行所有安全测试
./test/security/run-all-tests.sh

# 3. 查看报告
cat test/security/reports/summary.txt
```

---

**文档维护**: 安全团队
**更新频率**: 每季度
**反馈**: 提交Issue或PR
