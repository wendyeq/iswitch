/**
 * ---
 * [INPUT]: {router/index.ts}
 *     - router: source: ./index.ts ([POS]: 路由配置)
 * [OUTPUT]: {路由表测试} - 确认路径与组件映射
 * [POS]: iswitch-tauri/src/router/index.test.ts
 * [PROTOCOL]:
 * 1. 校验所有已注册的路径
 * 2. 确保各路径解析到正确组件
 * ---
 */
import { describe, expect, it } from 'vitest';
import router from './index';
import MainPage from '../components/Main/Index.vue';
import LogsPage from '../components/Logs/Index.vue';
import SettingsPage from '../components/Settings/CrystalControl.vue';
import McpPage from '../components/Mcp/index.vue';
import SkillPage from '../components/Skill/Index.vue';
import HUDPage from '../components/HUD/Index.vue';

describe('router configuration', () => {
  it('contains expected static routes', () => {
    const resolvedPaths = router
      .getRoutes()
      .map(record => record.path)
      .sort();

    expect(resolvedPaths).toEqual(['/', '/hud', '/logs', '/mcp', '/settings', '/skill'].sort());
  });

  it('resolves each route to the correct component', () => {
    const cases: Record<string, unknown> = {
      '/': MainPage,
      '/logs': LogsPage,
      '/settings': SettingsPage,
      '/mcp': McpPage,
      '/skill': SkillPage,
      '/hud': HUDPage,
    };

    Object.entries(cases).forEach(([path, component]) => {
      const match = router.resolve(path).matched[0];
      expect(match?.components?.default).toBe(component);
    });
  });
});
