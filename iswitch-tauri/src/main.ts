/**
 * ---
 * [INPUT]: {App.vue, Router, I18n, ThemeManager}
 *     - App.vue: source: ./App.vue ([POS]: Vue 应用根组件)
 *     - Router: source: ./router/index.ts ([POS]: 路由配置)
 *     - I18n: source: ./utils/i18n.ts ([POS]: 国际化)
 *     - ThemeManager: source: ./utils/ThemeManager.ts ([POS]: 主题管理)
 * [OUTPUT]: {Vue 应用实例} - 初始化并挂载 Vue 应用
 * [POS]: 前端应用入口，负责初始化 Vue 实例、插件和全局配置
 * [PROTOCOL]:
 * 1. 保持最小化，仅做初始化工作
 * 2. 所有业务逻辑在组件中处理
 * ---
 */
import { createApp } from 'vue';
import App from './App.vue';
import './style.css';
import { i18n, setupI18n } from './utils/i18n';
import { initTheme } from './utils/ThemeManager';
import router from './router/index';

initTheme();
const isMac = navigator.userAgent.includes('Mac');
if (isMac) {
  document.documentElement.classList.add('mac');
}

async function bootstrap() {
  await setupI18n('zh'); //默认语言或从后端读取
  createApp(App).use(router).use(i18n).mount('#app');
}
bootstrap();
