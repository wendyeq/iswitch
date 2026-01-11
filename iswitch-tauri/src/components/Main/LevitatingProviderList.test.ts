/**
 * ---
 * [INPUT]:
 * - source: iswitch-tauri/src/components/Main/LevitatingProviderList.vue ([POS]: 目标组件)
 * - source: openspec/changes/refactor-provider-capsules/specs/provider-capsules/spec.md ([POS]: 验收标准)
 * [OUTPUT]: {TestSuite} - LevitatingProviderList 组件测试结果
 * [POS]: iswitch-tauri/src/components/Main/LevitatingProviderList.test.ts - 容器组件测试
 * [PROTOCOL]:
 * 1. 测试列表渲染和空状态。
 * 2. 测试展开状态管理。
 * 3. 测试基于鼠标事件的拖拽排序逻辑。
 * 4. 测试事件转发（启用/禁用切换）。
 * ---
 */

import { mount } from '@vue/test-utils';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import LevitatingProviderList from './LevitatingProviderList.vue';
import LevitatingCapsule from './LevitatingCapsule.vue';
import type { AutomationCard } from '../../data/cards';

const mockProviders: AutomationCard[] = [
  {
    id: 1,
    name: 'Provider 1',
    apiUrl: 'https://api1.com',
    apiKey: 'key1',
    officialSite: 'https://site1.com',
    icon: 'openai',
    tint: '#ffffff',
    accent: '#000000',
    enabled: true,
  },
  {
    id: 2,
    name: 'Provider 2',
    apiUrl: 'https://api2.com',
    apiKey: 'key2',
    officialSite: 'https://site2.com',
    icon: 'anthropic',
    tint: '#ffffff',
    accent: '#000000',
    enabled: true,
  },
];

const mockStatsMap = {
  'provider 1': {
    successRate: 0.9,
    requests: 100,
    tokens: 1000,
    cost: 0.1,
  },
};

// Mock i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
    locale: { value: 'en' },
  }),
}));

// Mock LevitatingCapsule to simplify testing and focus on list logic
vi.mock('./LevitatingCapsule.vue', () => ({
  default: {
    name: 'LevitatingCapsule',
    template:
      '<div class="levitating-capsule" :data-id="provider.id" @mousedown="$emit(\'drag-start\', $event)">Capsule {{provider.name}}</div>',
    props: ['provider', 'index', 'isExpanded', 'isActive', 'isDragging', 'isDragOver', 'stats'],
    emits: ['drag-start', 'toggle-expand', 'toggle-enabled', 'configure', 'remove'],
  },
}));

