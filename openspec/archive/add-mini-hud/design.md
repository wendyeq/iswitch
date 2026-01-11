# Design: Mini HUD Architecture

## 1. Window Management (The "Ghost" Window)
Instead of a static window defined in `tauri.conf.json`, the HUD window will be dynamically created/toggled via Rust code (`WebviewWindowBuilder`).

### Attributes
- **Transparent**: `true`
- **Decorations**: `false` (Frameless)
- **AlwaysOnTop**: `true`
- **SkipTaskbar**: `true`
- **Click-Through (macOS)**: Use `NSWindow.setIgnoresMouseEvents_` to toggle between "Interactive Mode" (draggable) and "Passive Mode" (click-through).

## 2. Event Stream Architecture (Real-time Metrics)

The system works on a **Producer-Consumer** model optimized for high-frequency updates.

```mermaid
graph TD
    Proxy[Proxy Service] -->|Chunk Stream| Monitor[Monitor Interceptor]
    
    subgraph "Rust Backend"
        Monitor -->|1. Parse/Estimate| Estimator[Token Estimator]
        Estimator -->|2. High-Freq Event| EventBus[Tauri Event Bus]
    end
    
    EventBus -->|3. payload: {speed, cost, delta}| HUD[Mini HUD Frontend]
    HUD -->|4. RAF Throttled Render| UI[Glassmorphism UI]
```

### Estimation Logic (Hybrid Approach)
1.  **Input**: Calculate exact tokens before request using tokenizer (or char-ratio).
2.  **Streaming**:
    *   **Heuristic**: `len(chunk) / 4` (English) or `0.6` (Chinese) per chunk for immediate visual feedback.
    *   **Calibration**: If `usage` field is detected in SSE stream (e.g. OpenAI `stream_options`), overwrite estimated value.
3.  **End**: Overwrite with final accurate log data.

## 3. Frontend Rendering
- **Framework**: Standard Vue.js component mounted at `/hud`.
- **Performance**: Use CSS transitions for number rolling; avoid heavy Canvas unless visualization complexity grows.
- **State**: `useHUDState` composable to handle event buffering and throttling.

## 4. Trade-offs & Decisions

### 为什么不使用 tiktoken-wasm？
| 方案 | 优点 | 缺点 |
|------|------|------|
| **tiktoken-wasm** | 精确 (误差 < 1%) | +500KB bundle，每 chunk 调用开销大 |
| **字符比例估算** | 零依赖，极快 (< 0.1ms/chunk) | 误差 10-20%（视语言混合程度而定） |

**决策**: 采用字符比例估算。理由：
1. HUD 显示的是**实时趋势**而非精确计费数据
2. 最终统计值由 Provider 返回的 `usage` 覆盖，不影响准确性
3. 性能优先，避免流式传输延迟

### 估算精度分析
| 场景 | 典型误差 | 可接受性 |
|------|----------|----------|
| 纯英文代码 | 5-15% | ✅ 可接受 |
| 中英混合 | 10-25% | ⚠️ 勉强可接受 |
| 纯中文内容 | 15-30% | ⚠️ 显示 "≈" 前缀表示估算 |

**缓解措施**: 在 HUD UI 中对估算值显示 `≈` 前缀，明确告知用户这是实时估算值。

## 5. Platform Considerations

### macOS
- 完整支持 Click-Through (Ghost Mode)
- 使用 `NSWindow.setIgnoresMouseEvents_` API

### Windows
- 不支持原生 Click-Through
- HUD 窗口始终可交互
- 建议用户将 HUD 放置在屏幕角落，减少遮挡

### Linux
- 依赖窗口管理器，行为可能不一致
- 回退到始终可交互模式

## 6. Performance Considerations

### 后端 (Rust)
- `emit_hud_event` 调用必须异步 (spawn)，不阻塞主流
- 事件发射频率上限：100 次/秒 (每 10ms 合并一次)

### 前端 (Vue)
- 使用 `requestAnimationFrame` 节流渲染
- 避免每次事件都触发 DOM 更新；缓冲后批量更新
- CSS 动画优于 JS 动画，利用 GPU 加速

### 内存
- HUD 窗口与主窗口共享同一进程
- 预期增加内存占用：< 20MB

