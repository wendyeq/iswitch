import { mount } from '@vue/test-utils';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import LevitatingCapsule from './LevitatingCapsule.vue';
import type { AutomationCard } from '../../data/cards';
import { ProviderType } from '../../types/provider';

/**
 * ---
 * [INPUT]:
 * - source: iswitch-tauri/src/components/Main/LevitatingCapsule.vue ([POS]: 目标组件)
 * - source: openspec/changes/refactor-provider-capsules/specs/provider-capsules/spec.md ([POS]: 验收标准)
 * [OUTPUT]: {TestSuite} - LevitatingCapsule 组件测试
 * [POS]: iswitch-tauri/src/components/Main/LevitatingCapsule.test.ts - 单元测试
 * [PROTOCOL]:
 * 1. 测试渲染和样式。
 * 2. 测试展开/收起交互。
 * 3. 测试拖拽和按钮事件。
 * 4. 测试数据展示（Stats）。
 * ---
 */

const mockProvider: AutomationCard = {
  id: 1,
  name: 'Test Provider',
  apiUrl: 'https://api.test.com',
  apiKey: 'sk-test',
  officialSite: 'https://test.com',
  icon: 'openai',
  tint: '#ffffff',
  accent: '#000000',
  enabled: true,
};

const mockStats = {
  successRate: 0.98,
  requests: 1234,
  tokens: 50000,
  cost: 1.25,
  hourlyRequests: [10, 20, 5, 40, 15, 60], // Sparkline data
};

// Mock tauri-apps/plugin-shell
vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn(),
}));

// Mock i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
    locale: { value: 'en' },
  }),
}));

