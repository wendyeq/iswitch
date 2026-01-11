# Change: Add Mini HUD Mode

## Why
Currently, iSwitch operates as a "silent" background proxy. Users lack real-time visibility into the AI generation process (speed, cost, provider status) without switching away from their IDE/Work context to the main iSwitch window.

## What Changes
Implement a **Mini HUD (Heads-Up Display)** mode: a small, transparent, always-on-top floating window that provides non-intrusive, real-time feedback on AI agent activities.

- **Desktop**: Create a secondary Tauri window (`mini-hud`) with transparent/frameless attributes
- **UX**: Implement "Glassmorphism" UI displaying Tokens/sec, current cost, and provider status
- **Proxy**: Refactor `monitor.rs` to intercept streaming chunks and emit real-time usage estimation events to the frontend
- **Interaction**: Support "Click-Through" mode (macOS) to avoid blocking code editing

**Value Proposition**:
- Enhanced DevEx: Developers get immediate feedback on model performance (speed) and failover events without context switching
- Cost Awareness: Real-time cost estimation helps prevent unexpected billing spikes
- "Geek" Factor: reinforcing iSwitch's identity as a power-user tool

## Impact

### Affected Specs
- `desktop`: 新增动态窗口创建、Click-Through 交互、实时视觉反馈需求
- `settings`: 新增 HUD 设置持久化需求
- `core` (via proxy delta): 扩展流式响应转发，新增实时事件发射能力

### Affected Code
| 文件 | 变更类型 | 说明 |
|------|----------|------|
| `src-tauri/src/commands/hud.rs` | 新增 | HUD 窗口管理命令 |
| `src-tauri/src/proxy/monitor.rs` | 修改 | 添加 ChunkInterceptor 和事件发射 |
| `src-tauri/src/lib.rs` | 修改 | 注册 HUD 命令 |
| `src/views/HUD/Index.vue` | 新增 | HUD 前端组件 |
| `src/layouts/TransparentLayout.vue` | 新增 | 透明布局组件 |
| `src/composables/useHUDState.ts` | 新增 | HUD 状态管理 |
| `src/router/index.ts` | 修改 | 添加 `/hud` 路由 |

