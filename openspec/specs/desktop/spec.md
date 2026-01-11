# desktop Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-stack. Update Purpose after archive.
## Requirements
### Requirement: 系统托盘
系统 SHALL 在系统托盘显示图标并提供快捷操作。

#### Scenario: 托盘图标显示
- **GIVEN** 应用已启动
- **WHEN** 应用初始化完成
- **THEN** 系统托盘显示应用图标
- **AND** 图标适配系统主题 (明/暗)

#### Scenario: 托盘右键菜单
- **GIVEN** 托盘图标可见
- **WHEN** 用户右键点击托盘图标
- **THEN** 显示菜单: "显示主窗口"、"退出"

#### Scenario: 托盘点击打开窗口
- **GIVEN** 主窗口已隐藏
- **WHEN** 用户左键点击托盘图标
- **THEN** 主窗口显示并获得焦点

---

### Requirement: 窗口管理
系统 SHALL 提供窗口状态管理功能。

#### Scenario: 关闭时隐藏
- **GIVEN** 主窗口可见
- **WHEN** 用户点击窗口关闭按钮
- **THEN** 窗口隐藏而非退出应用
- **AND** 托盘图标保持可见

#### Scenario: 窗口居中
- **GIVEN** 应用首次启动
- **WHEN** 主窗口显示
- **THEN** 窗口在屏幕中央显示

#### Scenario: 最小化恢复
- **GIVEN** 主窗口已最小化
- **WHEN** 用户通过托盘或 Dock 激活应用
- **THEN** 窗口从最小化恢复
- **AND** 窗口获得焦点

---

### Requirement: Dock 图标控制 (macOS)
系统 SHALL 在 macOS 上控制 Dock 图标的显示。

#### Scenario: 隐藏 Dock 图标
- **GIVEN** 运行在 macOS 上
- **AND** 主窗口被隐藏
- **WHEN** 窗口关闭事件触发
- **THEN** Dock 图标隐藏

#### Scenario: 显示 Dock 图标
- **GIVEN** 运行在 macOS 上
- **AND** Dock 图标已隐藏
- **WHEN** 主窗口重新显示
- **THEN** Dock 图标显示

---

### Requirement: 应用生命周期
系统 SHALL 正确处理应用启动和退出。

#### Scenario: 应用启动
- **GIVEN** 用户启动应用
- **WHEN** 应用初始化
- **THEN** 启动代理服务器
- **AND** 显示主窗口
- **AND** 创建系统托盘

#### Scenario: 应用退出
- **GIVEN** 用户选择 "退出" 操作
- **WHEN** 退出事件触发
- **THEN** 停止代理服务器
- **AND** 保存当前设置
- **AND** 清理系统资源

#### Scenario: macOS 重新打开
- **GIVEN** 运行在 macOS 上
- **AND** 主窗口已隐藏
- **WHEN** 用户点击 Dock 图标
- **THEN** 主窗口重新显示并获得焦点

---

### Requirement: 版本信息显示
应用前端界面 **必须 (SHALL)** 在设置页面显示当前应用版本号，使用户能够确认当前运行的版本。

#### Scenario: 用户查看应用版本
- **GIVEN** 用户打开 iSwitch 应用
- **WHEN** 用户导航到设置页面
- **THEN** 页面底部显示当前版本号，格式为 "iSwitch v{x.y.z}"
- **AND** 版本号与 `tauri.conf.json` 中配置的版本一致

#### Scenario: 版本号格式规范
- **GIVEN** 应用已配置版本号
- **WHEN** 前端获取版本信息
- **THEN** 版本号 **必须 (SHALL)** 遵循语义化版本规范 (SemVer)
- **AND** 格式为 `MAJOR.MINOR.PATCH`（例如 `1.0.0`, `2.3.1`）

#### Scenario: 版本信息国际化
- **GIVEN** 用户使用中文或英文界面
- **WHEN** 查看版本信息
- **THEN** 版本标签文本根据当前语言显示
  - 中文: "版本 1.0.0" 或 "iSwitch v1.0.0"
  - 英文: "Version 1.0.0" 或 "iSwitch v1.0.0"

### Requirement: 动态 HUD 窗口创建
系统 MUST 能够动态创建 HUD 副窗口，**每次打开都初始化到主屏右上角**。

#### Scenario: 打开 HUD 窗口 (MODIFIED)
- **GIVEN** 应用已启动且代理服务正在运行
- **WHEN** 用户点击托盘菜单 "打开 Mini HUD"
- **THEN** 一个约 180x210 的无边框窗口在**主屏幕右上角**显示
- **AND** 窗口位置为 (屏幕宽度 - 180 - 20, 20) 逻辑像素
- **AND** 根据配置决定是否置顶（默认 False）

#### Scenario: 多屏适配与智能恢复 (SMART)
- **GIVEN** 用户连接了多个显示器
- **WHEN** 用户将 HUD 拖动到副屏
- **THEN** HUD 保持在副屏该位置
- **WHEN** 用户关闭应用并重新打开
- **THEN** HUD **恢复到副屏该位置**（记忆生效）
- **WHEN** 用户**拔掉副屏**，并重新打开 HUD
- **THEN** 系统检测到坐标无效
- **AND** HUD **自动重置**到主显示器 (`primary_monitor`) 的右上角显示

#### Scenario: 智能位置持久化 (SMART)
- **GIVEN** 用户已拖动 HUD 到自定义位置
- **WHEN** 用户关闭应用并重新打开 HUD
- **THEN** HUD 窗口恢复到用户上次设定的位置
- **AND** 系统在后台验证该位置是否在当前可视区域内