describe('LevitatingCapsule', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders provider information correctly (Collapsed)', () => {
    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: mockProvider,
        index: 0,
        isExpanded: false,
        stats: mockStats,
      },
    });

    // Check name
    expect(wrapper.text()).toContain('Test Provider');

    // Check collapsed summary (Cost and Tokens)
    // Assuming formatCost returns $1.25 and formatNumber returns 50.0K
    // But since we can't easily mock inner computed if they depend on real Intl, we check for presence.
    // Or we can check if the mocked stats are used.
    // We know 'tokens' string is there.
    expect(wrapper.text()).toContain('tokens');

    // Check badge (mini success rate)
    // 98%
    expect(wrapper.text()).toContain('98.0%');

    // Dashboard should not be visible (isExpanded = false)
    expect(wrapper.find('.capsule-dashboard').exists()).toBe(false);
  });

  it('renders dashboard when expanded', () => {
    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: mockProvider,
        index: 0,
        isExpanded: true, // Expanded
        stats: mockStats,
      },
    });

    // Dashboard region should be visible
    const dashboard = wrapper.find('.capsule-dashboard');
    expect(dashboard.exists()).toBe(true);

    // Check metrics visibility
    // Success Rate
    expect(wrapper.text()).toContain('98.0%');
    // Requests (1234 -> 1.2K)
    expect(wrapper.text()).toContain('1.2K');
    // Tokens (50000 -> 50.0K)
    expect(wrapper.text()).toContain('50.0K');
    // Cost
    // $1.25
    expect(wrapper.text()).toContain('$1.25');
  });

  it('emits toggle-expand event on click', async () => {
    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: mockProvider,
        index: 0,
        isExpanded: false,
      },
    });

    // Click the article element
    await wrapper.trigger('click');

    expect(wrapper.emitted()).toHaveProperty('toggle-expand');
    expect(wrapper.emitted()['toggle-expand']).toHaveLength(1);
  });

  it('does NOT emit toggle-expand when clicking actions', async () => {
    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: mockProvider,
        index: 0,
        isExpanded: false,
      },
    });

    // Find the checkbox input (enable toggle)
    const checkbox = wrapper.find('input[type="checkbox"]');
    await checkbox.trigger('click');
    // Note: wrapper.find('.capsule-toggle').trigger('click') might be better if input is hidden

    // The click propagates to label, then input.
    // We need to make sure we click something inside .capsule-actions

    // Let's target the label
    const label = wrapper.find('.capsule-toggle');
    await label.trigger('click');

    // Should emit toggle-enabled (via change) but let's check toggle-expand
    expect(wrapper.emitted()).not.toHaveProperty('toggle-expand');
  });

  it('emits configure event', async () => {
    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: mockProvider,
        index: 0,
        isExpanded: true,
      },
    });

    // Find configure button. It's the 2nd button in .capsule-actions usually.
    // Or find by svg or class.
    // .capsule-action-btn
    const buttons = wrapper.findAll('.capsule-action-btn');
    // 0: Official (exists), 1: Configure, 2: Remove
    const configureBtn = buttons[1];

    await configureBtn.trigger('click');
    expect(wrapper.emitted()).toHaveProperty('configure');
    expect(wrapper.emitted()['configure'][0]).toEqual([mockProvider]);
  });

  it('emits toggle-enabled event with correct value', async () => {
    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: { ...mockProvider, enabled: false },
        index: 0,
        isExpanded: false,
      },
    });

    const checkbox = wrapper.find('input[type="checkbox"]');
    // Set checked and trigger change
    await checkbox.setValue(true);

    expect(wrapper.emitted()).toHaveProperty('toggle-enabled');
    expect(wrapper.emitted()['toggle-enabled'][0]).toEqual([true]);
  });

  it('emits drag-start event', async () => {
    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: mockProvider,
        index: 0,
        isExpanded: false,
      },
    });

    const handle = wrapper.find('.capsule-drag-handle');
    await handle.trigger('mousedown');

    expect(wrapper.emitted()).toHaveProperty('drag-start');
  });

  it('renders URL icon with <img> tag', () => {
    const urlIconProvider: AutomationCard = {
      ...mockProvider,
      icon: 'https://example.com/icon.png',
    };
    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: urlIconProvider,
        index: 0,
        isExpanded: false,
      },
    });

    expect(wrapper.find('img.icon-img').exists()).toBe(true);
    expect(wrapper.find('img.icon-img').attributes('src')).toBe('https://example.com/icon.png');
  });

  it('falls back to initials when icon image fails to load', async () => {
    const urlIconProvider: AutomationCard = {
      ...mockProvider,
      name: 'Broken Icon',
      icon: 'https://example.com/broken.png',
    };
    const wrapper = mount(LevitatingCapsule, {
      props: { provider: urlIconProvider, index: 0, isExpanded: false },
    });

    const img = wrapper.find('img.icon-img');
    expect(img.exists()).toBe(true);

    await img.trigger('error');
    await wrapper.vm.$nextTick();

    expect(wrapper.find('img.icon-img').exists()).toBe(false);
    expect(wrapper.find('.icon-fallback').text()).toBe('BI');
  });

  it('uses apiUrl fallback for dashboard when officialSite is missing', async () => {
    const fallbackProvider: AutomationCard = {
      ...mockProvider,
      officialSite: '',
      apiUrl: 'https://api.fallback.com/v1',
    };

    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: fallbackProvider,
        index: 0,
        isExpanded: false,
      },
    });

    // Button should exist because effectiveDashboardUrl detects 'https://api.fallback.com'
    const buttons = wrapper.findAll('.capsule-action-btn');
    // Expect 3 buttons: Official, Configure, Remove
    expect(buttons.length).toBe(3);
    // First button should be the website one
    expect(buttons[0].attributes('aria-label')).toMatch(/Official Website|components.main.common.website/);
  });

  it('uses known provider preset (Zhipu AI) for dashboard when officialSite is missing', async () => {
    const zhipuProvider: AutomationCard = {
      ...mockProvider,
      name: 'Zhipu AI',
      officialSite: '',
      apiUrl: 'https://open.bigmodel.cn/api/anthropic',
      // No type specified here to test detection fallback, or we can spec it if we want
    };

    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: zhipuProvider,
        index: 0,
        isExpanded: false,
      },
    });

    // This relies on `effectiveDashboardUrl` calling `detectProvider` or type lookup
    // We mocked 'plugin-shell' but `providerDetector` is real?
    // We imported LevitatingCapsule which imports providerDetector real file.
    // It should match Zhipu URL and return the dashboardUrl
    // https://www.bigmodel.cn/usercenter/glm-coding/usage

    // We can check if the button click attempts to open the correct URL
    const websiteBtn = wrapper.findAll('.capsule-action-btn')[0];
    await websiteBtn.trigger('click');

    const shell = await import('@tauri-apps/plugin-shell');
    expect(shell.open).toHaveBeenCalledWith('https://www.bigmodel.cn/usercenter/glm-coding/usage');
  });

  it('prefers provider type default dashboard URLs before auto-detection', async () => {
    const typedProvider: AutomationCard = {
      ...mockProvider,
      officialSite: '',
      type: ProviderType.OPENAI,
    };
    const wrapper = mount(LevitatingCapsule, {
      props: { provider: typedProvider, index: 0, isExpanded: false },
    });

    const vm = wrapper.vm as any;
    expect(vm.effectiveDashboardUrl).toBe('https://platform.openai.com');
  });

  it('returns empty dashboard URL when apiUrl cannot be parsed', async () => {
    const badProvider: AutomationCard = {
      ...mockProvider,
      officialSite: '',
      apiUrl: '%%%not-a-url%%%',
      type: undefined,
    };
    const wrapper = mount(LevitatingCapsule, {
      props: { provider: badProvider, index: 0, isExpanded: false },
    });
    const vm = wrapper.vm as any;
    expect(vm.effectiveDashboardUrl).toBe('');
  });

  it('generates idle sparkline points when no hourly data is available', () => {
    const wrapper = mount(LevitatingCapsule, {
      props: { provider: mockProvider, index: 0, isExpanded: false, stats: { ...mockStats, hourlyRequests: [] } },
    });
    const vm = wrapper.vm as any;
    const points = vm.sparklinePoints;
    expect(points.length).toBeGreaterThan(0);
    expect(points.every((pt: any) => typeof pt.x === 'number' && typeof pt.y === 'number')).toBe(true);
  });

  it('produces empty sparkline paths when sparkline has too few points', () => {
    const wrapper = mount(LevitatingCapsule, {
      props: {
        provider: mockProvider,
        index: 0,
        isExpanded: false,
        stats: { ...mockStats, hourlyRequests: [5] },
      },
    });
    const vm = wrapper.vm as any;
    expect(vm.sparklineLinePath).toBe('');
    expect(vm.sparklineAreaPath).toBe('');
  });

  it('formats large numbers with K/M/B suffixes', () => {
    const wrapper = mount(LevitatingCapsule, {
      props: { provider: mockProvider, index: 0, isExpanded: false },
    });
    const vm = wrapper.vm as any;
    expect(vm.formatNumber(1_500)).toBe('1.5K');
    expect(vm.formatNumber(2_500_000)).toBe('2.5M');
    expect(vm.formatNumber(3_500_000_000)).toBe('3.5B');
  });

  it('fallbacks to window.open when shell.open rejects', async () => {
    const shell = await import('@tauri-apps/plugin-shell');
    const openMock = vi.mocked(shell.open);
    openMock.mockRejectedValueOnce(new Error('shell fail'));
    const windowOpenSpy = vi.spyOn(window, 'open').mockImplementation(() => null);
    const wrapper = mount(LevitatingCapsule, {
      props: { provider: mockProvider, index: 0, isExpanded: true },
    });

    const websiteBtn = wrapper.findAll('.capsule-action-btn')[0];
    await websiteBtn.trigger('click');

    expect(openMock).toHaveBeenCalledWith(mockProvider.officialSite);
    expect(windowOpenSpy).toHaveBeenCalledWith(mockProvider.officialSite, '_blank');
    windowOpenSpy.mockRestore();
  });

  it('ignores toggle-expand when clicking within capsule actions', async () => {
    const wrapper = mount(LevitatingCapsule, {
      props: { provider: mockProvider, index: 0, isExpanded: false },
    });
    const actions = wrapper.find('.capsule-actions');
    await actions.trigger('click');
    expect(wrapper.emitted()['toggle-expand']).toBeUndefined();
  });
});
