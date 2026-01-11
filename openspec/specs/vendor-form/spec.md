# vendor-form Specification

## Purpose
TBD - created by archiving change redesign-vendor-form. Update Purpose after archive.
## Requirements
### Requirement: Single Input Receptacle

供应商接收槽 **MUST** 采用单一输入槽设计，实现渐进式展开。

#### Scenario: URL 自动识别成功
- **Given** 用户打开新增供应商弹窗
- **When** 用户粘贴 `https://api.openai.com/v1`
- **Then** 系统自动识别为 OpenAI 供应商
- **And** 自动填充名称为 "OpenAI"
- **And** 自动显示 OpenAI 图标
- **And** 仅展示 API Key 输入框

#### Scenario: URL 无法自动识别
- **Given** 用户粘贴了未知的 API 地址
- **When** 系统无法自动识别供应商类型
- **Then** 展开显示所有字段（名称、API Key）
- **And** 名称字段预填充域名作为默认值

### Requirement: Context-Aware Detection

系统 **MUST** 将用户所处的 Tab 上下文作为第一优先级判定依据，并在冲突时提供智能纠正。

#### Scenario: Perfect Match (Claude Tab + Anthropic URL)
- **Given** 用户处于 "Claude" Tab
- **And** 用户打开接收槽
- **When** 用户粘贴 `https://api.anthropic.com`
- **Then** 接收槽立即锁定
- **And** 供应商类型确认为 "Claude"
- **And** 图标显示 Claude

#### Scenario: Auto Correct (Claude Tab + Codex URL)
- **Given** 用户处于 "Claude" Tab
- **And** 用户打开接收槽
- **When** 用户粘贴 `https://api.openai.com/v1` (或 Azure URL)
- **Then** 接收槽执行 Haptic Shake (轻微振动)
- **And** 显示提示 "已切换至 Codex (OpenAI 兼容模式)"
- **And** 系统自动将供应商归类为 "Codex"

#### Scenario: Ambiguity Handling (Claude Tab + Generic Proxy)
- **Given** 用户处于 "Claude" Tab
- **And** 用户粘贴 `https://api.third-party.com/v1` (无明显特征)
- **Then** 系统默认将其视为 "Claude" 类型
- **And** 接收槽展开一个 "Protocol Toggle" 开关
- **And** 开关显示 "Native" (默认选中) vs "Compatible"

### Requirement: Concentric Corners Mathematics

所有圆角值 **MUST** 遵循同心圆角数学原则：`R_inner = R_outer - Padding`

#### Scenario: 圆角数学验证
- **Given** Modal 外框圆角为 24px
- **And** 内容区到边缘 Padding 为 20px
- **When** 渲染输入框
- **Then** 输入框圆角 **MUST** 为 4px (24 - 20 = 4)

### Requirement: Physically Correct Lighting

弹窗光影 **MUST** 遵循物理正确原则，光源假定来自上方 45°。

#### Scenario: Light Mode 光影
- **Given** 应用处于 Light Mode
- **When** 弹窗显示
- **Then** 顶部边缘有高光 (`border-top` 较亮)
- **And** 底部边缘较暗 (`border-bottom` 微深)
- **And** 阴影分为两层：近景软阴影 + 远景环境光遮挡

#### Scenario: Dark Mode 光影
- **Given** 应用处于 Dark Mode
- **When** 弹窗显示
- **Then** 背景为深色磨砂玻璃
- **And** 有淡蓝色环境光微发光

### Requirement: Spring Physics Animation

所有交互动效 **MUST** 使用弹簧物理曲线，而非线性或简单的 ease。

#### Scenario: 输入框 Focus 反馈
- **Given** 用户点击输入框
- **When** 输入框获得焦点
- **Then** 输入框轻微放大 `scale(1.005)`
- **And** 使用弹簧缓动 `cubic-bezier(0.34, 1.56, 0.64, 1)`
- **And** 边框渐变为 Azure 蓝色

#### Scenario: 验证错误反馈
- **Given** 用户输入了无效的 URL
- **When** 触发验证
- **Then** 输入框执行 shake 动画
- **And** 边框变为 Coral Red
- **And** 不显示独立的错误提示框

### Requirement: Gesture-First Dismissal

