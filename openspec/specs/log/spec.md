# log Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-stack. Update Purpose after archive.
## Requirements
### Requirement: 请求日志记录
系统 SHALL 记录所有经过代理的请求详情和 Token 用量。

#### Scenario: 记录请求日志
- **GIVEN** 代理完成一次请求转发
- **WHEN** 收到上游响应
- **THEN** 异步写入日志记录到 SQLite
- **AND** 包含: 平台、模型、供应商、成功状态、Token 用量

#### Scenario: Token 用量提取 (Claude)
- **GIVEN** 代理转发 Claude API 请求
- **WHEN** 响应中包含 usage 字段
- **THEN** 提取 input_tokens、output_tokens
- **AND** 提取 cache_creation_input_tokens、cache_read_input_tokens

#### Scenario: Token 用量提取 (Codex)
- **GIVEN** 代理转发 Codex API 请求
- **WHEN** 响应中包含 usage 字段
- **THEN** 提取 input_tokens、output_tokens、reasoning_tokens

---

### Requirement: 日志查询
系统 SHALL 提供日志查询和分页功能。

#### Scenario: 按平台筛选
- **GIVEN** 用户打开日志页面
- **WHEN** 用户选择 "claude" 平台
- **THEN** 仅显示 Claude 相关的请求日志

#### Scenario: 按供应商筛选
- **GIVEN** 用户打开日志页面
- **WHEN** 用户选择特定供应商
- **THEN** 仅显示该供应商的请求日志

#### Scenario: 分页加载
- **GIVEN** 存在大量日志记录
- **WHEN** 用户滚动到底部
- **THEN** 自动加载下一页日志
- **AND** 保持流畅的用户体验

#### Scenario: 分页大小 (Pagination Size)
- **GIVEN** 用户查看日志列表
- **WHEN** 加载日志数据
- **THEN** 每页默认显示 5 条记录
- **AND** 保持界面轻盈 (Reductive Design)

#### Scenario: 自动刷新与联动 (Auto-Refresh & Dependency)
- **GIVEN** 用户更改筛选条件
- **WHEN** 切换 "平台" (Platform)
- **THEN** 自动重置 "供应商" (Provider) 选项为空
- **AND** 立即刷新日志列表
- **WHEN** 切换 "供应商" 或 "时间范围"
- **THEN** 立即刷新日志列表，无需点击查询按钮

---

### Requirement: Token 展示与计算 (Token Display & Calculation)
系统 SHALL 采用极简方式展示 Token 用量，并在需要时提供细节 (Jony Ive "Reductive" Philosophy)。

#### Scenario: Token 总量计算
- **GIVEN** 一条请求日志
- **WHEN** 计算 `Total Tokens`
- **THEN** 公式为 `Input Tokens + Output Tokens`
- **AND** 不包含 Reasoning 或 Cache Tokens（避免双重计数，因 Provider API 通常已包含）

#### Scenario: 极简展示与详情浮层
- **GIVEN** 日志表格的一行
- **WHEN** 默认显示
- **THEN** 仅展示 `Total Tokens` 数值
- **WHEN** 用户鼠标悬停 (Hover)
- **THEN** 显示浮层 (Tooltip/Popover) 包含 Input, Output, Reasoning, Cache Reads, Cache Writes 详情
- **AND** 当 Reasoning 或 Cache 为 0 (或空) 时，在浮层中隐藏对应行，减少干扰

#### Scenario: 统计卡片布局
- **GIVEN** 顶部统计区域
- **THEN** 展示 4 张核心卡片：Total Requests, Total Tokens, Cache Reads, Total Cost


---

### Requirement: 费用计算

系统 SHALL 基于**配置文件中的模型定价**计算每次请求的费用。

#### Scenario: 配置文件加载

- **GIVEN** 应用启动
- **WHEN** 初始化定价服务
- **THEN** 优先加载 `~/.iswitch/model-pricing.json`
- **AND** 如用户配置不存在，加载内置 `resources/model-pricing.json`
- **AND** 如都失败，使用硬编码默认价格

#### Scenario: 计算单次请求费用

- **GIVEN** 请求使用模型 `claude-sonnet-4-20250514`
- **AND** input_tokens = 1,000,000, output_tokens = 100,000
- **WHEN** 系统计算费用
- **THEN** 从配置中查找模型定价
- **AND** input_cost = 1,000,000 × `input_cost_per_token` = $3.00
- **AND** output_cost = 100,000 × `output_cost_per_token` = $1.50
- **AND** total_cost = $4.50
- **AND** 费用精确到小数点后 6 位

