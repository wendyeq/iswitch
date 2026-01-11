---
name: unit-test-generator
description: Generate high-quality unit tests for Frontend (.tsx/.ts), Python Backend (.py), and Rust Backend (.rs) files. Use when users request to create, add, or generate unit tests for specific source files. Triggers on phrases like "生成测试", "create tests", "add unit tests", or when a test file is explicitly requested.
---

# 单元测试生成器

本 skill 为 Frontend (React/TypeScript)、Python Backend (FastAPI) 和 Rust Backend 提供高质量单元测试的自动化生成能力。

## 快速开始

为源文件生成测试的基本流程:

1. **确定目标文件** - 识别用户请求的源文件
2. **关联 OpenSpec** (可选) - 查找验收标准和 Scenario
3. **分析源文件** - 理解代码逻辑和依赖关系
4. **生成测试** - 按技术栈规范编写测试
5. **验证运行** - 执行测试确保通过

## 测试生成工作流

### 分支 A: Frontend (.tsx / .ts)

**测试文件路径**: 与源文件同级
```
src/components/MyComponent.tsx → src/components/MyComponent.test.tsx
```

**生成步骤**:

1. 读取源文件并分析组件逻辑
2. 确定需要的 Mock (API hooks, Router, i18n)
3. 编写测试用例:
   - 组件渲染测试
   - 用户交互测试
   - 边界条件和错误状态测试
4. 运行验证:
   ```bash
   npm run test -- --run "src/path/to/Component.test.tsx"
   ```

### 分支 B: Python Backend (.py)

**测试文件路径**: 映射到 `tests/unit` 目录
```
backend/app/services/market.py → backend/tests/unit/services/test_market.py
```

**生成步骤**:

1. 读取源文件并分析 API 逻辑
2. Mock 所有 Repository 和外部 Service
3. 编写测试用例:
   - 成功场景测试
   - 异常处理测试
   - 权限验证测试
4. 运行验证:
   ```bash
   cd backend && uv run pytest <测试文件绝对路径>
   ```

### 分支 C: Rust Backend (.rs)

**测试策略**:
- **单元测试**: 直接在源文件底部添加 `#[cfg(test)] mod tests`
- **集成测试**: 在 `src-tauri/tests/` 下创建测试文件

**生成步骤**:

1. 读取源文件并分析模块逻辑
2. 使用 `mockall` 或标准 Mock 隔离外部依赖
3. 编写 `#[test]` 测试用例
4. 运行验证:
   ```bash
   cargo test <test_module_name>
   ```

## 边界条件处理

在生成测试前，评估以下条件:

- **大文件 (>500 行)**: 询问用户是测试整个文件还是特定函数
- **类型定义文件**: 提示用户此类文件通常不需要单元测试
- **已有测试**: 默认追加新用例，除非用户要求覆盖

## 逻辑风险扫描

生成测试前，快速检查源代码:

| 风险类型 | 检查内容 |
|---------|---------|
| 错误处理遗漏 | 异常未捕获、Promise 未 catch |
| 边界条件 | 空数组/空对象、null/undefined、除零保护 |
| 逻辑错误 | 条件判断反转、循环边界错误 |
| 硬编码 | 魔法数字/字符串未提取为常量 |

**处理策略**:
- 未发现问题 → 直接生成测试
- 发现疑似问题 → 向用户报告并确认是否继续

## 参考资源

本 skill 包含以下参考资源，按需加载:

- **[testing-guidelines.md](testing-guidelines.md)**: 详细的技术规范和代码示例
  - Frontend: Header 协议、Mocking 模式、查询优先级
  - Backend: Mocking 策略、Repository Mock 示例
  - 覆盖率目标: >= 80% 行覆盖率

## 测试质量标准

所有测试必须满足:

1. **Header 协议**: 包含 FractalFlow 语义链接
2. **原子性**: 每个 test/it 只测试一个逻辑概念
3. **独立性**: 测试之间不共享状态
4. **覆盖率**: >= 80% 行覆盖率
5. **必测场景**:
   - Happy Path (主要成功路径)
   - Error Boundary (错误处理)
   - Loading States (Frontend)
   - Empty States (空数据展示)