#### Scenario: 会话内拖动有效 (NEW)
- **GIVEN** HUD 窗口正在显示
- **WHEN** 用户拖动 HUD 到其他位置
- **THEN** HUD 保持在新位置直到关闭
- **AND** 会话期间可自由拖动

---

### Requirement: Click-Through 交互 (仅 macOS)
HUD 窗口 MUST 使用图钉按钮控制位置锁定，锁定 = 禁止拖动（非点击穿透）。

#### Scenario: 默认可拖动模式
- **GIVEN** HUD 窗口已创建
- **WHEN** 窗口首次显示
- **THEN** 窗口处于**可拖动 (Draggable)** 模式
- **AND** 用户可以直接拖动 HUD 窗口到任意位置
- **AND** 右上角显示一个**圆形图钉按钮**（22px，带圆形边框，40% 透明度）

#### Scenario: 图钉锁定位置 (MODIFIED)
- **GIVEN** HUD 窗口正在显示
- **WHEN** 用户点击右上角的**图钉按钮**
- **THEN** 窗口进入**锁定 (Locked)** 模式
- **AND** 图钉按钮变为**实心圆点 + 发光效果**：
  - 亮色模式: Apple Blue (#0a84ff) + 蓝色发光
  - 暗色模式: Neon Cyan (#00f3ff) + 青色发光
- **AND** 拖动操作被**禁用**
- **AND** 图钉按钮**始终可点击**（不受锁定影响）

#### Scenario: 图钉解锁位置 (NEW)
- **GIVEN** HUD 处于锁定模式
- **WHEN** 用户再次点击图钉按钮
- **THEN** 窗口恢复**可拖动**模式
- **AND** 图钉按钮恢复默认样式

#### Scenario: 快捷键关闭 (SIMPLIFIED)
- **GIVEN** HUD 窗口正在显示
- **WHEN** 用户按下 `Escape` 键
- **THEN** HUD 窗口关闭

> **设计决策**: 移除 Click-Through 模式和 Alt 键临时解锁功能。
> 原因：Click-Through 模式会导致整个窗口（包括图钉按钮）的鼠标事件穿透，
> 无法再次点击解锁。改用简单的"禁止拖动"方式实现位置锁定，
> 图钉按钮始终保持可交互状态，用户体验更佳。

---

### Requirement: 实时视觉反馈
UI MUST 使用极简设计展示流式指标，**数字直接漂浮在磨砂玻璃上**，无标签、无卡片边框。

#### Scenario: HUD 极简布局 (MODIFIED)
- **GIVEN** Mini HUD 正在显示
- **WHEN** 用户查看 HUD 内容
- **THEN** 显示以下 3 个层级的指标（按视觉权重）：
  1. **Hero (Speed)**: 42px 超大轻字重数字 + "tok/s" 单位
  2. **Secondary (Tokens)**: 22px 中等数字 + "tokens" 单位
  3. **Tertiary (Model)**: 11px 极淡大写字母，几乎融入背景
- **AND** **无任何标签** (如 "Speed:", "Total Tokens:")
- **AND** **无内部卡片背景**，数字直接漂浮
- **AND** 使用**发丝分割线**分隔层级

#### Scenario: 状态光晕效果 - Streaming (MODIFIED)
- **GIVEN** 系统正在处理 AI 请求（Streaming 状态）
- **WHEN** HUD 正在显示
- **THEN** **整个玻璃容器**显示缓慢呼吸的光晕（非单个卡片）
- **AND** 光晕颜色使用 `var(--hud-streaming-glow)`
- **AND** 呼吸周期为 2.5 秒

#### Scenario: 状态光晕效果 - Idle (UNCHANGED)
- **GIVEN** 系统处于空闲状态
- **WHEN** HUD 显示
- **THEN** 无光晕效果
- **AND** 从 Streaming 过渡到 Idle 时有 0.4s 渐隐动画

#### Scenario: 窗口尺寸 (UNCHANGED)
- **GIVEN** 用户打开 Mini HUD
- **WHEN** HUD 窗口创建
- **THEN** 窗口尺寸为 180x210 像素

---

### Requirement: Mini HUD 主题同步
Mini HUD 窗口 SHALL 通过使用共享主题 Token 并响应运行时主题更改来镜像 Prism 亮色/暗色色板，而无需重启窗口。

#### Scenario: 亮色模式 HUD
- **GIVEN** 操作系统设置为亮色模式
- **WHEN** HUD 窗口打开
- **THEN** 其卡片、上下文菜单和状态芯片必须使用为主页定义的相同亮色玻璃颜色（变量如 `--capsule-accent-azure`, `--surface-card`）
- **AND** 文本/图标必须保持为主页指定的对比度 (>= 4.5:1)，完全由 Media Query 驱动，忽略主窗口状态。

#### Scenario: 暗色模式 HUD
- **GIVEN** 操作系统切换到暗色 (System Dark Mode)，同时 HUD 打开
- **WHEN** 系统 `prefers-color-scheme` 更改
- **THEN** HUD 必须检测到更改并在 300ms 内切换到暗色霓虹色板，忽略主窗口的主题设置。

### Requirement: HUD 智能位置记忆
系统 MUST 能够保存和恢复 HUD 窗口位置，并在启动时验证位置有效性。当保存的位置无效时（例如屏幕断开），系统 MUST 自动重置到主显示器右上角。

#### Scenario: 位置有效性检测与自动重置
- **GIVEN** 用户上次将 HUD 放置在副屏位置并关闭应用
- **WHEN** 用户断开副屏并重新打开应用
- **THEN** 系统检测到保存的坐标不在当前任何显示器范围内
- **AND** 系统自动将 HUD 位置重置到主显示器右上角
- **AND** HUD 正常显示

---

