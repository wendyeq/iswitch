### Code Review Report

**目标**: Phase 4 Changes (MCP & Skill Services)
**审查时间**: 2026-01-05 16:40
**总体评分**: 9/10

---

#### 📋 变更概览

- **变更文件数**: 8
  - `src-tauri/src/services/mcp_service.rs` (New)
  - `src-tauri/src/services/skill_service.rs` (New)
  - `src-tauri/src/commands/mcp.rs` (New)
  - `src-tauri/src/commands/skill.rs` (New)
  - `src-tauri/src/lib.rs` (Modified)
  - `src-tauri/src/error.rs` (Modified)
  - `src-tauri/Cargo.toml` (Modified)
  - `openspec/changes/migrate-to-tauri-stack/tasks.md` (Modified)

---

#### 🔴 Critical Issues (必须修复)

无。

---

#### 🟡 Improvements (建议改进)

- **Testability (测试性)**
  - `MCPService` and `SkillService` currently rely on global path functions (`paths::mcp_servers_path()`), making it difficult to write isolated unit tests involving file I/O (as seen in the empty `test_list_servers_default`).
  - **建议**: Apply the same pattern used in `ProviderService` and `CodexSettingsService`, adding a `root_path: Option<PathBuf>` field to the struct and using it in path resolution. This allows injecting a temp directory during tests.

- **Performance (性能)**
  - `MCPService::check_missing_placeholders` compiles the Regex `\{([a-zA-Z0-9_]+)\}` on every call.
  - **建议**: Use `once_cell::sync::Lazy` or `std::sync::OnceLock` to compile the Regex once globally.

- **Security (安全)**
  - In `SkillService::install_skill`, while `zip` crate's `mangled_name()` provides basic protection, it is best practice to explicitly verify that the extraction `dest_path` starts with the `target_dir` to absolutely prevent Zip Slip vulnerabilities.
  - **建议**: Add `if !dest_path.starts_with(&target_dir) { return Err(...) }` check.

---

#### 🟢 Good Practices (值得肯定)

- **Async/Blocking Separation**: explicitly used `tokio::task::spawn_blocking` for ZIP extraction and CPU-intensive operations, preventing async runtime blocking.
- **FractalFlow Compliance**: All new files strictly follow the FractalFlow header and documentation standards.
- **Standard Error Handling**: Consistently used `AppError` and `AppResult`, integrating well with the existing error infrastructure.
- **Dependency Management**: correctly added `zip` and `serde_yaml` to `Cargo.toml` with appropriate feature flags.

---

#### 🧩 FractalFlow Check

| 检查项 | 状态 | 备注 |
| :--- | :--- | :--- |
| Header 完整性 | ✅ Pass | 所有新文件包含完整 Header |
| 语义链接有效性 | ✅ Pass | 指向 Go 源码和 Design 文档 |
| .folder.md 一致性 | ✅ Pass | 已更新 `services` 和 `commands` 的文档 |
| 中文注释 | ✅ Pass | 注释清晰且为中文 |

---

#### 📋 OpenSpec 符合度检查

**关联规范**: `openspec/changes/migrate-to-tauri-stack/design.md`

| Requirement | 实现状态 | 备注 |
| :--- | :--- | :--- |
| MCP Service 结构体 | ✅ | |
| list_servers/save_servers | ✅ | |
| sync_claude_servers | ✅ | 支持双向同步 logic |
| sync_codex_servers | ✅ | 支持 TOML experimental 字段 |
| import_from_claude | ✅ | |
| Skill Service 结构体 | ✅ | |
| list_skills/repos | ✅ | |
| install/uninstall_skill | ✅ | 支持 ZIP 下载解压 |
| skill.yaml 解析 | ✅ | 使用 serde_yaml |
| Tauri Commands | ✅ | 接口定义一致 |

---

#### 🎭 Mock 数据诚信度检查

| 检查项 | 状态 | 备注 |
| :--- | :--- | :--- |
| 生产代码无硬编码 Mock 数据 | ✅ Pass | 使用真实文件 IO 和网络请求 |
| 数据来源真实性 | ✅ Pass | 真实下载 ZIP，真实读写配置 |
| 业务逻辑完整性 | ✅ Pass | 完整实现同步和安装逻辑 |
| Mock 仅限测试代码 | ✅ Pass | 仅在 tests 模块中使用 mock |

---

#### ✅ 测试覆盖

- [x] 单元测试已添加 (`mcp_service.rs`, `skill_service.rs` 底部)
- [ ] 集成测试尚未编写 (Phase 7)
- [ ] **提示**: 由于 `root_path` 缺乏，部分 IO 逻辑未能在单元测试中完全覆盖。

---

#### 📝 审查结论

本次变更完整实现了 Phase 4 的目标功能，代码质量高，符合项目规范。逻辑实现稳健，特别是在异步处理方面表现良好。建议在后续迭代中优化测试性支持。

**建议操作**:

- [x] 直接合并 (当前实现已满足要求)
- [ ] 考虑在 Phase 7 或后续重构中添加 `root_path` 支持以增强测试覆盖率。
