/**
 * ---
 * [INPUT]: {Section Component}
 *     - Section: source: ./Section.vue ([POS]: 玻璃区域组件)
 * [OUTPUT]: {测试结果} - 区域组件渲染测试
 * [POS]: Section 单元测试
 * [PROTOCOL]:
 * 1. 测试默认渲染
 * 2. 测试带标题渲染
 * 3. 测试玻璃效果开关
 * ---
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import SettingsSection from './Section.vue';

describe('Settings/Section.vue', () => {
  it('正确渲染默认状态', () => {
    const wrapper = mount(SettingsSection);

    const section = wrapper.find('.glass-section');
    expect(section.exists()).toBe(true);
  });

  it('渲染带标题的区域', () => {
    const wrapper = mount(SettingsSection, {
      props: {
        title: 'Test Section',
      },
    });

    const title = wrapper.find('.section-title');
    expect(title.exists()).toBe(true);
    expect(title.text()).toBe('Test Section');
  });

  it('渲染空标题时不显示标题元素', () => {
    const wrapper = mount(SettingsSection, {
      props: {
        title: '',
      },
    });

    const title = wrapper.find('.section-title');
    expect(title.exists()).toBe(false);
  });

  it('正确渲染插槽内容', () => {
    const wrapper = mount(SettingsSection, {
      slots: {
        default: '<div class="test-content">Test Content</div>',
      },
    });

    const content = wrapper.find('.test-content');
    expect(content.exists()).toBe(true);
    expect(content.text()).toBe('Test Content');
  });

  it('默认启用玻璃效果', () => {
    const wrapper = mount(SettingsSection);

    const section = wrapper.find('.glass-section');
    expect(section.exists()).toBe(true);
  });

  it('可以禁用玻璃效果', () => {
    const wrapper = mount(SettingsSection, {
      props: {
        glassEffect: false,
      },
    });

    const section = wrapper.find('.glass-section');
    expect(section.exists()).toBe(false);
  });

  it('包含内容容器', () => {
    const wrapper = mount(SettingsSection, {
      slots: {
        default: '<div>Content</div>',
      },
    });

    const content = wrapper.find('.section-content');
    expect(content.exists()).toBe(true);
  });
});
