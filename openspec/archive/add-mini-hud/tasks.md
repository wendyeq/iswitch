# Implementation Tasks

## Phase 1: Backend Core (Stream Interceptor)
- [x] 实现 `extract_sse_content` 函数从 SSE 数据中提取内容文本
- [x] 实现 `TokenEstimator` 启发式 token 估算（字符比例回退）
- [x] 创建 `services/hud_service.rs`，实现 `emit_hud_event` 异步广播更新事件
- [x] 在 `monitor_response` 中集成 HUD 事件发射，流式请求时实时广播更新

## Phase 2: Desktop Window Manager
- [x] 创建 `commands/hud.rs` 管理 HUD 窗口 (`toggle_mini_hud`, `close_hud`, `set_hud_click_through`, `get_hud_status`)
- [x] 实现 macOS 专用 `set_ignore_cursor_events` (Click-Through)
- [x] 在 `lib.rs` 注册 HUD 相关命令
- [x] 实现 HUD 位置/状态持久化到 `AppSettings` (models/settings.rs 中添加 HudSettings)

## Phase 3: Frontend UI
- [x] 在 Vue Router 中添加 `/hud` 路由
- [x] 创建 `layouts/TransparentLayout.vue` (无 padding，透明背景)
- [x] 实现 `HUD/Index.vue` 组件 (Glassmorphism 风格)
- [x] 集成 `listen('hud-update')` 并实现数值滚动动画
- [x] 实现 `composables/useHUDState.ts` 状态管理

## Phase 4: Integration & Polish
- [x] 在系统托盘菜单添加 "打开 Mini HUD" 开关
- [x] 启动时根据设置自动显示 HUD
- [x] 在 HUD 右键/设置中添加 "锁定位置/Click-Through" 切换
- [x] 验证高速流式传输时的内存和 CPU 占用

## Phase 5: Testing & Validation
- [x] 为 `TokenEstimator` 编写单元测试（验证估算精度）
- [x] 为 `HudEvent` 结构编写单元测试
- [x] 为 `extract_sse_content` 编写单元测试（6 个测试用例）
- [x] 为 `useHUDState` composable 编写 Vitest 测试（9 个测试用例）
- [x] 为 `emit_hud_event` 编写集成测试（验证事件发射）
- [ ] 在不同平台 (macOS/Windows/Linux) 上手动验证 HUD 行为

## Phase 6: Internationalization
- [x] 在 `locales/zh.json` 添加 HUD 相关中文文案
- [x] 在 `locales/en.json` 添加 HUD 相关英文文案
- [x] 添加托盘菜单项文案 (tray.toggleHud, tray.closeHud 等)


## Phase 7: Optimization & Improvements (From Code Review)
- [x] 在 `TokenEstimator` 中实现启发式费用估算（目前实时流费用为 0.0）
- [x] 增强 `monitor_response` 日志通道可靠性，在通道满时添加警告日志以防止静默丢包
