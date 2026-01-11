### Code Review Report

**目标**: Git Staged Changes (refactor-provider-capsules)
**审查时间**: 2026-01-09 12:45
**总体评分**: 8 / 10

---

#### 📋 变更概览

- **变更文件数**: 6 (Code: 4, Spec: 2 related checks)
- **重点组件**: `LevitatingCapsule.vue`, `LevitatingProviderList.vue`

---

#### 🔴 Critical Issues (必须修复)

- **[Testing] 缺失单元测试**
  > **描述**: 新增的组件 `LevitatingCapsule.vue` 和 `LevitatingProviderList.vue` 没有对应的测试文件（如 `*.test.ts`）。
  > **建议**: 必须为这两个核心 UI 组件编写单元测试，覆盖展开/收起、拖拽事件发射、数据展示逻辑。

- **[Spec Conflict] 功能移除未更新 Spec** (Type C)
  > **描述**: 代码中注明 `// Flow Line 已移除 (Jobs Mode)`，但 Spec (`specs/provider-capsules/spec.md`) 中仍包含 **Requirement: Flow Line Visualization**。
  > **建议**: 如果确认移除 Flow Line，**必须同步更新 Spec** 删除该 Requirement，否则视为实现缺失。

---

#### 🟡 Improvements (建议改进)

- **[FractalFlow] 缺失局部 .folder.md**
  > **描述**: `iswitch-tauri/src/components/Main/` 目录下缺失 `.folder.md` 文件。虽然 `src/.folder.md` 可能涵盖了它，但作为主要功能模块，建议添加局部 `.folder.md` 以明确该目录的职责和 Input/Output 定义。

- **[Cleanup] 遗留注释**
  > **描述**: 存在 `// Flow Point 已移除 (Jobs Mode)` 等注释。
  > **建议**: 既然功能已移除，相关的 CSS 类或无用代码（如果还有）应清理干净，或保留注释但说明原因（已说明）。

---

#### 🟢 Good Practices (值得肯定)

- **Header 规范**: 所有 Vue 文件都包含了完整的 FractalFlow Header，且链接有效。
- **Mock 分离**: 生产代码中使用了真实数据 (`fetchProviderDailyStats`)，仅 Sparkline 使用了算法生成的视觉装饰数据（符合 Spec "visual decoration" 定义）。
- **样式分离**: `style.css` 很好地利用 CSS 变量实现了 Dark/Light 模式切换（Ocean Depth / Milky Glass）。

---

#### 🧩 FractalFlow Check

| 检查项            | 状态              | 备注                   |
| ----------------- | ----------------- | ---------------------- |
| Header 完整性     | ✅ Pass           |                        |
| 语义链接有效性    | ✅ Pass           | Checked `cards.ts` etc |
| .folder.md 一致性 | ⚠️ Warning        | `src/components/Main/` 缺失 .folder.md |
| 中文注释          | ✅ Pass           |                        |

---

#### 📋 OpenSpec 符合度检查

**关联规范**: `openspec/changes/refactor-provider-capsules/specs/provider-capsules/spec.md`

| Requirement        | 实现状态       | 冲突类型 | 备注       |
| ------------------ | -------------- | -------- | ---------- |
| Visual Theme       | ✅ 已实现      | -        | Ocean Depth & Milky Glass implemented |
| Levitating Sorting | ✅ 已实现      | -        | Drag & Drop logic present |
| Expandable Details | ✅ 已实现      | -        | Toggle logic present |
| **Flow Line**      | ❌ 未实现      | **C**    | 代码中已移除，Spec 仍保留 |
| Data Metrics       | ✅ 已实现      | -        | 4 metrics implemented |
| Component Replacement | ✅ 已实现   | -        | Index.vue updated |

---

#### 🎭 Mock 数据诚信度检查

| 检查项                     | 状态              | 备注                      |
| -------------------------- | ----------------- | ------------------------- |
| 生产代码无硬编码 Mock 数据 | ✅ Pass           | Sparkline 仅为视觉装饰    |
| 数据来源真实性             | ✅ Pass           | 使用真实 API 数据         |
| 业务逻辑完整性             | ✅ Pass           |                           |
| Mock 仅限测试代码          | ✅ Pass           |                           |

---

#### ✅ 测试覆盖

- [ ] 单元测试已更新/添加 (MISSED)
- [ ] 集成测试已验证 (MISSED)
- [ ] 边缘情况已覆盖 (MISSED)
- [ ] **提示**: 若测试缺失或覆盖率低，建议运行 `/unit-test-generator`

---

#### 📝 审查结论

代码质量较高，视觉效果实现完善。主要问题在于 **Spec 与代码的冲突** (Flow Line) 以及 **测试缺失**。

**建议操作**:
1. **更新 Spec**: 删除 Flow Line 相关 Requirement。
2. **补全测试**: 运行 `/unit-test-generator` 为 `LevitatingCapsule.vue` 生成测试。
3. **补充文档**: 在 `src/components/Main/` 添加 `.folder.md`。

