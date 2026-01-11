# capsule-nav Specification

## Purpose
TBD - created by archiving change implement-capsule-navigation. Update Purpose after archive.
## Requirements
### Requirement: Centralized Navigation Container

应用程序 MUST (必须) 渲染一个"胶囊导航 (Capsule Navigation)"组件，包含 **3 个** 核心操作按钮。

#### Scenario: Verify Capsule Existence and Placement (MODIFIED)
- **Given** 应用程序加载在 "Main" 视图
- **When** 用户观察界面
- **Then** 一个可见的"胶囊"容器应该出现（例如在视口底部）
- **And** 它应该包含确切的 **3 个** 操作图标：设置、主题、日志
- **And** 胶囊中 **不再** 包含 MCP 和 Skill 图标

---

### Requirement: Actions Functionality
将按钮移动到胶囊中 MUST NOT (不得) 破坏其现有功能。

#### Scenario: Toggle Theme via Capsule
- **Given** 当前主题是 "Light" (亮色)
- **When** 用户点击胶囊中的 "切换主题" 图标
- **Then** 应用程序主题应切换为 "Dark" (暗色)。

#### Scenario: Open Settings via Capsule
- **Given** 用户在 Main 视图
- **When** 用户点击胶囊中的 "设置" 图标
- **Then** 设置视图（或模态框）应该打开。

#### Scenario: Open MCP Panel via Capsule
- **Given** 用户在 Main 视图
- **When** 用户点击胶囊中的 "MCP" 图标
- **Then** 应用程序应导航到 MCP 视图。

#### Scenario: Open Skill List via Capsule
- **Given** 用户在 Main 视图
- **When** 用户点击胶囊中的 "Skill" 图标
- **Then** 应用程序应导航到 Skill/Tools 视图。

#### Scenario: View Logs via Capsule
- **Given** 用户在 Main 视图
- **When** 用户点击胶囊中的 "查看日志" 图标
- **Then** 应用程序应导航到日志视图。

### Requirement: Integration - Retained Controls
`section-controls` 区域在移除导航按钮后 MUST (必须) 保留以下控件：
- **Relay Toggle** (代理开关)
- **Add Card 按钮** (添加卡片)

#### Scenario: Verify Retained Controls After Migration
- **Given** 胶囊导航已实现
- **When** 用户检查 `section-controls` 区域
- **Then** Relay Toggle 代理开关应仍然存在且功能正常
- **And** Add Card 按钮应仍然存在且功能正常
- **And** MCP、Skill、日志按钮应不存在（已迁移至胶囊）

### Requirement: Integration - Clean up Old Controls
先前位于 `global-actions` 和 `section-controls` 中的这些按钮实例 MUST (必须) 被移除，以避免重复。

#### Scenario: Verify Removal of Old Buttons
- **Given** 胶囊导航已实现
- **When** 用户检查 `global-actions` 区域
- **Then** 旧的主题和设置按钮不应存在。
- **When** 用户检查 `section-controls` 区域
- **Then** 旧的 MCP、Skill 和日志按钮不应存在。

### Requirement: Context Controls
The Capsule Navigation MUST host the primary context controls for the application.

#### Scenario: Switching Context
Given the user is on the main page
When the user clicks "Codex" in the capsule
Then the provider list updates to show Codex providers
And the "Enable Codex" toggle reflects the Codex proxy state

#### Scenario: Adding Supplier
Given the user is on the "Claude Code" tab
When the user clicks the "Add Supplier" (+) button in the capsule
Then the "Add Supplier" modal opens with "Claude Code" context

### Requirement: Layout
The capsule MUST accommodate the new controls without breaking layout on smaller screens.

#### Scenario: Visual Integrity
Given the application is running
When viewed on a standard desktop resolution
Then the capsule navigation items ARE clearly visible and interactive
And no elements overlap or overflow the capsule boundary

