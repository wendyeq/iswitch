/**
 * ---
 * [INPUT]: {Tauri Events}
 *   source: @tauri-apps/api/event ([POS]: 事件监听)
 *   source: ../../../openspec/changes/simplify-mini-hud/specs/desktop/spec.md ([POS]: HUD 事件规范)
 *
 * [OUTPUT]: {useHUDState composable} - HUD 状态管理
 *
 * [POS]: Mini HUD 状态管理 Composable，处理事件监听、数值动画和状态聚合
 *
 * [PROTOCOL]: FractalFlow v1.0
 * ---
 */

import { ref, computed, onMounted, onUnmounted, readonly, type Ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

/**
 * HUD 事件类型（与后端 HudEvent 结构对应）
 */
export interface HudEvent {
  provider: string;
  model: string;
  delta_tokens: number;
  total_tokens: number;
  speed: number;
  status: 'starting' | 'streaming' | 'completed' | 'error';
}

/**
 * HUD 状态接口
 */
export interface HUDState {
  /** 是否正在流式传输 */
  isStreaming: Ref<boolean>;
  /** 当前事件数据 */
  currentEvent: Ref<HudEvent | null>;
  /** 显示的速度值（带动画） */
  displaySpeed: Ref<number>;
  /** 显示的 token 数（带动画） */
  displayTokens: Ref<number>;
  /** 格式化的速度 */
  formattedSpeed: Ref<string>;
  /** 关闭 HUD */
  closeHud: () => Promise<void>;
  /** 设置 Click-Through 模式 */
  setClickThrough: (enabled: boolean) => Promise<void>;
}

/**
 * Mini HUD 状态管理 Composable
 *
 * 功能：
 * - 监听 hud-update 事件
 * - 使用 requestAnimationFrame 节流渲染
 * - 数值滚动动画
 * - 状态颜色和文本管理
 *
 * @returns HUD 状态和控制方法
 */
export function useHUDState(): HUDState {
  // 原始状态
  const currentEvent = ref<HudEvent | null>(null);
  const isStreaming = ref(false);

  // 动画显示值
  const displaySpeed = ref(0);
  const displayTokens = ref(0);

  // 动画目标值
  const targetSpeed = ref(0);
  const targetTokens = ref(0);

  // 动画相关
  let animationFrame: number | null = null;
  let lastUpdateTime = Date.now();
  let unlistenHud: UnlistenFn | null = null;

  // 超时检测
  const IDLE_TIMEOUT_MS = 3000; // 3秒无事件则标记为 Idle
  const RESET_TIMEOUT_MS = 30000; // 30秒无活动则重置为 0
  let idleTimer: number | null = null;
  let resetTimer: number | null = null;

  // 格式化速度
  const formattedSpeed = computed(() => {
    return displaySpeed.value.toFixed(1);
  });

  /**
   * 数值动画循环
   */
  function animateValues() {
    const now = Date.now();
    const elapsed = now - lastUpdateTime;
    const factor = Math.min(elapsed / 100, 1); // 100ms 内完成过渡

    // 平滑过渡到目标值
    displaySpeed.value += (targetSpeed.value - displaySpeed.value) * factor;
    displayTokens.value += (targetTokens.value - displayTokens.value) * factor;

    lastUpdateTime = now;

    if (isStreaming.value) {
      animationFrame = requestAnimationFrame(animateValues);
    } else {
      animationFrame = null;
    }
  }

  /**
   * 重置所有数值为 0
   */
  function resetValues() {
    isStreaming.value = false;
    displaySpeed.value = 0;
    displayTokens.value = 0;
    targetSpeed.value = 0;
    targetTokens.value = 0;
    currentEvent.value = null;
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
      animationFrame = null;
    }
  }

  /**
   * 处理 HUD 更新事件
   */
  function handleHudUpdate(event: HudEvent) {
    // console.log('HUD Update:', event) // Debug

    // 检查数据是否真正变化（用于判断是否重置超时）
    const isDataChanged =
      !currentEvent.value ||
      event.total_tokens !== currentEvent.value.total_tokens ||
      event.status !== currentEvent.value.status;

    currentEvent.value = event;
    targetSpeed.value = event.speed;
    targetTokens.value = event.total_tokens;

    // 只有数据真正变化时才重置超时计时器
    if (isDataChanged) {
      // 清除之前的超时计时器
      if (idleTimer) {
        clearTimeout(idleTimer);
        idleTimer = null;
      }
      if (resetTimer) {
        clearTimeout(resetTimer);
        resetTimer = null;
      }
    }

    if (event.status === 'streaming') {
      isStreaming.value = true;
      if (!animationFrame) {
        lastUpdateTime = Date.now();
        animateValues();
      }

      // 只有数据变化时才设置新的超时
      if (isDataChanged) {
        // 设置 Idle 超时：3秒无新事件则标记为 Idle
        idleTimer = window.setTimeout(() => {
          isStreaming.value = false;
          // 同步显示值到最终值
          displaySpeed.value = targetSpeed.value;
          displayTokens.value = targetTokens.value;
          if (animationFrame) {
            cancelAnimationFrame(animationFrame);
            animationFrame = null;
          }
        }, IDLE_TIMEOUT_MS);

        // 设置重置超时：10秒无活动则重置为 0
        resetTimer = window.setTimeout(() => {
          resetValues();
        }, RESET_TIMEOUT_MS);
      }
    } else if (event.status === 'completed' || event.status === 'error') {
      isStreaming.value = false;
      // 最终更新，直接设置为目标值
      displaySpeed.value = targetSpeed.value;
      displayTokens.value = targetTokens.value;
      if (animationFrame) {
        cancelAnimationFrame(animationFrame);
        animationFrame = null;
      }

      // 设置重置超时：10秒后重置为 0
      if (isDataChanged) {
        resetTimer = window.setTimeout(() => {
          resetValues();
        }, RESET_TIMEOUT_MS);
      }
    }
  }

  /**
   * 关闭 HUD
   */
  async function closeHud() {
    await invoke('close_hud');
  }

  /**
   * 设置 Click-Through 模式
   */
  async function setClickThrough(enabled: boolean) {
    await invoke('set_hud_click_through', { enabled });
  }

  let pollingInterval: number | null = null;
  let lastPolledTokens = 0; // 上次轮询获取的 token 数

  /**
   * 启动轮询 (Fallback for event system issues)
   */
  function startPolling() {
    if (pollingInterval) return;

    pollingInterval = window.setInterval(async () => {
      try {
        const event = await invoke<HudEvent | null>('get_latest_hud_event');
        if (event) {
          // 关键：检查数据是否真正变化
          const tokensChanged = event.total_tokens !== lastPolledTokens;

          if (tokensChanged) {
            // 数据真正变化了，更新
            lastPolledTokens = event.total_tokens;
            handleHudUpdate(event);
          }
          // 如果数据没变化，完全忽略（不调用 handleHudUpdate）
          // 这样超时计时器就不会被重置
        }
      } catch (err) {
        console.error('[HUD] Polling error:', err);
      }
    }, 100);
  }

  // 生命周期
  onMounted(async () => {
    console.log('[HUD] Mounting HUD state, setting up listener...');

    // 1. 设置事件监听 (可能在非 Tauri 环境失败)
    try {
      unlistenHud = await listen<HudEvent>('hud-update', event => {
        console.log('[HUD] Received event payload:', event.payload);
        handleHudUpdate(event.payload);
      });
      console.log('[HUD] Event listener setup successful');
    } catch (err) {
      console.warn('[HUD] Event listener setup failed, will rely on polling:', err);
    }

    // 2. 启动轮询 (双重保障，即使事件监听失败也能工作)
    startPolling();
  });

  onUnmounted(() => {
    if (unlistenHud) {
      unlistenHud();
    }
    if (pollingInterval) {
      clearInterval(pollingInterval);
    }
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
    }
    if (idleTimer) {
      clearTimeout(idleTimer);
    }
    if (resetTimer) {
      clearTimeout(resetTimer);
    }
  });

  return {
    isStreaming: readonly(isStreaming),
    currentEvent: readonly(currentEvent),
    displaySpeed: readonly(displaySpeed),
    displayTokens: readonly(displayTokens),
    formattedSpeed,
    closeHud,
    setClickThrough,
  };
}
