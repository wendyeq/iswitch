/**
 * ---
 * [INPUT]: {useHUDState composable}
 *   source: ./useHUDState.ts ([POS]: HUD 状态管理)
 * [OUTPUT]: {测试结果} - useHUDState 单元测试
 * [POS]: 测试 HUD 状态管理 composable 的核心功能
 * [PROTOCOL]: FractalFlow v1.0
 * ---
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { nextTick } from 'vue';
import { emitMockEvent, clearEventListeners } from '../test/setup';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useHUDState, type HudEvent } from './useHUDState';

// 需要手动模拟 Vue 的 onMounted 和 onUnmounted
const registeredUnmounts: Array<() => void> = [];
vi.mock('vue', async () => {
  const actual = await vi.importActual<typeof import('vue')>('vue');
  return {
    ...actual,
    onMounted: (fn: () => void) => fn(),
    onUnmounted: (fn: () => void) => {
      registeredUnmounts.push(fn);
    },
  };
});

describe('useHUDState', () => {
  beforeEach(() => {
    registeredUnmounts.length = 0;
    // 模拟 requestAnimationFrame
    vi.useFakeTimers();
    vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => {
      return setTimeout(() => cb(Date.now()), 16);
    });
    vi.stubGlobal('cancelAnimationFrame', (id: number) => {
      clearTimeout(id);
    });
    vi.spyOn(console, 'log').mockImplementation(() => {});
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    vi.spyOn(console, 'error').mockImplementation(() => {});
    const invokeMock = vi.mocked(invoke);
    invokeMock.mockReset();
    invokeMock.mockImplementation(async cmd => {
      if (cmd === 'get_latest_hud_event') {
        return null;
      }
      return undefined;
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
    clearEventListeners();
  });

  it('should initialize with default values', async () => {
    const state = useHUDState();

    await nextTick();

    expect(state.isStreaming.value).toBe(false);
    expect(state.currentEvent.value).toBeNull();
    expect(state.displaySpeed.value).toBe(0);
    expect(state.displayTokens.value).toBe(0);
    expect(state.displayTokens.value).toBe(0);
  });

  it('should format speed correctly', async () => {
    const state = useHUDState();

    await nextTick();

    expect(state.formattedSpeed.value).toBe('0.0');
  });

  it('should update state on hud-update event', async () => {
    const state = useHUDState();

    await nextTick();

    const mockEvent: HudEvent = {
      provider: 'claude',
      model: 'claude-sonnet-4',
      delta_tokens: 10,
      total_tokens: 100,
      speed: 25.5,
      status: 'streaming',
    };

    emitMockEvent('hud-update', mockEvent);

    await nextTick();

    expect(state.currentEvent.value).toEqual(mockEvent);
    expect(state.isStreaming.value).toBe(true);
    expect(state.isStreaming.value).toBe(true);
  });

  it('should stop streaming on completed event', async () => {
    const state = useHUDState();

    await nextTick();

    // 先发送 streaming 事件
    emitMockEvent('hud-update', {
      provider: 'claude',
      model: 'claude-sonnet-4',
      delta_tokens: 10,
      total_tokens: 100,
      speed: 25.5,
      status: 'streaming',
    });

    await nextTick();
    expect(state.isStreaming.value).toBe(true);

    // 发送 completed 事件
    emitMockEvent('hud-update', {
      provider: 'claude',
      model: 'claude-sonnet-4',
      delta_tokens: 0,
      total_tokens: 150,
      speed: 30.0,
      status: 'completed',
    });

    await nextTick();

    expect(state.isStreaming.value).toBe(false);
    expect(state.isStreaming.value).toBe(false);
  });

  it('should stop streaming on error event', async () => {
    const state = useHUDState();

    await nextTick();

    emitMockEvent('hud-update', {
      provider: 'claude',
      model: 'claude-sonnet-4',
      delta_tokens: 0,
      speed: 0,
      status: 'error',
    });

    await nextTick();

    expect(state.isStreaming.value).toBe(false);
  });

  it('should provide closeHud and setClickThrough methods', () => {
    const state = useHUDState();

    expect(typeof state.closeHud).toBe('function');
    expect(typeof state.setClickThrough).toBe('function');
  });

  it('should schedule idle/reset timers for streaming events', async () => {
    const state = useHUDState();
    await nextTick();
    emitMockEvent('hud-update', {
      provider: 'claude',
      model: 'claude-sonnet-4',
      delta_tokens: 5,
      total_tokens: 50,
      speed: 10,
      status: 'streaming',
    });
    await nextTick();
    expect(state.isStreaming.value).toBe(true);

    vi.advanceTimersByTime(3100); // idle timeout
    await nextTick();
    expect(state.isStreaming.value).toBe(false);
    expect(state.displayTokens.value).toBe(50);

    vi.advanceTimersByTime(30000); // reset timeout
    await nextTick();
    expect(state.currentEvent.value).toBeNull();
    expect(state.displayTokens.value).toBe(0);
  });

  it('invokes closeHud and setClickThrough handlers', async () => {
    const state = useHUDState();
    const invokeMock = vi.mocked(invoke);
    await state.closeHud();
    await state.setClickThrough(true);
    expect(invokeMock).toHaveBeenCalledWith('close_hud');
    expect(invokeMock).toHaveBeenCalledWith('set_hud_click_through', { enabled: true });
  });

  it('polls latest HUD events only when tokens change and logs errors', async () => {
    const queue: Array<HudEvent | Error | null> = [
      {
        provider: 'claude',
        model: 'sonnet',
        delta_tokens: 2,
        total_tokens: 10,
        speed: 5,
        status: 'streaming',
      },
      new Error('polling failed'),
      {
        provider: 'claude',
        model: 'sonnet',
        delta_tokens: 0,
        total_tokens: 10,
        speed: 5,
        status: 'streaming',
      },
      {
        provider: 'claude',
        model: 'sonnet',
        delta_tokens: 1,
        total_tokens: 20,
        speed: 6,
        status: 'completed',
      },
      null,
    ];
    const invokeMock = vi.mocked(invoke);
    invokeMock.mockImplementation(async cmd => {
      if (cmd === 'get_latest_hud_event') {
        const next = queue.shift() ?? null;
        if (next instanceof Error) {
          throw next;
        }
        return next;
      }
      return undefined;
    });

    const state = useHUDState();
    await nextTick();

    vi.advanceTimersByTime(500); // allow several polling ticks
    await nextTick();
    expect(state.currentEvent.value?.total_tokens).toBe(20);
    expect(console.error).toHaveBeenCalled();
  });

  it('cleans up resources on unmount', async () => {
    const unlistenSpy = vi.fn();
    const listenMock = vi.mocked(listen);
    const originalImpl = listenMock.getMockImplementation();
    listenMock.mockImplementationOnce(async (event, handler) => {
      const unlisten = await originalImpl!(event, handler);
      return () => {
        unlisten();
        unlistenSpy();
      };
    });
    const clearIntervalSpy = vi.spyOn(window, 'clearInterval');
    const clearTimeoutSpy = vi.spyOn(window, 'clearTimeout');
    const cancelSpy = vi.spyOn(window, 'cancelAnimationFrame');

    useHUDState();
    await nextTick();

    emitMockEvent('hud-update', {
      provider: 'claude',
      model: 'sonnet',
      delta_tokens: 5,
      total_tokens: 10,
      speed: 2,
      status: 'streaming',
    });
    await nextTick();

    expect(registeredUnmounts).toHaveLength(1);
    registeredUnmounts.forEach(cb => cb());

    expect(unlistenSpy).toHaveBeenCalled();
    expect(clearIntervalSpy).toHaveBeenCalled();
    expect(cancelSpy).toHaveBeenCalled();
    expect(clearTimeoutSpy).toHaveBeenCalled();
  });
});
