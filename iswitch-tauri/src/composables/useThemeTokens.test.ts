/**
 * ---
 * [INPUT]: {useThemeTokens}
 *     - useThemeTokens: source: ./useThemeTokens.ts ([POS]: 主题 Token Composable)
 * [OUTPUT]: {Composable 测试} - 验证 CSS 变量监听与清理逻辑
 * [POS]: iswitch-tauri/src/composables/useThemeTokens.test.ts
 * [PROTOCOL]:
 * 1. immediate=false 时需手动刷新
 * 2. MutationObserver 触发时通过 requestAnimationFrame 去抖
 * 3. 卸载时清理 observer、media query 以及 RAF
 * ---
 */
import { defineComponent, nextTick } from 'vue';
import { mount } from '@vue/test-utils';
import { beforeAll, beforeEach, describe, expect, it, vi, type Mock } from 'vitest';
import { useThemeTokens, type UseThemeTokensOptions } from './useThemeTokens';

type MutationCallback = (mutations: MutationRecord[], observer: MutationObserver) => void;

const observers: MutationObserverMock[] = [];

class MutationObserverMock {
  public observe = vi.fn();
  public disconnect = vi.fn();
  public takeRecords = vi.fn(() => []);
  private readonly callback: MutationCallback;

  constructor(callback: MutationCallback) {
    this.callback = callback;
    observers.push(this);
  }

  trigger(mutations: Partial<MutationRecord>[] = []) {
    this.callback(mutations as MutationRecord[], this as unknown as MutationObserver);
  }
}

const rafCallbacks = new Map<number, FrameRequestCallback>();
let rafId = 0;
const requestAnimationFrameSpy = vi.fn((cb: FrameRequestCallback) => {
  const id = ++rafId;
  rafCallbacks.set(id, cb);
  return id;
});
const cancelAnimationFrameSpy = vi.fn((id: number) => {
  rafCallbacks.delete(id);
});

const flushAnimationFrame = async () => {
  const callbacks = [...rafCallbacks.values()];
  rafCallbacks.clear();
  callbacks.forEach(cb => cb(performance.now()));
  await Promise.resolve();
};

beforeAll(() => {
  vi.stubGlobal('MutationObserver', MutationObserverMock as unknown as typeof MutationObserver);
  vi.stubGlobal('requestAnimationFrame', requestAnimationFrameSpy);
  vi.stubGlobal('cancelAnimationFrame', cancelAnimationFrameSpy);
});

const mountHarness = (names = ['--hud-primary'], options?: UseThemeTokensOptions) => {
  let api: ReturnType<typeof useThemeTokens> | undefined;
  const Harness = defineComponent({
    name: 'ThemeTokenHarness',
    setup() {
      api = useThemeTokens(names, options);
      return { api };
    },
    template: '<div />',
  });

  const wrapper = mount(Harness);
  if (!api) {
    throw new Error('Composable did not initialize');
  }
  return { wrapper, api };
};

let mediaQueryMock: MediaQueryList;
let matchMediaSpy: Mock;

describe('useThemeTokens', () => {
  beforeEach(() => {
    observers.length = 0;
    rafCallbacks.clear();
    requestAnimationFrameSpy.mockClear();
    cancelAnimationFrameSpy.mockClear();
    document.documentElement.className = '';
    document.documentElement.style.cssText = '';

    mediaQueryMock = {
      matches: false,
      media: '(prefers-color-scheme: dark)',
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    };
    matchMediaSpy = window.matchMedia as unknown as Mock;
    matchMediaSpy.mockClear();
    matchMediaSpy.mockImplementation(() => mediaQueryMock);
  });

  it('defers initial read when immediate=false and refreshTokens manually hydrates values', async () => {
    const target = document.createElement('div');
    target.style.setProperty('--hud-primary', '#123456');
    document.body.appendChild(target);

    const { wrapper, api } = mountHarness(['--hud-primary'], { target, immediate: false });
    await nextTick();
    expect(api.tokens.value).toEqual({});

    api.refreshTokens();
    await nextTick();
    expect(api.tokens.value['--hud-primary']).toBe('#123456');
    expect(api.getTokenValue('--missing', 'fallback')).toBe('fallback');

    document.body.removeChild(target);
    await wrapper.unmount();
  });

  it('reacts to MutationObserver events and batches DOM reads with RAF', async () => {
    document.documentElement.style.setProperty('--hud-primary', '#111111');
    const { wrapper, api } = mountHarness();
    await nextTick();
    expect(api.tokens.value['--hud-primary']).toBe('#111111');

    document.documentElement.style.setProperty('--hud-primary', '#654321');
    const observer = observers.at(-1);
    expect(observer).toBeTruthy();
    observer?.trigger([{ attributeName: 'class' } as MutationRecord]);

    expect(requestAnimationFrameSpy).toHaveBeenCalledTimes(1);
    await flushAnimationFrame();
    expect(api.tokens.value['--hud-primary']).toBe('#654321');

    await wrapper.unmount();
  });

  it('cleans up observers, media listeners, and pending RAF callbacks on unmount', async () => {
    const addListenerSpy = vi.fn();
    const removeListenerSpy = vi.fn();
    (mediaQueryMock as MediaQueryList).addEventListener =
      undefined as unknown as typeof mediaQueryMock.addEventListener;
    (mediaQueryMock as MediaQueryList).removeEventListener =
      undefined as unknown as typeof mediaQueryMock.removeEventListener;
    mediaQueryMock.addListener = addListenerSpy;
    mediaQueryMock.removeListener = removeListenerSpy;

    const { wrapper } = mountHarness();
    await nextTick();
    expect(matchMediaSpy).toHaveBeenCalledTimes(1);
    expect(addListenerSpy).toHaveBeenCalledTimes(1);

    const observer = observers.at(-1);
    observer?.trigger([{ attributeName: 'class' } as MutationRecord]);
    expect(requestAnimationFrameSpy).toHaveBeenCalledTimes(1);

    await wrapper.unmount();
    expect(observer?.disconnect).toHaveBeenCalledTimes(1);
    expect(cancelAnimationFrameSpy).toHaveBeenCalledTimes(1);
    expect(removeListenerSpy).toHaveBeenCalledTimes(1);
  });
});
