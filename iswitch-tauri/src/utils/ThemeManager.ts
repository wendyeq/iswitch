/**
 * ---
 * [INPUT]: {ThemeMode}
 * [OUTPUT]: {Theme 应用函数} - 应用和切换主题
 * [POS]: 主题管理器，处理深色/浅色/系统默认主题切换
 * [PROTOCOL]:
 * 1. 主题配置持久化到 localStorage
 * 2. 支持跟随系统主题自动切换
 * 3. 使用 CSS class 控制主题 (dark/light)
 * ---
 */
// src/utils/ThemeManager.ts
const THEME_KEY = 'theme';
const DARK_MEDIA_QUERY = '(prefers-color-scheme: dark)';
const isClient = typeof window !== 'undefined';

const isHudContext = () => {
  if (!isClient) return false;
  return window.location.hash.startsWith('#/hud');
};

const readSystemTheme = (): Exclude<ThemeMode, 'systemdefault'> => {
  if (!isClient || typeof window.matchMedia !== 'function') {
    return 'light';
  }
  return window.matchMedia(DARK_MEDIA_QUERY).matches ? 'dark' : 'light';
};

const syncNativeTheme = (theme: 'light' | 'dark') => {
  if (!isClient) return;
  import('@tauri-apps/api/core').then(({ invoke }) => {
    invoke('sync_window_theme', { theme }).catch(() => {
      // 忽略在非 Tauri 环境下的报错
    });
  });
};

export type ThemeMode = 'light' | 'dark' | 'systemdefault';

export function applyTheme(mode: ThemeMode) {
  if (typeof document === 'undefined') return;
  const resolvedTheme = mode === 'systemdefault' ? readSystemTheme() : mode;
  document.documentElement.classList.remove('dark', 'light');
  document.documentElement.classList.add(resolvedTheme);
  syncNativeTheme(resolvedTheme);
}

export function initTheme() {
  if (!isClient) return;
  const hudOnly = isHudContext();
  const savedTheme = hudOnly ? 'systemdefault' : getCurrentTheme();
  applyTheme(savedTheme);

  const media = typeof window.matchMedia === 'function' ? window.matchMedia(DARK_MEDIA_QUERY) : null;
  const handleSystemChange = () => {
    const current = hudOnly ? 'systemdefault' : getCurrentTheme();
    if (hudOnly || current === 'systemdefault') {
      applyTheme('systemdefault');
    }
  };
  if (media?.addEventListener) {
    media.addEventListener('change', handleSystemChange);
  } else if (media?.addListener) {
    media.addListener(handleSystemChange);
  }
}

export function setTheme(mode: ThemeMode) {
  if (isClient) {
    localStorage.setItem(THEME_KEY, mode);
  }
  applyTheme(mode);
}

export function getCurrentTheme(): ThemeMode {
  if (!isClient) return 'systemdefault';
  if (isHudContext()) return 'systemdefault';
  return (localStorage.getItem(THEME_KEY) || 'systemdefault') as ThemeMode;
}
