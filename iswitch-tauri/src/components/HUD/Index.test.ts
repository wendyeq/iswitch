/**
 * ---
 * [INPUT]: {Mini HUD Component}
 *     - HUD: source: ./Index.vue ([POS]: Mini HUD 浮窗)
 * [OUTPUT]: {HUD 测试用例} - 覆盖 HUD 交互与显示逻辑
 * [POS]: iswitch-tauri/src/components/HUD/Index.test.ts
 * [PROTOCOL]:
 * 1. Mock HUD 状态和主题 Token
 * 2. 验证核心渲染、上下文菜单和键盘事件
 * 3. 覆盖 3 卡片布局和 streaming 光晕效果
 * ---
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils';
import { nextTick, ref } from 'vue';
import HUDIndex from './Index.vue';
import type { HUDState } from '../../composables/useHUDState';

type HudStateRefs = ReturnType<typeof createHudState>;

const hudStateStore = vi.hoisted(() => ({
  current: null as HudStateRefs | null,
}));

function createHudState(): HUDState {
  const initialEvent = {
    provider: 'Test',
    model: 'claude-3',
    delta_tokens: 0,
    total_tokens: 128,
    speed: 42,
    status: 'streaming' as const,
  };

  return {
    isStreaming: ref(true),
    currentEvent: ref(initialEvent),
    displayTokens: ref(2048),
    displaySpeed: ref(42),
    formattedSpeed: ref('42.0'),
    closeHud: vi.fn().mockResolvedValue(undefined),
    setClickThrough: vi.fn().mockResolvedValue(undefined),
  } as unknown as HUDState;
}

const getHudState = () => {
  if (!hudStateStore.current) {
    throw new Error('HUD state not initialized');
  }
  return hudStateStore.current;
};

vi.mock('../../composables/useHUDState', () => ({
  useHUDState: () => {
    const state = createHudState();
    hudStateStore.current = state;
    return state;
  },
}));

vi.mock('../../composables/useThemeTokens', () => {
  const tokensRef = ref({
    '--hud-context-bg': 'rgba(255,255,255,0.9)',
    '--hud-context-border': 'rgba(0,0,0,0.1)',
    '--hud-text-primary': '#000',
  });
  return {
    useThemeTokens: () => ({
      tokens: tokensRef,
    }),
  };
});

const mountedWrappers: VueWrapper[] = [];

async function mountHUD() {
  const mountTarget = document.createElement('div');
  document.body.appendChild(mountTarget);
  const wrapper = mount(HUDIndex, {
    attachTo: mountTarget,
  });
  mountedWrappers.push(wrapper);
  await flushPromises();
  await nextTick();
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
  const appRoot = document.createElement('div');
  appRoot.id = 'app';
  document.body.appendChild(appRoot);
});

afterEach(() => {
  mountedWrappers.splice(0).forEach(wrapper => wrapper.unmount());
  document.documentElement.removeAttribute('style');
  document.body.removeAttribute('style');
  document.body.classList.remove('is-mini-hud');
  vi.clearAllTimers();
  vi.clearAllMocks();
});

describe('HUD/Index.vue', () => {
  it('renders 3 cards: Speed, Tokens, and Model (no Cost or Status cards)', async () => {
    const wrapper = await mountHUD();
    const hudState = getHudState();
    hudState.formattedSpeed.value = '12.3';
    hudState.displayTokens.value = 1337;
    hudState.currentEvent.value = {
      ...hudState.currentEvent.value!,
      model: 'deepseek-chat',
    };
    await nextTick();

    // Should contain the 3 metrics (no labels now)
    expect(wrapper.text()).toContain('12.3');
    expect(wrapper.text()).toContain('tok/s');
    expect(wrapper.text()).toContain('1,337');
    expect(wrapper.text()).toContain('tokens');
    expect(wrapper.text()).toContain('deepseek-chat'); // model name in uppercase style

    // Should NOT contain Cost or old labels
    expect(wrapper.text()).not.toContain('Cost:');
    expect(wrapper.text()).not.toContain('Speed:');
    expect(wrapper.text()).not.toContain('Total Tokens:');
    expect(wrapper.text()).not.toContain('$');

    // Should render hero, secondary, tertiary metrics
    expect(wrapper.find('.metric-hero').exists()).toBe(true);
    expect(wrapper.find('.metric-secondary').exists()).toBe(true);
    expect(wrapper.find('.metric-tertiary').exists()).toBe(true);

    expect(document.body.classList.contains('is-mini-hud')).toBe(true);

    await wrapper.unmount();
    expect(document.body.classList.contains('is-mini-hud')).toBe(false);
  });

  it('shows streaming class on container when streaming', async () => {
    const wrapper = await mountHUD();
    const hudState = getHudState();

    // When streaming
    hudState.isStreaming.value = true;
    await nextTick();

    const container = wrapper.find('.hud-glass-container');
    expect(container.classes()).toContain('streaming');

    // When idle
    hudState.isStreaming.value = false;
    await nextTick();

    expect(container.classes()).not.toContain('streaming');
  });

  it('toggles lock state with pin button', async () => {
    const wrapper = await mountHUD();

    // Find pin button and click to lock
    const pinButton = wrapper.find('.pin-button');
    expect(pinButton.exists()).toBe(true);
    expect(pinButton.classes()).not.toContain('pinned');

    // Click to lock
    await pinButton.trigger('click');
    await nextTick();
    expect(pinButton.classes()).toContain('pinned');

    // Verify container has locked class
    const container = wrapper.find('.hud-glass-container');
    expect(container.classes()).toContain('locked');

    // Click again to unlock
    await pinButton.trigger('click');
    await nextTick();
    expect(pinButton.classes()).not.toContain('pinned');
    expect(container.classes()).not.toContain('locked');
  });

  it('handles Escape key to close HUD', async () => {
    await mountHUD();
    const hudState = getHudState();

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    expect(hudState.closeHud).toHaveBeenCalled();
  });
});
