/**
 * ---
 * [INPUT]: {Locale loader}
 *     - locales: source: ./index.ts ([POS]: 国际化语言入口)
 * [OUTPUT]: {Locale 测试} - 验证可用语言与消息加载
 * [POS]: iswitch-tauri/src/locales/index.test.ts
 * [PROTOCOL]:
 * 1. 校验 availableLocales 列表
 * 2. 验证 loadLocaleMessages 返回 JSON 内容
 * 3. 对未知 locale 返回 undefined
 * ---
 */
import { describe, it, expect } from 'vitest';
import { availableLocales, loadLocaleMessages, type Locale } from './index';

describe('locales module', () => {
  it('exposes supported locale list in deterministic order', () => {
    expect(availableLocales).toEqual(['en', 'zh']);
  });

  it('loads locale JSON payloads for known locales', async () => {
    const enMessages = await loadLocaleMessages('en');
    const zhMessages = await loadLocaleMessages('zh');

    expect(enMessages?.components?.main?.hero?.eyebrow).toBeDefined();
    expect(zhMessages?.components?.main?.hero?.eyebrow).toBeDefined();
    expect(enMessages).not.toBe(zhMessages);
  });

  it('returns undefined when an unknown locale is coerced into the API', async () => {
    const bogusLocale = 'fr' as Locale;
    const payload = await loadLocaleMessages(bogusLocale);
    expect(payload).toBeUndefined();
  });
});
