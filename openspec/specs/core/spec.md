# core Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-stack. Update Purpose after archive.
## Requirements
### Requirement: 本地代理中转服务
系统 SHALL 在本地运行一个 HTTP 代理服务器（默认 18099 端口），拦截并转发 Claude Code 和 Codex 的请求。

#### Scenario: 代理服务器启动
- **GIVEN** 应用已启动
- **WHEN** 代理服务初始化完成
- **THEN** 本地 18099 端口进入监听状态
- **AND** 代理状态可通过 API 查询

#### Scenario: Claude Code 请求路由
- **GIVEN** 代理服务器正在运行
- **WHEN** 收到发往 `/v1/messages` 的 POST 请求
- **THEN** 根据当前配置的 Provider 优先级进行转发
- **AND** 若首选 Provider 失败，自动降级到下一个可用 Provider

#### Scenario: Codex 请求路由
- **GIVEN** 代理服务器正在运行
- **WHEN** 收到发往 `/responses`, `/v1/chat/completions`, 或 `/v1/completions` 的 POST 请求
- **THEN** 根据 Codex Provider 配置进行转发
- **AND** 支持流式响应 (SSE) 透传

#### Scenario: CORS 预检请求处理
- **GIVEN** 代理服务器正在运行
- **WHEN** 收到 OPTIONS 预检请求
- **THEN** 返回正确的 CORS 头部
- **AND** 允许所有来源访问

---

### Requirement: 多供应商管理

系统 SHALL 允许用户配置多个模型供应商，并管理相应的 API Key、优先级和模型支持。

#### Scenario: 添加供应商

- **GIVEN** 用户在供应商管理界面
- **WHEN** 用户添加一个新的 Provider (包含 Name, API URL, API Key)
- **THEN** 供应商信息被安全保存到本地配置文件
- **AND** 新供应商可在代理请求中被选中

#### Scenario: 供应商优先级排序

- **GIVEN** 存在多个已配置的 Provider
- **WHEN** 用户拖拽调整顺序
- **THEN** 系统根据新的显示顺序自动更新优先级排序值
- **AND** 优先级顺序被持久化到配置文件
- **AND** 代理请求按新顺序选择 Provider

#### Scenario: 供应商启用/禁用

- **GIVEN** 存在一个已配置的 Provider
- **WHEN** 用户切换其启用状态
- **THEN** 禁用的 Provider 不参与代理路由
- **AND** 状态变更立即生效

---

### Requirement: 模型支持检测
系统 SHALL 根据 Provider 配置的支持模型列表或映射规则，判断是否可以转发特定模型的请求。

#### Scenario: 精确模型匹配
- **GIVEN** Provider 配置了 `supported_models: ["claude-sonnet-4-20250514"]`
- **WHEN** 请求模型为 `claude-sonnet-4-20250514`
- **THEN** 该 Provider 被视为支持此模型

#### Scenario: 通配符模型匹配
- **GIVEN** Provider 配置了 `supported_models: ["claude-*"]`
- **WHEN** 请求模型为 `claude-sonnet-4-20250514`
- **THEN** 该 Provider 被视为支持此模型

#### Scenario: 模型名称映射
- **GIVEN** Provider 配置了 `model_mapping: {"claude-*": "anthropic/claude-*"}`
- **WHEN** 请求模型为 `claude-sonnet-4-20250514`
- **THEN** 转发时请求体中的模型名被替换为 `anthropic/claude-sonnet-4-20250514`

---

### Requirement: 自动故障转移

系统 SHALL 在 Provider 请求失败时自动尝试下一个可用的 Provider，并基于连续失败次数智能降级不健康的供应商。

#### Scenario: 首选 Provider 超时

- **GIVEN** 存在 Provider A (优先级 1) 和 Provider B (优先级 2)
- **WHEN** Provider A 请求超时 (5xx 或连接失败)
- **THEN** 自动重试 Provider B
- **AND** 最终响应返回给客户端

#### Scenario: 所有 Provider 失败

- **GIVEN** 所有配置的 Provider 都不可用
- **WHEN** 代理尝试转发请求
- **THEN** 返回 503 Service Unavailable
- **AND** 错误信息包含失败原因

#### Scenario: Provider 连续失败触发降级

- **GIVEN** Provider A 连续失败次数达到阈值（默认 3 次）
- **WHEN** 收到新的请求
- **THEN** 系统自动跳过 Provider A
- **AND** 直接尝试下一个可用的 Provider
- **AND** 记录日志: "Provider A 状态变更: Healthy -> Degraded"

#### Scenario: 降级 Provider 自动恢复（超时恢复）

> **实现说明**: 恢复超时采用惰性检查机制，即在下次请求到达时检查是否已过超时时间，而非后台定时器自动触发。

