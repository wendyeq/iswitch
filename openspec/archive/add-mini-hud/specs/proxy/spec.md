# Spec: Proxy Monitor 实时事件扩展

> **关联**: 此变更扩展了 `core` spec 中的 `Requirement: 流式响应转发`，新增实时事件发射能力。

## MODIFIED Requirements

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

## ADDED Requirements

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
