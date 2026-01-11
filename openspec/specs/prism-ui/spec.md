# prism-ui Specification

## Purpose
Defines the visual and interactive standards for the Prism UI system, focusing on aesthetic quality, glassmorphism, and motion.

## Requirements
### Requirement: Ocean Spectrum Heatmap

热力图 **MUST** 使用蓝色系（Level 1-4: `#E0F2FE` → `#0369A1`），取代绿色系。

#### Scenario: 查看 Ocean Spectrum 热力图
- **Given** 用户在主页面上
- **When** 我查看贡献热力图
- **Then** 单元格必须使用蓝色/青色渐变，不是绿色

### Requirement: Breathing Glow Animation

选中态 Capsule **MUST** 呈现 3.2s 周期的呼吸灯动效。

#### Scenario: 观察呼吸灯动效
- **Given** 我有多个 Provider Capsules
- **When** 一个 Capsule 被选中
- **Then** 它必须显示呼吸灯辉光动画，周期为 3.2 秒

### Requirement: Crystal Glassmorphism

Levitating Capsule **MUST** 呈现高品质玻璃态效果，在不同光线条件下保持清晰边缘和适度透明度。

#### Scenario: 在明亮环境下查看 Capsule
- **Given** 用户在明亮的办公室环境
- **And** 使用 Light Mode
- **When** 我查看 Provider Capsule
- **Then** Capsule 边缘必须锐利清晰
- **And** 背景模糊效果不得影响文字可读性
- **And** 整体感觉像高级珠宝，不是塑料

#### Scenario: 在暗光环境下查看 Capsule
- **Given** 用户在暗光环境（夜间/暗室）
- **And** 使用 Dark Mode
- **When** 我查看 Provider Capsule
- **Then** Capsule 必须呈现霓虹内发光效果
- **And** 边缘必须有微妙的光晕但不刺眼

### Requirement: Dashboard Metric Legibility

展开后的 Dashboard 指标卡 **MUST** 在玻璃背景上保持高对比度可读性。

#### Scenario: 查看展开的 Dashboard 指标
- **Given** 一个 Provider Capsule 已展开
- **When** 我查看 Success Rate、Requests、Tokens、Cost 指标
- **Then** 所有数字必须清晰可读
- **And** 数字必须有适度的立体感（阴影/光晕）
- **And** Cost 卡的金色文字在 Dark Mode 下必须醒目

### Requirement: Unified Health Color Tokens

健康状态颜色 **MUST** 使用统一的 CSS 变量，禁止硬编码。

#### Scenario: 查看健康状态指示器
- **Given** 任何显示健康状态的组件
- **When** 状态为 Healthy/Warning/Critical
- **Then** 颜色必须来自 `--capsule-ring-healthy/warning/critical` 变量
- **And** Light Mode 和 Dark Mode 必须有各自优化的色值