弹窗关闭 **MUST** 支持手势操作，无需显式关闭按钮。

#### Scenario: 点击遮罩关闭
- **Given** 弹窗处于打开状态
- **When** 用户点击弹窗外的遮罩区域
- **Then** 弹窗以消散动画关闭

#### Scenario: 按 Esc 键关闭
- **Given** 弹窗处于打开状态
- **When** 用户按下 Esc 键
- **Then** 弹窗以消散动画关闭

### Requirement: Emergence Animation

弹窗进入 **MUST** 使用浮现动画，而非简单的滑入。

#### Scenario: 弹窗浮现
- **Given** 用户触发新增供应商
- **When** 弹窗开始显示
- **Then** 弹窗从下方浮现 (`translateY(12px) → 0`)
- **And** 同时从微缩状态恢复 (`scale(0.98) → 1`)
- **And** 伴随解模糊效果 (`blur(4px) → 0`)
- **And** 动画时长为 280ms
- **And** 缓动函数为 `cubic-bezier(0.16, 1, 0.3, 1)`

### Requirement: Dissolution Animation

弹窗离开 **MUST** 使用消散动画。

#### Scenario: 弹窗消散
- **Given** 用户关闭弹窗
- **When** 弹窗开始隐藏
- **Then** 弹窗微缩 (`scale(0.96)`)
- **And** 同时淡出 (`opacity → 0`)
- **And** 伴随模糊效果 (`blur(2px)`)
- **And** 动画时长为 200ms

### Requirement: Create Vendor Functionality

新增供应商 **MUST** 正确保存数据并刷新列表。

#### Scenario: 成功新增供应商（自动识别）
- **Given** 用户在接收槽中粘贴 `https://api.openai.com/v1`
- **And** 系统自动识别为 OpenAI
- **And** 用户填写了 API Key
- **When** 用户点击「添加」按钮
- **Then** 弹窗以消散动画关闭
- **And** 新供应商出现在列表末尾
- **And** 数据被持久化到后端

#### Scenario: 成功新增供应商（手动填写）
- **Given** 用户粘贴了未识别的 URL
- **And** 用户手动填写了名称和 API Key
- **When** 用户点击「添加」按钮
- **Then** 弹窗关闭
- **And** 新供应商使用用户填写的名称

#### Scenario: URL 验证失败
- **Given** 用户输入了无效 URL `not-a-url`
- **When** 用户尝试提交
- **Then** 输入框执行 shake 动画
- **And** 输入框边框变红
- **And** 弹窗不关闭

### Requirement: Edit Vendor Functionality

编辑供应商 **MUST** 正确加载和更新现有数据。

#### Scenario: 编辑模式数据回显
- **Given** 列表中存在一个名为 "AICoding" 的供应商
- **When** 用户点击该供应商的「编辑」操作
- **Then** 所有字段预填充该供应商的现有数据
- **And** URL 字段为只读状态（不可修改）
- **And** 名称字段为只读状态

#### Scenario: 成功更新供应商
- **Given** 用户在编辑模式中修改了 API 密钥
- **When** 用户点击「保存」按钮
- **Then** 弹窗关闭
- **And** 供应商数据被更新

### Requirement: Deferred Configuration

高级配置项（模型白名单、模型映射）**MUST NOT** 在新增流程中直接展示，必须在编辑模式或通过"高级设置"展开。

#### Scenario: 初始隐藏
- **Given** 用户正在新增供应商
- **Then** 界面不显示"模型白名单"输入框
- **And** 界面不显示"模型映射"配置区
- **And** 界面保持极简

#### Scenario: 编辑模式展开
- **Given** 用户进入编辑模式
- **When** 用户点击 "Advanced Settings" (高级设置)
- **Then** 以手风琴动画展开白名单和映射配置区

### Requirement: Smart Dashboard Link

供应商 Logo/名称 **MUST** 作为通往官方控制台/账单页面的入口。

#### Scenario: 仪表盘跳转
- **Given** 供应商识别为 OpenAI
- **When** 用户点击供应商 Logo
- **Then** 浏览器打开 `https://platform.openai.com/account/billing`
- **And** 鼠标 Hover 时显示 "Open Dashboard ↗" 提示

---

