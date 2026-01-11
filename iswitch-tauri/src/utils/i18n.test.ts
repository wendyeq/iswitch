/**
 * ---
 * [INPUT]: {i18n utils}
 *     - i18n: source: ./i18n.ts ([POS]: 国际化配置)
 * [OUTPUT]: {国际化单元测试} - 验证默认配置与懒加载流程
 * [POS]: iswitch-tauri/src/utils/i18n.test.ts
 * [PROTOCOL]:
 * 1. Mock locale 资源
 * 2. 断言 createI18n 配置
 * 3. 验证 setupI18n 行为
 * ---
 */
import { describe, it, expect, vi } from 'vitest';
import type { Locale } from '../locales';

function localesFactory() {
  const availableLocales = ['zh', 'en'] as const;
  type AvailableLocale = (typeof availableLocales)[number];
  const loadLocaleMessages = vi.fn(async (locale: AvailableLocale) => ({
    locale,
    message: `hello-${locale}`,
  }));
  return {
    availableLocales,
    loadLocaleMessages,
  };
}

vi.mock('../locales', localesFactory);

const importVueI18n = () => import('vue-i18n');
const importLocales = () => import('../locales');
const importI18nModule = () => import('./i18n');

describe('i18n utils', () => {
  it('initializes with macOS defaults and fallback locale', async () => {
    const [, { i18n }] = await Promise.all([importVueI18n(), importI18nModule()]);
    expect(i18n.global.locale.value).toBe('zh');
  });

  it('setupI18n lazily loads locale messages and updates runtime state', async () => {
    const [localesModule, { setupI18n, i18n }] = await Promise.all([importLocales(), importI18nModule()]);
    const loadLocaleMessagesMock = vi.mocked(localesModule.loadLocaleMessages);
    loadLocaleMessagesMock.mockReset();
    const mockMessages = { locale: 'en', message: '你好' } as any;
    loadLocaleMessagesMock.mockResolvedValueOnce(mockMessages);
    const setLocaleMessageSpy = vi.spyOn(i18n.global, 'setLocaleMessage');

    await setupI18n('en' as Locale);

    expect(loadLocaleMessagesMock).toHaveBeenCalledWith('en');
    expect(setLocaleMessageSpy).toHaveBeenCalledWith('en', mockMessages);
    expect(i18n.global.locale.value).toBe('en');
    i18n.global.locale.value = 'zh';
  });
});
