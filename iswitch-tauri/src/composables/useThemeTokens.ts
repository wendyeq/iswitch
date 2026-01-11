/**
 * ---
 * [INPUT]: {Prism Theme Tokens}
 *     - source: ../style.css ([POS]: 全局 Prism Token 定义)
 * [OUTPUT]: {useThemeTokens} - 返回响应式 CSS 变量映射
 * [POS]: Vue Composable，监听 `html.dark` 与系统主题变化并推送 Token 更新
 * [PROTOCOL]: FractalFlow v1.0
 * ---
 */
import { onMounted, onUnmounted, readonly, shallowRef } from 'vue';

export type ThemeTokenMap = Record<string, string>;

export interface UseThemeTokensOptions {
  /**
   * 自定义读取目标，默认 document.documentElement
   */
  target?: HTMLElement | Document;
  /**
   * 是否立即读取 Token，默认 true
   */
  immediate?: boolean;
}

const isClient = typeof window !== 'undefined';

const getStyleTarget = (target?: HTMLElement | Document) => {
  if (!isClient) return null;
  if (target instanceof HTMLElement) {
    return target;
  }
  if (target && 'documentElement' in target) {
    return target.documentElement;
  }
  return document.documentElement;
};

export function useThemeTokens(names: string[], options: UseThemeTokensOptions = {}) {
  const tokenRef = shallowRef<ThemeTokenMap>({});
  const immediate = options.immediate ?? true;
  let observer: MutationObserver | null = null;
  let mediaQuery: MediaQueryList | null = null;
  let mediaHandler: (() => void) | null = null;
  let rafId: number | null = null;

  const readTokens = () => {
    if (!isClient) return;
    const target = getStyleTarget(options.target);
    if (!target) return;
    const style = getComputedStyle(target);
    const next: ThemeTokenMap = {};
    names.forEach(name => {
      next[name] = style.getPropertyValue(name)?.trim() ?? '';
    });
    tokenRef.value = next;
  };

  const scheduleRead = () => {
    if (!isClient) return;
    if (rafId) {
      cancelAnimationFrame(rafId);
    }
    rafId = requestAnimationFrame(() => {
      rafId = null;
      readTokens();
    });
  };

  const setupObserver = () => {
    if (!isClient || observer) return;
    const target = getStyleTarget(options.target);
    if (!target) return;
    observer = new MutationObserver(mutations => {
      if (mutations.some(mutation => mutation.attributeName === 'class' || mutation.attributeName === 'style')) {
        scheduleRead();
      }
    });
    observer.observe(target, {
      attributes: true,
      attributeFilter: ['class', 'style'],
    });
  };

  const cleanupObserver = () => {
    observer?.disconnect();
    observer = null;
  };

  onMounted(() => {
    if (immediate) {
      readTokens();
    }
    setupObserver();
    if (isClient && !mediaQuery) {
      mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      mediaHandler = () => scheduleRead();
      if (mediaQuery.addEventListener) {
        mediaQuery.addEventListener('change', mediaHandler);
      } else if ((mediaQuery as MediaQueryList & { addListener?: (cb: () => void) => void }).addListener) {
        (mediaQuery as MediaQueryList & { addListener?: (cb: () => void) => void }).addListener(mediaHandler);
      }
    }
  });

  onUnmounted(() => {
    cleanupObserver();
    if (mediaQuery) {
      if (mediaHandler && mediaQuery.removeEventListener) {
        mediaQuery.removeEventListener('change', mediaHandler);
      } else if (
        mediaHandler &&
        (mediaQuery as MediaQueryList & { removeListener?: (cb: () => void) => void }).removeListener
      ) {
        (mediaQuery as MediaQueryList & { removeListener?: (cb: () => void) => void }).removeListener(mediaHandler);
      }
      mediaQuery = null;
      mediaHandler = null;
    }
    if (rafId) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }
  });

  return {
    tokens: readonly(tokenRef),
    getTokenValue: (name: string, fallback = '') => tokenRef.value[name] || fallback,
    refreshTokens: readTokens,
  };
}
