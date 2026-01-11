/**
 * ---
 * [INPUT]: {ThemeManager}
 *     - ThemeManager: source: ../utils/ThemeManager.ts ([POS]: 主题管理工具)
 * [OUTPUT]: {测试结果} - ThemeManager 单元测试
 * [POS]: 工具函数单元测试示例
 * [PROTOCOL]:
 * 1. 测试主题切换逻辑
 * 2. 测试 DOM class 操作
 * ---
 */
import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import * as ThemeManager from '../utils/ThemeManager';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, value: string) => {
      store[key] = value;
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, 'localStorage', { value: localStorageMock, writable: true });

describe('ThemeManager', () => {
  beforeEach(() => {
    // 清理状态
    localStorageMock.clear();
    document.documentElement.classList.remove('dark', 'light');
    window.location.hash = '#/';
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('getCurrentTheme', () => {
    it('未设置时返回 systemdefault', () => {
      const theme = ThemeManager.getCurrentTheme();
      expect(theme).toBe('systemdefault');
    });

    it('返回 localStorage 中存储的 dark 主题', () => {
      localStorageMock.setItem('theme', 'dark');
      const theme = ThemeManager.getCurrentTheme();
      expect(theme).toBe('dark');
    });

    it('返回 localStorage 中存储的 light 主题', () => {
      localStorageMock.setItem('theme', 'light');
      const theme = ThemeManager.getCurrentTheme();
      expect(theme).toBe('light');
    });

    it('HUD 窗口始终返回 systemdefault', () => {
      window.location.hash = '#/hud';
      localStorageMock.setItem('theme', 'dark');
      const theme = ThemeManager.getCurrentTheme();
      expect(theme).toBe('systemdefault');
    });
  });

  describe('applyTheme', () => {
    it('应用 dark 主题时添加 dark class', () => {
      ThemeManager.applyTheme('dark');
      expect(document.documentElement.classList.contains('dark')).toBe(true);
      expect(document.documentElement.classList.contains('light')).toBe(false);
    });

    it('应用 light 主题时添加 light class', () => {
      ThemeManager.applyTheme('light');
      expect(document.documentElement.classList.contains('light')).toBe(true);
      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('systemdefault 时根据 matchMedia 决定主题', () => {
      // matchMedia 默认返回 matches: false (light mode)
      ThemeManager.applyTheme('systemdefault');
      expect(document.documentElement.classList.contains('light')).toBe(true);
    });

    it('matchMedia 不可用时默认回退到 light', () => {
      const original = window.matchMedia;
      // @ts-expect-error overwrite for test
      delete window.matchMedia;
      ThemeManager.applyTheme('systemdefault');
      expect(document.documentElement.classList.contains('light')).toBe(true);
      if (original) {
        Object.defineProperty(window, 'matchMedia', {
          configurable: true,
          writable: true,
          value: original,
        });
      }
    });
  });

  describe('setTheme', () => {
    it('设置 dark 主题时存储并应用', () => {
      ThemeManager.setTheme('dark');
      expect(localStorageMock.getItem('theme')).toBe('dark');
      expect(document.documentElement.classList.contains('dark')).toBe(true);
    });

    it('设置 light 主题时存储并应用', () => {
      ThemeManager.setTheme('light');
      expect(localStorageMock.getItem('theme')).toBe('light');
      expect(document.documentElement.classList.contains('light')).toBe(true);
    });

    it('设置 systemdefault 主题并存储', () => {
      ThemeManager.setTheme('systemdefault');
      expect(localStorageMock.getItem('theme')).toBe('systemdefault');
    });
  });

  describe('initTheme', () => {
    it('应用存储主题并监听系统切换', () => {
      localStorageMock.setItem('theme', 'dark');
      const addEventListener = vi.fn();
      const mediaQueryMock = {
        matches: false,
        addEventListener,
        removeEventListener: vi.fn(),
        addListener: undefined,
        removeListener: undefined,
      };
      if (!window.matchMedia) {
        Object.defineProperty(window, 'matchMedia', {
          writable: true,
          configurable: true,
          value: vi.fn(),
        });
      }
      const matchMediaSpy = vi.spyOn(window, 'matchMedia').mockReturnValue(mediaQueryMock as any);
      ThemeManager.initTheme();
      expect(document.documentElement.classList.contains('dark')).toBe(true);
      expect(matchMediaSpy).toHaveBeenCalled();
      const handler = addEventListener.mock.calls[0]?.[1] as (() => void) | undefined;
      expect(handler).toBeTypeOf('function');

      // 切换为 systemdefault 后触发事件应重新应用
      localStorageMock.setItem('theme', 'systemdefault');
      handler?.();
      expect(document.documentElement.classList.contains('light')).toBe(true);
      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('HUD 环境始终使用 systemdefault', () => {
      window.location.hash = '#/hud';
      localStorageMock.setItem('theme', 'dark');
      ThemeManager.initTheme();
      expect(document.documentElement.classList.contains('light')).toBe(true);
      expect(document.documentElement.classList.contains('dark')).toBe(false);
    });

    it('在旧浏览器中使用 addListener 注册监听', () => {
      const addListener = vi.fn();
      const mediaQueryMock = {
        matches: false,
        addEventListener: undefined,
        addListener,
        removeListener: vi.fn(),
      };
      if (!window.matchMedia) {
        Object.defineProperty(window, 'matchMedia', {
          configurable: true,
          writable: true,
          value: vi.fn(),
        });
      }
      const matchMediaSpy = vi.spyOn(window, 'matchMedia').mockReturnValue(mediaQueryMock as any);
      ThemeManager.initTheme();
      expect(addListener).toHaveBeenCalledTimes(1);
      matchMediaSpy.mockRestore();
    });
  });
});