#### Scenario: 缓存 Token 费用

- **GIVEN** 请求包含缓存 Token
- **AND** cache_creation_tokens = 50,000, cache_read_tokens = 200,000
- **WHEN** 系统计算费用
- **THEN** cache_create_cost = 50,000 × `cache_creation_input_token_cost` = $0.1875
- **AND** cache_read_cost = 200,000 × `cache_read_input_token_cost` = $0.06
- **AND** 如配置中无缓存价格，cache_creation 使用 input_price × 1.25
- **AND** 如配置中无缓存价格，cache_read 使用 input_price × 0.1

#### Scenario: 模型匹配策略

- **GIVEN** 请求使用模型 `anthropic/claude-sonnet-4-20250514`
- **WHEN** 系统查找定价
- **THEN** 首先尝试精确匹配 `anthropic/claude-sonnet-4-20250514`
- **AND** 如未找到，移除 provider 前缀后匹配 `claude-sonnet-4-20250514`
- **AND** 如未找到，模糊匹配包含 `claude-sonnet-4`
- **AND** 如未找到，`has_pricing` 设为 `false`，费用设为最小值 0.0001

#### Scenario: 未知模型处理

- **GIVEN** 请求使用配置中不存在的模型
- **WHEN** 系统计算费用
- **THEN** `has_pricing` 设为 `false`
- **AND** 各项费用设为 0
- **AND** `total_cost` 设为 0.0001 (确保热力图显示)

---

### Requirement: 统计汇总
系统 SHALL 提供多维度的用量统计。

#### Scenario: 热力图统计
- **GIVEN** 用户查看首页
- **WHEN** 加载热力图数据
- **THEN** 按日汇总请求次数
- **AND** 显示最近 N 天的活动热力图

#### Scenario: 总体统计
- **GIVEN** 用户查看统计页面
- **WHEN** 加载统计数据
- **THEN** 显示总请求数、总 Token 数、总费用
- **AND** 支持按时间范围筛选

#### Scenario: 供应商每日统计
- **GIVEN** 用户查看统计页面
- **WHEN** 加载供应商统计
- **THEN** 按供应商和日期汇总
- **AND** 显示成功率、平均 Token 数

---

### Requirement: 日志窗口
系统 SHALL 支持独立的日志查看窗口。

#### Scenario: 打开日志窗口
- **GIVEN** 用户在主窗口
- **WHEN** 用户点击 "查看日志" 按钮
- **THEN** 打开独立的日志窗口
- **AND** 日志窗口显示实时请求日志

#### Scenario: 多窗口独立
- **GIVEN** 日志窗口已打开
- **WHEN** 用户关闭主窗口
- **THEN** 日志窗口保持打开
- **AND** 可独立操作

### Requirement: Logs UI 的 Prism 主题对齐
Logs 仪表盘表面 SHALL 复用 Prism 玻璃 Token，以便切换主题时，背景、排版和图表在视觉上与主页保持一致。

#### Scenario: 亮色模式 Logs 视图
- **GIVEN** 应用程序处于亮色模式
- **WHEN** 用户打开 `/#/logs`
- **THEN** 页面容器必须在共享的 `main-shell` 玻璃表面内渲染（使用与主页相同的 `--app-background` 渐变）
- **AND** 摘要卡片 + 表格 Chrome 必须从共享的语义 CSS 变量（例如 `--mac-surface`, `--surface-card-border`, `--capsule-accent-azure`）获取颜色，而不是硬编码的 Hex 值
- **AND** Chart.js 数据集必须从 CSS 变量读取其描边/填充颜色，以便更改主题时图表自动使用 Prism 蓝色/青色重渲染。

#### Scenario: 暗色模式 Logs 视图
- **GIVEN** 用户切换暗色模式（或系统默认解析为暗色）
- **WHEN** 他们重新访问 `/#/logs`
- **THEN** 背景、摘要卡片、表格条纹和标签芯片必须采用从现有暗色 Token 派生的 "Neon Abyss" 色板（霓虹青/琥珀色）
- **AND** 摘要/图表颜色必须在 `html.dark` 类翻转后 300ms 内更新，无需完全重新加载。

