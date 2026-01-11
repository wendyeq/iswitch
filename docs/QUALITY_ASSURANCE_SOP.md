---
source: ../openspec/changes/establish-quality-standards/proposal.md ([POS]: Quality Standard Proposal)
---

# 质量保障标准作业程序 (SOP)

## 1. 概述

本稳定旨在建立 iSwitch 项目的系统化质量保障工作流，确保代码的高可用性、可维护性和稳定性。所有贡献者在提交代码前必须遵循本指南。

## 2. 开发工作流

### 2.1 提交前自查 (Pre-commit Checklist)

在发起 Pull Request 或提交代码前，开发者必须完成以下自查：

- [ ] **编译检查**: 确保 `cargo check` 和 `npm run type-check` 无错误。
- [ ] **单元测试**: 运行相关模块测试，确保全部通过。
  - Rust: `cargo test <module>`
  - Vue: `npm run test <component>`
- [ ] **覆盖率影响**: 确保新的核心逻辑有对应的测试覆盖。

### 2.2 测试规范

- **Rust 后端**:
  - **核心逻辑** (`src-tauri/src/proxy`, `src-tauri/src/services`): **必须** 包含单元测试。目标覆盖率 ≥ 80%。
  - **数据模型** (`src-tauri/src/models`): 关键序列化/反序列化逻辑需测试。
  - **工具链**: 使用 `cargo test`。
- **Vue 前端**:
  - **Service 层**: 所有 API 封装必须 Mock Tauri invoke 进行测试。
  - **Utils 层**: 纯逻辑函数必须 100% 覆盖。
  - **UI 组件**: 关键交互组件 (如 Settings, Logs) 必须包含交互测试。
  - **工具链**: 使用 `Vitest` + `Vue Testing Library`。

## 3. Code Review 规范

### 3.1 触发条件

- 任何涉及核心逻辑修改 (Rust Backend, Frontend Services) 的 PR 必须经过 Code Review。
- 可以使用 Agent `/code-review` 工作流辅助审查。

### 3.2 审查检查点 (Checklist)

审查者 (Reviewer) 需重点关注：

1. **安全性**:
   - 是否存在 SQL 注入风险？
   - 是否有路径遍历风险？
   - 敏感信息 (API Key) 是否有泄漏风险？
2. **正确性**:
   - 边界条件是否处理 (如空列表、网络超时)？
   - 错误处理是否优雅？
3. **可维护性**:
   - 是否遵循 FractalFlow 架构规范？
   - 命名是否清晰？
4. **测试覆盖**:
   - 新代码是否包含测试？

## 4. 覆盖率监控

### 4.1 报告生成

项目支持生成可视化的覆盖率报告：

- **Rust**:

  ```bash
  # 生成 HTML 报告至 reports/coverage/rust/
  cargo tarpaulin --out Html -o reports/coverage/rust/
  ```

  _(备选: 使用 grcov)_

- **Frontend**:
  ```bash
  # 生成报告至 reports/coverage/frontend/
  npm run test -- --coverage
  ```

### 4.2 覆盖率目标

| 模块类型            | 目标覆盖率 | 最低接受标准 |
| ------------------- | ---------- | ------------ |
| Rust Core           | 80%        | 60%          |
| Rust Utils          | 90%        | 70%          |
| Frontend Services   | 90%        | 70%          |
| Frontend Components | 50%        | 30%          |

## 5. 常见问题 (FAQ)

**Q: 我修改了 UI 样式，需要写测试吗？**
A: 不需要。纯样式修改不在测试强制范围内，但建议通过截图验证。

**Q: 为什么 tarpaulin 在我的 Mac 上运行失败？**
A: M系列芯片可能存在兼容性问题。请尝试使用 LLVM coverage + grcov 方案 (详见 `tasks.md`)。

**Q: Agent 生成的测试报错怎么办？**
A: Agent 生成的测试仅作为起步，开发者有责任修复并优化这些测试，确保其真实反映业务逻辑。