describe('LevitatingProviderList', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Ensure body cleanup
    document.body.innerHTML = '';
  });

  it('renders the correct number of capsules', () => {
    const wrapper = mount(LevitatingProviderList, {
      props: {
        providers: mockProviders,
        statsMap: mockStatsMap,
      },
    });

    const capsules = wrapper.findAll('.levitating-capsule');
    expect(capsules).toHaveLength(2);
    expect(capsules[0].text()).toContain('Provider 1');
    expect(capsules[1].text()).toContain('Provider 2');
  });

  it('shows empty state when no providers', () => {
    const wrapper = mount(LevitatingProviderList, {
      props: {
        providers: [],
      },
    });

    expect(wrapper.find('.empty-state').exists()).toBe(true);
    expect(wrapper.find('.empty-text').text()).toBe('components.main.capsule.emptyState');
  });

  it('toggles expansion state when child emits toggle-expand', async () => {
    const wrapper = mount(LevitatingProviderList, {
      props: {
        providers: mockProviders,
      },
    });

    const firstCapsule = wrapper.findComponent(LevitatingCapsule);

    // Expand first
    await firstCapsule.vm.$emit('toggle-expand', 1);
    expect(firstCapsule.props('isExpanded')).toBe(true);

    // Toggle second (should collapse first and expand second)
    const secondCapsule = wrapper.findAllComponents(LevitatingCapsule)[1];
    await secondCapsule.vm.$emit('toggle-expand', 2);
    expect(firstCapsule.props('isExpanded')).toBe(false);
    expect(secondCapsule.props('isExpanded')).toBe(true);

    // Toggle second again (should collapse)
    await secondCapsule.vm.$emit('toggle-expand', 2);
    expect(secondCapsule.props('isExpanded')).toBe(false);
  });

  it('forwards events from children', async () => {
    const wrapper = mount(LevitatingProviderList, {
      props: {
        providers: mockProviders,
      },
    });

    const firstCapsule = wrapper.findComponent(LevitatingCapsule);

    // toggle-enabled
    // LevitatingCapsule 发出的是 (enabled: boolean)
    await firstCapsule.vm.$emit('toggle-enabled', false);
    expect(wrapper.emitted('toggle-enabled')).toBeTruthy();
    expect(wrapper.emitted('toggle-enabled')![0]).toEqual([mockProviders[0], false]);

    // configure
    await firstCapsule.vm.$emit('configure', mockProviders[0]);
    expect(wrapper.emitted('configure')).toBeTruthy();
    expect(wrapper.emitted('configure')![0]).toEqual([mockProviders[0]]);

    // remove
    await firstCapsule.vm.$emit('remove', mockProviders[0]);
    expect(wrapper.emitted('remove')).toBeTruthy();
    expect(wrapper.emitted('remove')![0]).toEqual([mockProviders[0]]);
  });

  it('manages drag and drop sorting logic', async () => {
    const wrapper = mount(LevitatingProviderList, {
      props: {
        providers: mockProviders,
      },
      attachTo: document.body, // Required for getBoundingClientRect
    });

    const firstCapsule = wrapper.find('.levitating-capsule');

    // 1. Start drag
    await firstCapsule.trigger('mousedown', { clientY: 100 });

    // Verify dragging state
    const capsules = wrapper.findAllComponents(LevitatingCapsule);
    expect(capsules[0].props('isDragging')).toBe(true);
    expect(document.body.style.cursor).toBe('grabbing');

    // 2. Drag over second item
    // Mock getBoundingClientRect for second item
    const secondCapsuleEl = wrapper.findAll('.levitating-capsule')[1].element;
    vi.spyOn(secondCapsuleEl, 'getBoundingClientRect').mockReturnValue({
      top: 200,
      bottom: 260,
      left: 0,
      right: 0,
      width: 100,
      height: 60,
      x: 0,
      y: 200,
      toJSON: () => {},
    });

    // Move mouse to second item's vertical center
    const mousemoveEvent = new MouseEvent('mousemove', { clientY: 230 });
    document.dispatchEvent(mousemoveEvent);
    await wrapper.vm.$nextTick();

    expect(capsules[1].props('isDragOver')).toBe(true);

    // 3. Drop
    const mouseupEvent = new MouseEvent('mouseup');
    document.dispatchEvent(mouseupEvent);
    await wrapper.vm.$nextTick();

    // Verify reorder event
    expect(wrapper.emitted('reorder')).toBeTruthy();
    const reorderPayload = wrapper.emitted('reorder')![0][0] as AutomationCard[];
    expect(reorderPayload[0].id).toBe(2);
    expect(reorderPayload[1].id).toBe(1);

    // Verify state reset
    expect(capsules[0].props('isDragging')).toBe(false);
    expect(capsules[1].props('isDragOver')).toBe(false);
    expect(document.body.style.cursor).toBe('');

    wrapper.unmount();
  });

  it('cleans up event listeners on unmount', () => {
    const removeSpy = vi.spyOn(document, 'removeEventListener');
    const wrapper = mount(LevitatingProviderList, {
      props: { providers: mockProviders },
    });

    wrapper.unmount();

    // Should remove mousemove and mouseup listeners
    expect(removeSpy).toHaveBeenCalledWith('mousemove', expect.any(Function));
    expect(removeSpy).toHaveBeenCalledWith('mouseup', expect.any(Function));
  });

  it('passes stats map data and honors external activeId selection', () => {
    const wrapper = mount(LevitatingProviderList, {
      props: {
        providers: mockProviders,
        statsMap: mockStatsMap,
        activeId: 2,
      },
    });

    const capsules = wrapper.findAllComponents(LevitatingCapsule);
    expect(capsules[0].props('stats')).toEqual(mockStatsMap['provider 1']);
    expect(capsules[0].props('isActive')).toBe(false);
    expect(capsules[1].props('isActive')).toBe(true);
  });
});
