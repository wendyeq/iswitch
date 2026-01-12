/**
 * ---
 * [INPUT]: {Main Index Component}
 *     - Index: source: ./Index.vue ([POS]: 主页组件)
 * [OUTPUT]: {测试结果} - 主页组件测试
 * [POS]: 业务组件集成测试示例
 * [PROTOCOL]:
 * 1. Mock 路由和服务依赖
 * 2. 验证核心区块渲染与 Tab 行为
 * 3. 覆盖热力图 Tooltip、代理切换与智能 Provider 选择逻辑
 * ---
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import Index from './Index.vue';
import * as claudeSettingsService from '../../services/claudeSettings';
import * as appSettingsService from '../../services/appSettings';
import * as logService from '../../services/logs';
import type { AutomationCard } from '../../data/cards';
import { invoke } from '@tauri-apps/api/core';

vi.mock('../../services/claudeSettings', () => ({
  fetchProxyStatus: vi.fn().mockResolvedValue(false),
  enableProxy: vi.fn().mockResolvedValue(undefined),
  disableProxy: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('../../services/logs', () => ({
  fetchHeatmapStats: vi.fn().mockResolvedValue([]),
  fetchProviderDailyStats: vi.fn().mockResolvedValue([
    {
      provider: 'Alpha',
      total_requests: 12,
      successful_requests: 10,
      failed_requests: 2,
      success_rate: 0.8,
      input_tokens: 600,
      output_tokens: 300,
      reasoning_tokens: 0,
      cache_create_tokens: 0,
      cache_read_tokens: 0,
      cost_total: 1.5,
      hourly_requests: [1, 2, 3],
    },
  ]),
  fetchRequestLogs: vi.fn().mockResolvedValue([
    {
      id: 1,
      provider: 'Alpha',
      platform: 'claude',
      model: 'test',
      http_code: 200,
      input_tokens: 10,
      output_tokens: 5,
      cache_create_tokens: 0,
      cache_read_tokens: 0,
      reasoning_tokens: 0,
      created_at: new Date().toISOString(),
    },
  ]),
}));

vi.mock('../../services/appSettings', () => ({
  fetchAppSettings: vi.fn().mockResolvedValue({ show_heatmap: true }),
}));

vi.mock('../../services/version', () => ({
  fetchCurrentVersion: vi.fn().mockResolvedValue('0.0.1'),
}));

const heatmapWeek = vi.hoisted(() => [
  [
    {
      label: 'Mon',
      intensity: 2,
      dateKey: '2024-01-02T00:00:00.000Z',
      requests: 15,
      inputTokens: 300,
      outputTokens: 200,
      reasoningTokens: 0,
      cost: 0.5,
    },
  ],
]);

vi.mock('../../data/usageHeatmap', () => ({
  DEFAULT_HEATMAP_DAYS: 7,
  generateFallbackUsageHeatmap: () => heatmapWeek,
  buildUsageHeatmapMatrix: vi.fn().mockReturnValue(heatmapWeek),
  calculateHeatmapDayRange: vi.fn().mockReturnValue(7),
}));

const automationCards = vi.hoisted(() => ({
  claude: [
    {
      id: 101,
      name: 'Alpha',
      apiUrl: 'https://alpha.example.com',
      apiKey: '',
      officialSite: '',
      icon: 'alpha',
      tint: '#eee',
      accent: '#111',
      enabled: true,
    },
    {
      id: 102,
      name: 'Beta',
      apiUrl: 'https://beta.example.com',
      apiKey: '',
      officialSite: '',
      icon: 'beta',
      tint: '#eee',
      accent: '#222',
      enabled: false,
    },
  ],
  codex: [
    {
      id: 201,
      name: 'Gamma',
      apiUrl: 'https://gamma.example.com',
      apiKey: '',
      officialSite: '',
      icon: 'gamma',
      tint: '#eee',
      accent: '#333',
      enabled: true,
    },
  ],
}));

vi.mock('../../data/cards', () => ({
  automationCardGroups: automationCards,
  createAutomationCards: (data: AutomationCard[] = []) => data.map(card => ({ ...card })),
}));

vi.mock('../../icons/lobeIconMap', () => ({
  __esModule: true,
  default: { alpha: '<svg />', beta: '<svg />', gamma: '<svg />' },
}));

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}));

const BaseButtonStub = { template: '<button><slot /></button>' };
const BaseModalStub = { template: '<div v-if="open"><slot /></div>', props: ['open'] };

const mountMain = () =>
  mount(Index, {
    global: {
      stubs: {
        BaseButton: BaseButtonStub,
        BaseModal: BaseModalStub,
        'lucide-vue-next': true,
      },
    },
  });

async function createWrapper() {
  const wrapper = mountMain();
  await flushPromises();
  return wrapper;
}

describe('Main/Index.vue', () => {
  let intervalSpy: ReturnType<typeof vi.spyOn>;
  let clearIntervalSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.clearAllMocks();

    Object.defineProperty(window, 'localStorage', {
      value: {
        getItem: vi.fn(),
        setItem: vi.fn(),
        removeItem: vi.fn(),
        clear: vi.fn(),
      },
      configurable: true,
      writable: true,
    });

    Object.defineProperty(window, 'matchMedia', {
      value: vi.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
      configurable: true,
    });

    intervalSpy = vi.spyOn(window, 'setInterval').mockImplementation((() => 0) as unknown as typeof window.setInterval);
    clearIntervalSpy = vi.spyOn(window, 'clearInterval').mockImplementation(() => { });
  });

  afterEach(() => {
    intervalSpy?.mockRestore();
    clearIntervalSpy?.mockRestore();
  });

  it('渲染主要区域', async () => {
    const wrapper = await createWrapper();

    expect(wrapper.find('.main-shell').exists()).toBe(true);
    expect(wrapper.find('.global-actions').exists()).toBe(true);
    expect(wrapper.find('.contrib-page').exists()).toBe(true);
  });

  it('标签页切换', async () => {
    const wrapper = await createWrapper();

    const tabs = wrapper.findAll('.capsule-tab');
    expect(tabs.length).toBeGreaterThan(1);
    expect(tabs[0].classes()).toContain('active');

    await tabs[1].trigger('click');
    await wrapper.vm.$nextTick();

    expect(tabs[1].classes()).toContain('active');
    expect(tabs[0].classes()).not.toContain('active');
  });

  it('代理开关交互调用 enable/disable', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    const enableSpy = vi.mocked(claudeSettingsService.enableProxy);
    const disableSpy = vi.mocked(claudeSettingsService.disableProxy);

    await vm.onProxyToggle();
    expect(enableSpy).toHaveBeenCalledWith('claude');
    expect(vm.proxyStates.claude).toBe(true);

    await vm.onProxyToggle();
    expect(disableSpy).toHaveBeenCalledWith('claude');
    expect(vm.proxyStates.claude).toBe(false);
  });

  it('根据可见区域计算热力图 tooltip', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;

    const fakeEvent = {
      currentTarget: {
        getBoundingClientRect: () => ({
          left: 4,
          width: 12,
          top: 2,
          bottom: 14,
          height: 12,
        }),
      },
    } as unknown as MouseEvent;

    vm.showUsageTooltip(heatmapWeek[0][0], fakeEvent);
    expect(vm.usageTooltip.visible).toBe(true);
    expect(vm.usageTooltip.placement).toBe('below');
    expect(vm.usageTooltip.left).toBeGreaterThan(0);
    expect(String(vm.formattedTooltipAmount)).toMatch(/0\.50/);

    const requestsMetric = vm.usageTooltipMetrics.find((metric: any) => metric.key === 'requests');
    expect(requestsMetric?.value).toBe('15');

    vm.hideUsageTooltip();
    expect(vm.usageTooltip.visible).toBe(false);
  });

  it('smartActiveId 优先使用 dominant provider，回退到第一个启用项', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    const setupState = (wrapper.vm.$ as any).setupState as any;
    if (!Array.isArray(setupState.cards.claude) || setupState.cards.claude.length === 0) {
      setupState.cards.claude = automationCards.claude.map(card => ({
        ...card,
        supportedModels: {},
        modelMapping: {},
      }));
    }
    const providers = setupState.cards.claude as any[];

    expect(vm.smartActiveId).toBe(providers[0].id);

    setupState.dominantProviderMap.claude = providers[1].id;
    await wrapper.vm.$nextTick();
    expect(vm.smartActiveId).toBe(providers[1].id);

    setupState.dominantProviderMap.claude = null;
    providers[0].enabled = false;
    providers[1].enabled = true;
    await wrapper.vm.$nextTick();
    expect(vm.smartActiveId).toBe(providers[1].id);

    providers[1].enabled = false;
    await wrapper.vm.$nextTick();
    expect(vm.smartActiveId).toBe(providers[0].id);
  });

  it('重新排序提供商时会持久化 level 并处理错误', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    vm.cards.claude = automationCards.claude.map(card => ({ ...card }));
    const invokeMock = vi.mocked(invoke);
    invokeMock.mockClear();

    const reordered = [...vm.cards.claude].reverse();
    expect(reordered.length).toBeGreaterThan(0);
    await vm.onCapsuleReorder(reordered);
    await flushPromises();
    await wrapper.vm.$nextTick();
    const activeList = vm.activeCards as AutomationCard[];
    expect(activeList[0].level).toBe(1);
    expect(invokeMock).toHaveBeenCalledWith('save_providers', expect.objectContaining({ kind: 'claude' }));

    invokeMock.mockRejectedValueOnce(new Error('persist failed'));
    await expect(vm.persistProviders('claude')).resolves.toBeUndefined();
  });

  it('handleReceptacleSubmit 支持更新和新增', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    vm.cards.claude = automationCards.claude.map(card => ({ ...card }));
    const existing = vm.cards.claude[0];
    vm.modalState.tabId = 'claude';
    vm.openEditModal(existing);

    await vm.handleReceptacleSubmit({
      name: 'Updated Alpha',
      apiUrl: 'https://alpha.updated',
      apiKey: 'sk-updated',
      icon: 'alpha',
      enabled: false,
      modelMapping: { 'claude-*': 'glm-4' },
    });
    await flushPromises();
    expect(existing.name).toBe('Updated Alpha');
    expect(existing.enabled).toBe(false);
    expect(existing.modelMapping).toEqual({ 'claude-*': 'glm-4' });

    const lengthBefore = (vm.activeCards as AutomationCard[]).length;
    vm.openCreateModal();
    await vm.handleReceptacleSubmit({
      name: 'New Provider',
      apiUrl: 'https://new.example.com',
      apiKey: 'sk-new',
      icon: 'alpha',
      enabled: true,
      modelMapping: {},
    });
    await flushPromises();
    const updatedList = vm.activeCards as AutomationCard[];
    expect(updatedList.length).toBe(lengthBefore + 1);
    expect(updatedList.some((card: AutomationCard) => card.name === 'New Provider')).toBe(true);
  });

  it('删除确认流程会移除卡片', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    const confirmState = (wrapper.vm.$ as any).setupState.confirmState ?? vm.confirmState;
    const target = ((vm.activeCards as AutomationCard[]) ?? [])[0] ?? automationCards.claude[0];
    vm.requestRemove(target);
    await wrapper.vm.$nextTick();
    expect(confirmState.open).toBe(true);
    vm.confirmRemove();
    await flushPromises();
    await wrapper.vm.$nextTick();
    await wrapper.vm.$nextTick();
    expect((vm.activeCards as AutomationCard[]).find((card: AutomationCard) => card.id === target.id)).toBeUndefined();
    expect(confirmState.open).toBe(false);
  });

  it('热力图单元的鼠标事件驱动 tooltip 显示与隐藏', async () => {
    const wrapper = await createWrapper();
    const cell = wrapper.find('.contrib-cell');
    expect(cell.exists()).toBe(true);
    const rect = { left: 10, width: 20, top: 10, bottom: 30, height: 20 };
    Object.defineProperty(cell.element, 'getBoundingClientRect', {
      value: () => rect,
    });

    const enter = new MouseEvent('mouseenter', { bubbles: true });
    cell.element.dispatchEvent(enter);
    await wrapper.vm.$nextTick();
    expect((wrapper.vm as any).usageTooltip.visible).toBe(true);

    const leave = new MouseEvent('mouseleave', { bubbles: true });
    cell.element.dispatchEvent(leave);
    await wrapper.vm.$nextTick();
    expect((wrapper.vm as any).usageTooltip.visible).toBe(false);
  });

  it('formattedTooltipLabel 在无效日期时回退，在有效日期时格式化输出', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    vm.locale.value = 'en';
    vm.usageTooltip.label = 'Fallback';
    vm.usageTooltip.dateKey = 'invalid-date';
    expect(vm.formattedTooltipLabel).toBe('Fallback');

    vm.usageTooltip.label = '';
    vm.usageTooltip.dateKey = '2024-01-02T03:04:05.000Z';
    const formatted = vm.formattedTooltipLabel;
    expect(formatted).not.toBe('');
    expect(formatted).not.toBe('Fallback');
  });

  it('clamp 在 max 小于等于 min 时返回下界', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    expect(vm.clamp(5, 10, 2)).toBe(10);
    expect(vm.clamp(15, 0, 10)).toBe(10);
  });

  it('loadAppSettings 失败时保持热力图开启并响应 handleAppSettingsUpdated', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    const fetchMock = vi.mocked(appSettingsService.fetchAppSettings);
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => { });

    fetchMock.mockRejectedValueOnce(new Error('boom'));
    await vm.loadAppSettings();
    expect(vm.showHeatmap).toBe(true);

    fetchMock.mockResolvedValueOnce({ show_heatmap: false, auto_start: false });
    await vm.handleAppSettingsUpdated();
    expect(vm.showHeatmap).toBe(false);
    errorSpy.mockRestore();
  });

  it('refreshProxyState 失败时会将状态重置为 false', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    const fetchProxyStatusMock = vi.mocked(claudeSettingsService.fetchProxyStatus);
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => { });
    vm.proxyStates.claude = true;

    fetchProxyStatusMock.mockRejectedValueOnce(new Error('oops'));
    await vm.refreshProxyState('claude');
    expect(vm.proxyStates.claude).toBe(false);
    expect(errorSpy).toHaveBeenCalled();
    errorSpy.mockRestore();
  });

  it('胶囊启用切换会更新卡片并持久化', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    vm.cards.claude = automationCards.claude.map(card => ({ ...card }));
    await wrapper.vm.$nextTick();
    const card = (vm.activeCards as AutomationCard[])[0];
    expect(card).toBeTruthy();

    const invokeMock = vi.mocked(invoke);
    const initialCalls = invokeMock.mock.calls.length;
    vm.onCapsuleToggleEnabled(card, false);
    expect(card.enabled).toBe(false);
    await flushPromises();
    expect(invokeMock.mock.calls.length).toBeGreaterThan(initialCalls);
    expect(invokeMock).toHaveBeenCalledWith('save_providers', expect.objectContaining({ kind: vm.activeTab }));
  });

  it('configure 仅代理到 openEditModal，remove 会忽略缺失的列表', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    vm.cards.claude = automationCards.claude.map(card => ({ ...card }));
    await wrapper.vm.$nextTick();
    const card = (vm.activeCards as AutomationCard[])[0];

    vm.configure(card);
    await wrapper.vm.$nextTick();
    expect(vm.modalState.editingId).toBe(card.id);

    const original = vm.cards.codex;
    vm.cards.codex = undefined as any;
    expect(() => vm.remove(999, 'codex')).not.toThrow();
    vm.cards.codex = original;
  });

  it('provider stats 定时器会轮询所有 tab，卸载时清理监听', async () => {
    const wrapper = await createWrapper();
    const vm = wrapper.vm as any;
    const removeSpy = vi.spyOn(window, 'removeEventListener');
    const fetchStatsSpy = vi.mocked(logService.fetchProviderDailyStats);
    fetchStatsSpy.mockClear();
    let captured: (() => void) | null = null;
    const setIntervalSpy = vi.spyOn(window, 'setInterval').mockImplementation(((fn: TimerHandler) => {
      if (typeof fn === 'function') {
        captured = fn as () => void;
      }
      return 123 as unknown as number;
    }) as unknown as typeof window.setInterval);
    const clearIntervalSpy = vi.spyOn(window, 'clearInterval');

    vm.startProviderStatsTimer();
    expect(setIntervalSpy).toHaveBeenCalled();
    (captured as any)?.();
    await flushPromises();
    expect(fetchStatsSpy.mock.calls.length).toBeGreaterThanOrEqual(2);

    wrapper.unmount();
    expect(clearIntervalSpy).toHaveBeenCalledWith(123);
    expect(removeSpy).toHaveBeenCalledWith('app-settings-updated', expect.any(Function));

    setIntervalSpy.mockRestore();
    clearIntervalSpy.mockRestore();
    removeSpy.mockRestore();
  });

  describe('First Run Experience', () => {
    /**
     * [INPUT]: source: openspec/changes/add-onboarding-tooltip/specs/onboarding/spec.md
     * [Testing Requirements]:
     * - T1: Fresh Installation -> Enable Proxy & Show Hint
     * - T4: Persistence -> Do not show again
     * - T3: Interaction -> Dismiss
     */
    it('T1: 首次启动自动开启代理并显示 Ready 提示', async () => {
      const getItemSpy = vi.spyOn(window.localStorage, 'getItem').mockReturnValue(null);
      const setItemSpy = vi.spyOn(window.localStorage, 'setItem');
      const enableProxySpy = vi.mocked(claudeSettingsService.enableProxy);
      enableProxySpy.mockClear();

      const wrapper = await createWrapper();

      // Verify T1 & T2
      // Should invoke enableProxy for each provider
      expect(enableProxySpy).toHaveBeenCalled();
      expect(setItemSpy).toHaveBeenCalledWith('iswitch_ready', 'true');
      expect((wrapper.vm as any).showReadyHint).toBe(true);

      getItemSpy.mockRestore();
    });

    it('T4: 非首次启动不重复触发自动开启逻辑', async () => {
      const getItemSpy = vi.spyOn(window.localStorage, 'getItem').mockReturnValue('true');
      const enableProxySpy = vi.mocked(claudeSettingsService.enableProxy);
      enableProxySpy.mockClear();

      const wrapper = await createWrapper();

      // Logic: Only refreshProxyState is called, enableProxy should NOT be called
      expect(enableProxySpy).not.toHaveBeenCalled();
      expect((wrapper.vm as any).showReadyHint).toBe(false);

      getItemSpy.mockRestore();
    });

    it('T3: 用户交互后提示消失', async () => {
      const getItemSpy = vi.spyOn(window.localStorage, 'getItem').mockReturnValue(null);
      let clickHandler: ((e: Event) => void) | undefined;

      // Spy on addEventListener to capture the dismiss handler
      // Note: We mock matchMedia in beforeEach, but we need to ensure addEventListener works for window
      const addListenerSpy = vi.spyOn(window, 'addEventListener').mockImplementation((event, handler) => {
        if (event === 'click') {
          clickHandler = handler as any;
        }
      });

      vi.useFakeTimers();
      const wrapper = await createWrapper();

      // Initial state
      expect((wrapper.vm as any).showReadyHint).toBe(true);

      // Advance timers to pass the 500ms timeout which binds the listener
      vi.advanceTimersByTime(500);

      // Verify listener was attached
      expect(clickHandler).toBeDefined();

      if (clickHandler) {
        // Act: Trigger global click
        clickHandler(new Event('click'));
        await wrapper.vm.$nextTick();

        // Assert: Hint is gone
        expect((wrapper.vm as any).showReadyHint).toBe(false);
      }

      vi.useRealTimers();
      getItemSpy.mockRestore();
      addListenerSpy.mockRestore();
    });
  });
});
