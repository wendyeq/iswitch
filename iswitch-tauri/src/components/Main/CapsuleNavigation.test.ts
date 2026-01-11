/**
 * ---
 * [INPUT]:
 *     - source: ./CapsuleNavigation.vue ([POS]: 胶囊导航组件)
 *     - plan: openspec/changes/simplify-ui-controls/test-plan.md ([POS]: 简化 UI 测试计划)
 * [OUTPUT]: {TestSuite} - 胶囊导航组件单元测试 (简化版)
 * [POS]: 胶囊导航组件单元测试
 * [PROTOCOL]:
 * 1. Mock 路由和 i18n
 * 2. 验证 5 个导航项目渲染 (Proxy, Add, Logs, Theme, Settings)
 * 3. 验证点击事件触发正确路由/方法
 * 4. 验证主题切换逻辑
 * 5. 验证工具提示和可访问性
 * [NOTE]: Skill 和 MCP 按钮已移除 (simplify-ui-controls)
 * ---
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import CapsuleNavigation from './CapsuleNavigation.vue';

// Mock vue-router
const mockPush = vi.fn();
vi.mock('vue-router', async () => {
  const actual = await vi.importActual('vue-router');

  return {
    ...actual,
    useRouter: () => ({
      push: mockPush,
    }),
  };
});

// Mock vue-i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
    locale: { value: 'zh' },
  }),
}));

// Mock ThemeManager
const mockSetTheme = vi.fn();
// Default to dark for initial state
let mockCurrentTheme = 'dark';
vi.mock('../../utils/ThemeManager', () => ({
  getCurrentTheme: vi.fn(() => mockCurrentTheme),
  setTheme: (theme: string) => mockSetTheme(theme),
}));

// [simplify-ui-controls] lobeIconMap mock 已移除，因为 MCP 图标不再使用

describe('CapsuleNavigation.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockCurrentTheme = 'dark'; // Reset to dark by default

    // Mock window.matchMedia
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: vi.fn().mockImplementation((query: string) => ({
        matches: query === '(prefers-color-scheme: dark)',
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });
  });

  // TC-NAV-001: 验证胶囊外观
  it('TC-NAV-001: 渲染胶囊导航容器并具有正确的类', () => {
    const wrapper = mount(CapsuleNavigation);
    expect(wrapper.find('.capsule-navigation').exists()).toBe(true);
  });

  // TC-NAV-002: 验证图标顺序 (简化版: 5 个按钮)
  it('TC-NAV-002: 渲染所有导航项目 (简化版)', () => {
    const wrapper = mount(CapsuleNavigation);
    const items = wrapper.findAll('.capsule-item');
    // 简化后结构: Proxy(0), Add(1), Logs(2), Theme(3), Settings(4)
    // [simplify-ui-controls] Skill 和 MCP 已移除
    expect(items).toHaveLength(5);

    // Index 0: Proxy
    expect(items[0].attributes('data-tooltip')).toContain('components.main.relayToggle.tooltip');
    // Index 1: Add
    expect(items[1].attributes('data-tooltip')).toBe('components.main.tabs.addCard');
    // Index 2: Logs
    expect(items[2].attributes('data-tooltip')).toBe('components.main.logs.view');
    // Index 3: Theme (原 Index 5)
    expect(items[3].attributes('data-tooltip')).toBe('components.main.controls.theme');
    // Index 4: Settings (原 Index 6)
    expect(items[4].attributes('data-tooltip')).toBe('components.main.controls.settings');
  });

  // TC-NAV-NEW: 验证 Context Controls
  it('TC-NAV-NEW: 验证 Context Controls (Tabs, Proxy, Add)', async () => {
    const wrapper = mount(CapsuleNavigation, {
      props: {
        viewMode: 'claude',
        proxyEnabled: false,
        proxyLoading: false,
      },
    });

    // Tabs
    const tabs = wrapper.findAll('.capsule-tab');
    expect(tabs).toHaveLength(2);
    expect(tabs[0].text()).toContain('components.main.tabs.claude');
    expect(tabs[1].text()).toContain('components.main.tabs.codex');

    // Click Codex Tab
    await tabs[1].trigger('click');
    expect(wrapper.emitted('update:viewMode')?.[0]).toEqual(['codex']);

    // Proxy Toggle
    // Using partial match for tooltip because it contains (On)/(Off)
    const proxyBtn = wrapper.find('[data-tooltip*="components.main.relayToggle.tooltip"]');
    await proxyBtn.trigger('click');
    expect(wrapper.emitted('toggleProxy')).toBeTruthy();

    // Add Button
    const addBtn = wrapper.find('[data-tooltip="components.main.tabs.addCard"]');
    await addBtn.trigger('click');
    expect(wrapper.emitted('add')).toBeTruthy();
  });

  it('proxy button shows busy state and disables click when loading', async () => {
    const wrapper = mount(CapsuleNavigation, {
      props: {
        proxyEnabled: true,
        proxyLoading: true,
      },
    });

    const proxyBtn = wrapper.find('[data-tooltip*="components.main.relayToggle.tooltip"]');
    expect(proxyBtn.classes()).toEqual(expect.arrayContaining(['is-active', 'is-busy']));
    expect(proxyBtn.attributes('disabled')).toBeDefined();
    expect(proxyBtn.find('svg.animate-spin').exists()).toBe(true);

    await proxyBtn.trigger('click');
    expect(wrapper.emitted('toggleProxy')).toBeFalsy();
  });

  // TC-NAV-003, TC-NAV-003b: 验证主题切换
  describe('主题切换 (TC-NAV-003, TC-NAV-003b)', () => {
    it('TC-NAV-003: 初始为暗色，点击切换到亮色', async () => {
      mockCurrentTheme = 'dark';
      const wrapper = mount(CapsuleNavigation);

      const themeBtn = wrapper.find('[data-tooltip="components.main.controls.theme"]');
      await themeBtn.trigger('click');

      // 期望调用 setTheme('light')
      expect(mockSetTheme).toHaveBeenCalledWith('light');
    });

    it('TC-NAV-003b: 初始为亮色，点击切换到暗色', async () => {
      // 这里我们需要重新 mock getCurrentTheme 的返回值并重新挂载
      const themeManager = await import('../../utils/ThemeManager');
      // Force return light
      vi.mocked(themeManager.getCurrentTheme).mockReturnValue('light');

      const wrapper = mount(CapsuleNavigation);

      const themeBtn = wrapper.find('[data-tooltip="components.main.controls.theme"]');
      // 亮色模式应该显示太阳图标 (v-if="themeIcon === 'sun'")
      expect(wrapper.find('svg circle').exists()).toBe(true);

      await themeBtn.trigger('click');

      // 期望调用 setTheme('dark')
      expect(mockSetTheme).toHaveBeenCalledWith('dark');
    });
  });

  // TC-NAV-004 ~ TC-NAV-007: 导航功能 (简化版)
  describe('导航功能 (TC-NAV-004, TC-NAV-007)', () => {
    it('TC-NAV-007: 点击日志按钮导航到 /logs', async () => {
      const wrapper = mount(CapsuleNavigation);
      const btn = wrapper.find('[data-tooltip="components.main.logs.view"]');
      await btn.trigger('click');
      expect(mockPush).toHaveBeenCalledWith('/logs');
    });

    // [simplify-ui-controls] TC-NAV-005 (MCP) 和 TC-NAV-006 (Skill) 已移除

    it('TC-NAV-004: 点击设置按钮导航到 /settings', async () => {
      const wrapper = mount(CapsuleNavigation);
      const btn = wrapper.find('[data-tooltip="components.main.controls.settings"]');
      await btn.trigger('click');
      expect(mockPush).toHaveBeenCalledWith('/settings');
    });
  });

  // TC-NAV-010: 验证工具提示 (简化版)
  it('TC-NAV-010: 所有图标显示正确的工具提示 (简化版)', () => {
    const wrapper = mount(CapsuleNavigation, {
      props: { proxyEnabled: false },
    });
    const items = wrapper.findAll('.capsule-item');

    // 简化后: 5 个按钮
    expect(items).toHaveLength(5);
    expect(items[0].attributes('data-tooltip')).toContain('components.main.relayToggle.tooltip');
    expect(items[1].attributes('data-tooltip')).toBe('components.main.tabs.addCard');
    expect(items[2].attributes('data-tooltip')).toBe('components.main.logs.view');
    // [simplify-ui-controls] Skill 和 MCP 已移除
    expect(items[3].attributes('data-tooltip')).toBe('components.main.controls.theme');
    expect(items[4].attributes('data-tooltip')).toBe('components.main.controls.settings');
  });

  // TC-NAV-011: 可访问性
  describe('可访问性 (TC-NAV-011)', () => {
    it('导航栏具有 role="navigation" 和 aria-label', () => {
      const wrapper = mount(CapsuleNavigation);
      const nav = wrapper.find('.capsule-navigation');
      expect(nav.attributes('role')).toBe('navigation');
      expect(nav.attributes('aria-label')).toBe('components.main.capsule.ariaLabel');
    });

    it('所有按钮都是可聚焦的 (默认 button 行为)', () => {
      const wrapper = mount(CapsuleNavigation);
      const buttons = wrapper.findAll('button');
      buttons.forEach(btn => {
        expect(btn.element.tagName).toBe('BUTTON');
      });
    });
  });
});
