/**
 * ---
 * [INPUT]: {vue-i18n, Locale Messages}
 *     - vue-i18n: source: vue-i18n ([POS]: 国际化框架)
 *     - Messages: source: ../locales/ ([POS]: 语言包目录)
 * [OUTPUT]: {i18n 实例, setupI18n 函数} - 配置好的国际化对象和初始化函数
 * [POS]: 国际化配置中心，提供多语言支持
 * [PROTOCOL]:
 * 1. 使用 Composition API 模式
 * 2. 支持懒加载语言包
 * 3. 默认语言: 中文
 * ---
 */
// src/i18n.ts
import { createI18n } from 'vue-i18n';
import { Locale, loadLocaleMessages } from '../locales';

const defaultLocale: Locale = 'zh';
export const i18n = createI18n({
  legacy: false, // 使用 Composition API
  locale: defaultLocale, // 默认语言
  fallbackLocale: 'en',
  messages: {},
});

//export default i18n
// 初始化语言（只加载一次）
export async function setupI18n(locale: Locale) {
  const messages = await loadLocaleMessages(locale);
  i18n.global.setLocaleMessage(locale, messages);
  i18n.global.locale.value = locale;
}
