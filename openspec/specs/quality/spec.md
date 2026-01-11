# quality Specification

## Purpose
Defines the quality assurance standards and workflows for the iSwitch project, including code review processes, testing coverage requirements, and automated testing workflows.
## Requirements
### Requirement: 单元测试覆盖率标准

开发工作流 SHALL 对核心模块维护最低 80% 的单元测试覆盖率，确保代码质量可追溯。

#### Scenario: Rust 后端覆盖率检查

- **GIVEN** 开发者需要验证 Rust 后端代码库 (`src-tauri/src/`) 的测试覆盖率
- **WHEN** 执行覆盖率检查命令
- **THEN** 使用 `cargo-tarpaulin` 或备选方案 (LLVM coverage + grcov) 生成报告
- **AND** 核心模块 (`proxy/`, `services/`) 覆盖率应达到 ≥ 80%
- **AND** 报告保存至 `reports/coverage/`

#### Scenario: Vue 前端覆盖率检查

- **GIVEN** 开发者需要验证 Vue 前端代码库 (`src/`) 的测试覆盖率
- **WHEN** 执行覆盖率检查 (`npm run test -- --coverage`)
- **THEN** 核心组件和服务覆盖率应达到 ≥ 60%
- **AND** 生成覆盖率摘要

#### Scenario: 覆盖率不足处理

- **GIVEN** 某模块覆盖率低于目标值
- **WHEN** 执行覆盖率检查
- **THEN** 输出明确的警告信息
- **AND** 列出未覆盖的文件和行
- **AND** 开发者应使用 `/unit-test-generator` 或人工补齐测试

---

### Requirement: Code Review 工作流规范

开发工作流 SHALL 在代码变更前执行系统化的 Code Review，并生成标准化报告。

#### Scenario: Code Review 执行

- **GIVEN** 开发者完成代码变更
- **WHEN** 执行 `/code-review` 工作流
- **THEN** Agent 对目标文件执行以下检查:
  - 功能性
  - 类型安全
  - 错误处理
  - FractalFlow 合规性
  - Mock 数据诚信度
- **AND** 生成结构化报告到 `reports/code-review/YYYY-MM-DD/`

#### Scenario: Critical Issue 处理

- **GIVEN** Code Review 发现 Critical Issue
- **WHEN** 报告生成完毕
- **THEN** 报告明确标注 Critical Issues
- **AND** 开发者应在解决后重新审查

#### Scenario: OpenSpec 符合度验证

- **GIVEN** 被审查的代码对应某个 OpenSpec spec
- **WHEN** 执行 Code Review
- **THEN** 验证代码是否实现了 spec 中的所有 Requirements
- **AND** 标注已实现和未实现的功能

---

### Requirement: 测试生成工作流规范

开发工作流 SHALL 支持通过 Agent 工作流生成单元测试，并明确人工介入点。

#### Scenario: Rust 测试生成 (Agent 执行)

- **GIVEN** 目标 Rust 源文件 (`.rs`)
- **WHEN** 开发者执行 `/unit-test-generator` 工作流
- **THEN** Agent 在源文件底部生成 `#[cfg(test)] mod tests`
- **AND** 测试覆盖主要逻辑分支
- **AND** 测试可通过 `cargo test`

#### Scenario: Vue 组件测试生成 (Agent + 人工)

- **GIVEN** 目标 Vue 组件文件 (`.vue`)
- **WHEN** 开发者执行 `/unit-test-generator` 工作流
- **THEN** Agent 生成同级测试文件 (`*.test.ts`)
- **AND** Agent 生成渲染测试和基础交互测试
- **AND** **开发者应人工补充关键交互测试用例**（如复杂状态变更、异步操作）
- **AND** 测试可通过 `npm run test`

#### Scenario: OpenSpec 驱动的测试设计

- **GIVEN** 目标代码关联某个 OpenSpec change
- **WHEN** 执行 `/unit-test-generator` 工作流
- **THEN** 从 `proposal.md` 验收标准生成测试用例
- **AND** 从 `spec.md` Scenarios 生成测试用例

