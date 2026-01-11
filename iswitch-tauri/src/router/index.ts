/**
 * ---
 * [INPUT]: {Page Components}
 *     - MainPage: source: ../components/Main/Index.vue ([POS]: 主页)
 *     - LogsPage: source: ../components/Logs/Index.vue ([POS]: 日志页)
 *     - SettingsPage: source: ../components/Settings/CrystalControl.vue ([POS]: 设置页 - Crystal Control)
 *     - McpPage: source: ../components/Mcp/index.vue ([POS]: MCP 管理页)
 *     - SkillPage: source: ../components/Skill/Index.vue ([POS]: Skill 管理页)
 *     - HUDPage: source: ../components/HUD/Index.vue ([POS]: Mini HUD 页)
 * [OUTPUT]: {Vue Router 实例} - 配置好的路由对象
 * [POS]: 路由配置中心，定义 URL 到组件的映射关系
 * [PROTOCOL]:
 * 1. 使用 Hash 模式（Tauri 推荐）
 * 2. 路由配置保持扁平化
 * ---
 */
import { createRouter, createWebHashHistory } from 'vue-router';
import MainPage from '../components/Main/Index.vue';
import LogsPage from '../components/Logs/Index.vue';
import SettingsPage from '../components/Settings/CrystalControl.vue';
import McpPage from '../components/Mcp/index.vue';
import SkillPage from '../components/Skill/Index.vue';
import HUDPage from '../components/HUD/Index.vue';

const routes = [
  { path: '/', component: MainPage },
  { path: '/logs', component: LogsPage },
  { path: '/settings', component: SettingsPage },
  { path: '/mcp', component: McpPage },
  { path: '/skill', component: SkillPage },
  { path: '/hud', component: HUDPage },
];

export default createRouter({
  history: createWebHashHistory(), // Use createWebHashHistory for hash-based routing
  routes,
});