- **GIVEN** Provider A 处于降级状态
- **AND** 距离降级已过 recovery_timeout 时间（默认 5 分钟）
- **WHEN** 收到新的请求需要选择 Provider
- **THEN** 系统检测到超时已过，将 Provider A 状态重置为 Healthy
- **AND** Provider A 重新参与本次请求的 Provider 选择
- **AND** 记录日志: "Provider A 状态变更: Degraded -> Healthy, 原因: 恢复超时到期"

#### Scenario: 降级 Provider 自动恢复（请求成功）

- **GIVEN** Provider A 处于降级状态且已过恢复超时
- **AND** 系统将 Provider A 重置为 Healthy 并尝试请求
- **WHEN** 对 Provider A 的请求成功
- **THEN** Provider A 保持 Healthy 状态
- **AND** 重置连续失败计数为 0
- **AND** 记录日志: "Provider A 请求成功，健康状态已确认"

#### Scenario: 所有 Provider 降级时的保底策略

- **GIVEN** 所有配置的 Provider 都处于降级状态
- **WHEN** 收到新的请求
- **THEN** 系统忽略降级状态，强制按优先级尝试所有 Provider
- **AND** 记录警告日志: "所有 Provider 均已降级，启用保底策略"

#### Scenario: 降级状态日志透明

- **GIVEN** 供应商发生状态变更
- **WHEN** 状态从 Healthy 变为 Degraded 或反之
- **THEN** 日志记录包含: 供应商名称、ID、类型、变更原因
- **AND** 用户无需感知或手动干预

### Requirement: 流式响应转发
系统 SHALL 支持 Server-Sent Events (SSE) 流式响应的透明转发，**并在转发过程中实时发射 HUD 更新事件**。

#### Scenario: SSE 流透传
- **GIVEN** 上游 Provider 返回 SSE 流
- **WHEN** 代理接收到流数据
- **THEN** 实时转发给客户端
- **AND** 保持 chunk 边界完整

#### Scenario: SSE 字段兼容性
- **GIVEN** 提供商使用非标准 SSE 字段（如 MiniMax）
- **WHEN** 代理提取内容文本
- **THEN** MUST 支持提取 `thinking`, `message`, `partial_message` 等扩展字段
- **AND** 确保内容不丢失

#### Scenario: SSE 中 Token 统计提取
- **GIVEN** 代理正在转发 SSE 流
- **WHEN** 流中包含 `usage` 字段
- **THEN** 提取 input_tokens, output_tokens 等统计信息
- **AND** 异步写入日志数据库

#### Scenario: 实时 HUD 事件发射
- **GIVEN** 代理正在转发 SSE 流
- **WHEN** 收到上游数据 chunk
- **THEN** 立即计算 token 增量估算
- **AND** 通过 Tauri 事件总线发射 `hud-update` 事件
- **AND** 事件发射不阻塞主响应流（延迟 < 1ms）

### Requirement: 混合 Token 估算
系统 MUST 同时支持精确 usage 提取和启发式估算。

#### Scenario: 精确 Usage 提取
- **GIVEN** Provider 支持 `stream_options` (如 OpenAI)
- **WHEN** 流 chunk 中包含 `usage` 字段
- **THEN** 使用该精确值更新 HUD 显示

#### Scenario: 启发式估算
- **GIVEN** Provider 不返回 usage (如标准 Anthropic 流)
- **WHEN** 收到文本 chunk
- **THEN** 使用字符比例估算 token 数（英文: `chars / 4`, 中文: `chars * 0.6`）
- **AND** 估算值仅用于实时显示，不影响最终日志统计

#### Scenario: 最终校准
- **GIVEN** 流式响应结束
- **WHEN** 收到包含完整 `usage` 的最终消息
- **THEN** 用精确值覆盖之前的估算值
- **AND** HUD 显示更新为最终准确数据

### Requirement: HUD 事件格式
代理 MUST 发射标准化的 HUD 更新事件。

#### Scenario: 事件 Payload 结构
- **GIVEN** HUD 事件被触发
- **WHEN** 事件通过 Tauri 事件总线发送
- **THEN** Payload 结构如下：
```json
{
  "provider": "claude",
  "model": "claude-sonnet-4-20250514",
  "delta_tokens": 12,
  "total_tokens": 156,
  "current_cost": 0.0042,
  "speed": 45.5,
  "status": "streaming"
}
```

#### Scenario: 事件状态类型
- **GIVEN** AI 请求生命周期中
- **WHEN** 不同阶段发射事件
- **THEN** `status` 字段值为以下之一：
  - `"starting"`: 请求开始
  - `"streaming"`: 流式传输中
  - `"completed"`: 请求完成
  - `"error"`: 请求失败

