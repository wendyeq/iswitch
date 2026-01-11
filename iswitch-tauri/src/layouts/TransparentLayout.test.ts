/**
 * ---
 * [INPUT]: {TransparentLayout}
 *     - TransparentLayout: source: ./TransparentLayout.vue ([POS]: HUD 布局组件)
 * [OUTPUT]: {布局测试} - 验证 RouterView 包裹和透明背景结构
 * [POS]: iswitch-tauri/src/layouts/TransparentLayout.test.ts
 * [PROTOCOL]:
 * 1. 渲染 RouterView 插槽
 * 2. 保留透明布局容器
 * ---
 */
import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import TransparentLayout from './TransparentLayout.vue';

const RouterViewStub = {
  name: 'RouterView',
  template: '<div class="router-view-stub" />',
};

describe('TransparentLayout.vue', () => {
  it('wraps RouterView with transparent container', () => {
    const wrapper = mount(TransparentLayout, {
      global: {
        stubs: {
          RouterView: RouterViewStub,
        },
      },
    });

    const wrapperEl = wrapper.find('.transparent-layout');
    expect(wrapper.classes()).toContain('transparent-layout');
    expect(wrapperEl.exists()).toBe(true);
    expect(wrapperEl.find('.router-view-stub').exists()).toBe(true);
  });
});
