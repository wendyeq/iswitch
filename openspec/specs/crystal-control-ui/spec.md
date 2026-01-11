# crystal-control-ui Specification

## Purpose
TBD - created by archiving change redesign-settings-interface. Update Purpose after archive.
## Requirements
### Requirement: Floating Glass Architecture
The settings interface MUST be rendered as a non-modal, floating "slate" element that overlays the main content with a high-blur backdrop.

#### Scenario: Opening Settings
- **Given** the user is on the main dashboard
- **When** they click the "Settings" icon
- **Then** the background content calculates a blur radius of 5px
- **And** the Crystal Slate slides up from the bottom with a spring damping of 0.8.

### Requirement: Physic-Based Toggles
Boolean settings MUST be controlled by a custom "Physic Switch" providing haptic-like visual feedback.
- **Scope**: `Heatmap`, `Auto Start`.
- **Behavior**: Must include "Overshoot" and "Recoil" physics.

#### Scenario: Toggling Heatmap
- **Given** the heatmap is enabled
- **When** the user clicks the Heatmap toggle
- **Then** the toggle knob moves with a spring simulation
- **And** the background fills with "Prism Blue".

### Requirement: Focused Surface Controls
The interface MUST prioritize high-level intent over low-level parameters. Every control in this strata is about how the "Surface" behaves.

#### Scenario: Interaction Groups
- **Language Selection**: A minimal segmented control for `en` and `zh`.
- **Primary Toggles**: 
  - `Heatmap Visualization`: Toggle for main dashboard activity view.
  - `Auto Start`: Toggle for system-level startup behavior.

### Requirement: The Sync Portal
The configuration import functionality MUST be aggregated into a unified "Sync Portal" zone, providing entry points for different system ecosystems.

#### Scenario: Multi-Source Sync
- **Sources**: Support both `CC-Switch` (Provider configs) and `Code-Switch` (Project settings).
- **Indicator**: Each source is represented by a "Memory Crystal".
- **Visual State**: 
  - `Synced`: Green pulse indicator.
  - `Missing`: Amber warning/idle indicator.
  - `Pending`: Animated shimmer during operation.

### Requirement: Parameter Retirement
- The UI MUST NOT expose `failoverThreshold`, `recoveryTimeout`, or **network-level parameters like `proxyPort`**.
- **Logic**: These values are now autonomously managed or set during initial setup (e.g., `failoverThreshold` is optimized to **5** by default). The UI acts as a filter for complexity, ensuring the user only touches "Intent-Level" switches.

#### Scenario: Implicit Configuration
- **Given** the system defaults are loaded
- **When** the user opens settings
- **Then** `failoverThreshold` and `proxyPort` are NOT visible
- **And** they are managed internally by the autostart logic.

### Requirement: Crystal Slate Token 对齐
Crystal Control 覆盖层 SHALL 将其排版、边框和背景颜色建立为全局 Prism 主题 Token 的别名，以便在两种模式下在视觉上与主页匹配。

#### Scenario: 亮色模式 Slate
- **GIVEN** 应用程序在亮色模式下运行
- **WHEN** 用户打开 Crystal Control 路由或模态框
- **THEN** Slate 必须从共享 Token (`--mac-text`, `--mac-border`, `--capsule-bg`) 派生 `--text-primary`, `--text-secondary`, 边框和玻璃背景
- **AND** 半透明背景必须复用主页模糊层（相同的不透明度 + 模糊半径），而不是重新定义自己的 rgba 常量。

#### Scenario: 暗色模式 Slate
- **GIVEN** 暗色模式处于活动状态
- **WHEN** 用户打开 Crystal Control
- **THEN** Slate 必须通过引用共享 Token 的暗色变体自动切换到 Neon Abyss 色板
- **AND** 任何强调元素（如 Memory Crystal 同步指示器）必须映射到 `style.css` 中已存在的语义强调/危险颜色，以防止偏差。

