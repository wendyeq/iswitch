# Spec: Desktop Window & UI

## ADDED Requirements

### Requirement: 动态 HUD 窗口创建
系统 MUST 能够动态创建 HUD 副窗口，无需重启应用。

#### Scenario: 打开 HUD 窗口
- **GIVEN** 应用已启动且代理服务正在运行
- **WHEN** 用户点击托盘菜单 "打开 Mini HUD"
- **THEN** 一个约 280x120 的无边框窗口在屏幕右上角显示
- **AND** 窗口默认透明且始终置顶

#### Scenario: 关闭 HUD 窗口
- **GIVEN** Mini HUD 窗口正在显示
- **WHEN** 用户点击托盘菜单 "关闭 Mini HUD" 或按下 Escape 键（Interactive 模式下）
- **THEN** HUD 窗口关闭
- **AND** 当前窗口位置被保存到配置文件

#### Scenario: HUD 窗口属性
- **GIVEN** HUD 窗口已创建
- **WHEN** 窗口显示
- **THEN** 窗口具有以下属性：透明背景、无边框、始终置顶、不显示在任务栏

### Requirement: Click-Through 交互 (仅 macOS)
HUD 窗口 MUST 支持在 "交互模式" 和 "穿透模式" 之间切换。

> **平台限制**: 此功能使用 `NSWindow.setIgnoresMouseEvents_` 实现，仅支持 macOS。Windows 和 Linux 上 HUD 窗口默认为可交互模式。

#### Scenario: 穿透模式 (Ghost Mode)
- **GIVEN** HUD 窗口正在显示且运行在 macOS 上
- **WHEN** 用户未按住任何修饰键
- **THEN** 鼠标点击穿透 HUD 区域，传递给底层应用（如 VS Code）

#### Scenario: 交互模式切换
- **GIVEN** HUD 窗口处于穿透模式
- **WHEN** 用户按住 `Alt` (Option) 键
- **THEN** HUD 变为可交互状态（可拖拽）
- **AND** 释放按键后立即恢复穿透模式

#### Scenario: Windows/Linux 默认行为
- **GIVEN** HUD 窗口正在显示且运行在 Windows 或 Linux 上
- **WHEN** 用户与 HUD 交互
- **THEN** HUD 窗口始终可交互（可拖拽、可点击）
- **AND** 不支持穿透模式

### Requirement: 实时视觉反馈
UI MUST 使用平滑动画显示流式指标，并清晰区分活动状态。

#### Scenario: 脉冲动画与状态指示
- **GIVEN** Mini HUD 正在显示
- **WHEN** AI 请求开始（Streaming 状态）
- **THEN** Status 区域显示蓝紫色波浪动画
- **AND** Status 图标框显示数据脉冲闪烁效果

#### Scenario: Idle 状态视觉
- **GIVEN** 最后一次数据接收已超过 3 秒
- **WHEN** 系统判定为 Idle 状态
- **THEN** Status 图标变为紫色月亮图标（🌙）
- **AND** 文本显示 "Idle"
- **AND** 背景呈现静止状态

#### Scenario: 数值自动重置
- **GIVEN** HUD 处于 Idle 状态
- **WHEN** 无活动持续超过 30 秒
- **THEN** 所有数值（Speed, Tokens, Cost）重置为 0
- **AND** Model 名称显示为 "-"

#### Scenario: Model 区域布局
- **GIVEN** Model 名称较长
- **WHEN** 显示在 HUD 中
- **THEN** Model 图标显示为橙色 AI 芯片图标
- **AND** 名称超出部分用省略号 (...) 显示，确保布局不崩坏
