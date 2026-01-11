### Code Review Report

**目标**: Git Staged Changes (Rebranding & Integration Tests)
**审查时间**: 2026-01-05 23:15
**总体评分**: 8 / 10

---

#### 📋 变更概览

- **变更文件数**: ~60 (涉及 Rust Backend, Frontend, Config, Docs, Icons)
- **主要变更**:
    - 应用重命名 "code-switch" -> "iswitch" (配置, 路径, 图标)
    - Rust Backend 集成测试 (`integration_test.rs`)
    - 安全审计配置 (`audit.toml`)
    - Frontend API 迁移 (Wails -> Tauri)

---

#### 🔴 Critical Issues (必须修复)

- **Frontend 测试严重缺失** (Location: `iswitch-tauri/src/`)
  > **描述**: 审查 `iswitch-tauri/src` 目录，未发现任何 TypeScript 或 Vue 单元测试文件 (`*.test.ts`, `*.spec.ts`)。这不符合 Phase 7 测试阶段的要求，前端逻辑缺乏保障。
  > **建议**: 立即使用 `/unit-test-generator` 为核心组件（如 `MCPList.vue`, `Skill/Index.vue`）生成单元测试。

- **Integration Test 竞态与端口冲突** (Location: `src-tauri/tests/integration_test.rs:31,40`)
  > **描述**:
  > 1. 使用固定端口 `18099`，可能导致 CI 并行执行失败或与本地运行的实例冲突。
  > 2. 使用 `sleep(Duration::from_secs(1))` 等待服务器启动，这不仅拖慢测试，而且在慢速机器上可能不可靠 (Flaky Test)。
  > **建议**:
  > - 使用端口 `0` 让系统自动分配端口，然后从 `start_proxy_server` 返回实际绑定端口。
  > - 或者使用简单的重试/轮询机制检测端口是否开放，替代固定 `sleep`。

---

#### 🟡 Improvements (建议改进)

- **Locale 字符串硬编码路径** (Location: `src/locales/en.json`, `zh.json`)
  > **描述**: 提示文本直接写入了 `~/.iswitch/mcp-servers.json`。如果后续后端逻辑变更配置文件名，会导致文案误导。
  > **建议**: 在前端使用变量插值，或确保文案中的路径与后端 `paths.rs` 的定义严格同步。

- **Skill Repo 功能隐藏** (Location: `src/components/Skill/Index.vue`)
  > **描述**: 注释掉了 `skillRepoUrl` 并移除了打开仓库的按钮逻辑。这是临时屏蔽还是永久移除？
  > **建议**: 如果是永久移除，应直接删除代码而非注释；如果是临时，建议添加 `TODO` 注释说明原因。

---

#### 🟢 Good Practices (值得肯定)

- **FractalFlow Header**: `integration_test.rs` 完整包含了符合 V1.0 标准的 Header。
- **安全配置**: `audit.toml` 明确处理了已知的 `RUSTSEC` 漏洞，忽略了不相关的 MySQL/GTK 问题，减少了误报。
- **依赖隔离**: 集成测试正确使用了 `tempfile` 创建临时数据库，避免污染开发环境。

---

#### 🧩 FractalFlow Check

| 检查项            | 状态              | 备注                   |
| ----------------- | ----------------- | ---------------------- |
| Header 完整性     | ⚠️ Partial | 新文件 `integration_test.rs` ✅; 旧 Vue 文件未检查，可能缺失 |
| 语义链接有效性    | ✅ Pass | `integration_test.rs` 链接至 `proxy/server.rs` ✅ |
| .folder.md 一致性 | ✅ Pass | `src-tauri/tests` 符合结构 |
| 中文注释          | ✅ Pass | `integration_test.rs` 使用了清晰的中文注释 |

---

#### 📋 OpenSpec 符合度检查

**关联规范**: `openspec/changes/migrate-to-tauri-stack/design.md`

| Requirement        | 实现状态       | 冲突类型 | 备注       |
| ------------------ | -------------- | -------- | ---------- |
| App Rename (iSwitch) | ✅ 已实现 | - | 配置、图标、路径均已更新 |
| Config Path (~/.iswitch) | ✅ 已实现 | - | 设置与文档一致 |
| Import Legacy Config | ✅ 已实现 | - | `import_service.rs` 更新支持多源导入 |
| Integration Tests | ⏳ 部分实现 | - | 已添加 Proxy 基础测试，但覆盖率待提升 |

---

#### 📊 Impact Analysis (影响分析)

**直接影响**:
- **用户数据**: 升级后用户数据将存储在 `~/.iswitch/`，需确保导入向导 (Import Wizard) 切实可用，否则用户会丢失旧配置。
- **应用标识**: 图标和名称变更，用户需适应。

**风险评估**: 中
- 数据迁移是高风险点，需重点测试 `ImportService`。

---

#### 📝 审查结论

此次变更量大，完成了关键的品牌重塑和路径迁移。核心的 Rust 后端逻辑看起来稳健，但前端测试严重缺失是一个较大的质量风险。

**建议操作**:

- [ ] **必须** 修复 `integration_test.rs` 的端口和 sleep 问题。
- [ ] **强烈建议** 运行 `/unit-test-generator` 补全前端组件测试。
- [ ] 确保在合并前手动验证配置导入流程 (Import Logic)。
- [ ] 可以合并 (After Critical Fixes)。
