/**
 * ---
 * [INPUT]: {Vitest API, Tauri API}
 * [OUTPUT]: {全局 Mock 配置} - 测试环境初始化
 * [POS]: 测试全局设置，Mock Tauri API 和浏览器 API
 * [PROTOCOL]:
 * 1. Mock 所有 Tauri invoke 调用
 * 2. Mock localStorage 和 matchMedia
 * 3. 提供测试工具函数
 * ---
 */
import { vi, beforeAll, afterEach } from 'vitest';

// ============================================
// Mock Vue I18n
// ============================================
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
    locale: { value: 'zh' },
    d: (val: any) => val,
    n: (val: any) => val,
  }),
  createI18n: vi.fn().mockImplementation(() => {
    const localeRef = { value: 'zh' };
    return {
      global: {
        t: (key: string) => key,
        locale: localeRef,
        setLocaleMessage: vi.fn(),
      },
      install: vi.fn(),
    };
  }),
}));

// ============================================
// Mock Vue Router
// ============================================
const pushMock = vi.fn();
const replaceMock = vi.fn();
const goMock = vi.fn();
const backMock = vi.fn();

const createRouterMock = vi.fn().mockImplementation((options: { routes: any[] }) => ({
  options,
  install: vi.fn(),
  getRoutes: () => options.routes,
  resolve: (path: string) => ({
    matched: options.routes
      .filter(route => route.path === path)
      .map(route => ({ components: { default: route.component } })),
  }),
}));

const createWebHashHistoryMock = vi.fn().mockImplementation(() => ({ mode: 'hash' }));

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: pushMock,
    replace: replaceMock,
    go: goMock,
    back: backMock,
    currentRoute: { value: { path: '/' } },
  }),
  useRoute: () => ({ path: '/', params: {}, query: {} }),
  RouterView: {
    name: 'RouterView',
    render: () => null,
  },
  createRouter: createRouterMock,
  createWebHashHistory: createWebHashHistoryMock,
}));

// 导出 mock 函数供测试使用
export { pushMock, replaceMock, createRouterMock, createWebHashHistoryMock };

// ============================================
// Mock Tauri Core API
// ============================================
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockImplementation((cmd: string, _args?: unknown) => {
    // 根据命令返回默认值
    const mockResponses: Record<string, unknown> = {
      get_version: '0.1.1',
      load_providers: [],
      get_claude_proxy_status: { enabled: false, port: 18099, proxy_url: '' },
      get_codex_proxy_status: { enabled: false, port: 18099, proxy_url: '' },
      get_app_settings: {
        show_heatmap: true,
        auto_start: false,
        proxy_port: 18099,
        failover_threshold: 3,
        recovery_timeout_secs: 300,
      },
      list_request_logs: [],
      get_log_stats: null,
      get_heatmap_stats: [],
      list_mcp_servers: [],
      list_skills: [],
      list_skill_repos: [],
      close_hud: undefined,
      set_hud_click_through: undefined,
      toggle_mini_hud: true,
      get_hud_status: false,
    };
    return Promise.resolve(mockResponses[cmd] ?? null);
  }),
}));

// ============================================
// Mock Tauri Event API
// ============================================
// 事件监听器存储
const eventListeners: Map<string, Array<(event: unknown) => void>> = new Map();

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockImplementation((event: string, handler: (event: unknown) => void) => {
    const handlers = eventListeners.get(event) || [];
    handlers.push(handler);
    eventListeners.set(event, handlers);
    // 返回 unlisten 函数
    return Promise.resolve(() => {
      const handlers = eventListeners.get(event) || [];
      const index = handlers.indexOf(handler);
      if (index > -1) {
        handlers.splice(index, 1);
      }
    });
  }),
  emit: vi.fn().mockImplementation((event: string, payload?: unknown) => {
    const handlers = eventListeners.get(event) || [];
    handlers.forEach(handler => handler({ payload }));
    return Promise.resolve();
  }),
  once: vi.fn().mockResolvedValue(() => {}),
}));

/**
 * 触发模拟的 Tauri 事件 (用于测试)
 */
export function emitMockEvent(event: string, payload: unknown) {
  const handlers = eventListeners.get(event) || [];
  handlers.forEach(handler => handler({ payload }));
}

/**
 * 清除所有事件监听器 (用于测试清理)
 */
export function clearEventListeners() {
  eventListeners.clear();
}

// ============================================
// Mock Tauri App API
// ============================================
vi.mock('@tauri-apps/api/app', () => ({
  getVersion: vi.fn().mockResolvedValue('0.1.1'),
  getName: vi.fn().mockResolvedValue('iSwitch'),
  getTauriVersion: vi.fn().mockResolvedValue('2.0.0'),
}));

// ============================================
// Mock Tauri Dialog Plugin
// ============================================
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
  save: vi.fn().mockResolvedValue(null),
  message: vi.fn().mockResolvedValue(undefined),
  ask: vi.fn().mockResolvedValue(true),
  confirm: vi.fn().mockResolvedValue(true),
}));

// ============================================
// Mock Tauri Shell Plugin
// ============================================
vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn().mockResolvedValue(undefined),
  Command: vi.fn(),
}));

// ============================================
// Mock Tauri OS Plugin
// ============================================
vi.mock('@tauri-apps/plugin-os', () => ({
  platform: vi.fn().mockResolvedValue('macos'),
  arch: vi.fn().mockResolvedValue('aarch64'),
  version: vi.fn().mockResolvedValue('14.0.0'),
  type: vi.fn().mockResolvedValue('Darwin'),
}));

// ============================================
// Mock 浏览器 API
// ============================================
beforeAll(() => {
  // 注意: 不 mock localStorage，jsdom 已提供原生实现
  // 这样测试代码可以直接使用 localStorage

  // Mock matchMedia
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });

  // Mock ResizeObserver
  global.ResizeObserver = vi.fn().mockImplementation(() => ({
    observe: vi.fn(),
    unobserve: vi.fn(),
    disconnect: vi.fn(),
  }));

  // Mock IntersectionObserver
  global.IntersectionObserver = vi.fn().mockImplementation(() => ({
    observe: vi.fn(),
    unobserve: vi.fn(),
    disconnect: vi.fn(),
  }));
});

// 每个测试后清理 mock
afterEach(() => {
  vi.clearAllMocks();
});

// ============================================
// 测试辅助工具
// ============================================

/**
 * 创建自定义的 invoke mock
 * @param overrides - 覆盖默认响应的命令映射
 */
export async function createInvokeMock(overrides: Record<string, unknown> = {}) {
  const { invoke } = vi.mocked(await import('@tauri-apps/api/core'));
  invoke.mockImplementation((cmd: string) => {
    if (cmd in overrides) {
      return Promise.resolve(overrides[cmd]);
    }
    return Promise.resolve(null);
  });
  return invoke;
}

/**
 * 等待 Vue 组件更新
 */
export async function flushPromises() {
  return new Promise(resolve => setTimeout(resolve, 0));
}
