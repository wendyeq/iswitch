/**
 * ---
 * [INPUT]: {BaseButton 组件}
 *     - BaseButton: source: ./BaseButton.vue ([POS]: 基础按钮组件)
 * [OUTPUT]: {测试结果} - BaseButton 组件测试
 * [POS]: Vue 组件单元测试示例
 * [PROTOCOL]:
 * 1. 测试组件渲染
 * 2. 测试 props 变体
 * 3. 测试点击事件
 * ---
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import BaseButton from './BaseButton.vue';

describe('BaseButton', () => {
  describe('渲染', () => {
    it('正确渲染默认按钮', () => {
      const wrapper = mount(BaseButton, {
        slots: {
          default: '点击我',
        },
      });

      expect(wrapper.text()).toBe('点击我');
      expect(wrapper.classes()).toContain('btn');
      expect(wrapper.classes()).toContain('btn-primary');
    });

    it('渲染为 button 元素', () => {
      const wrapper = mount(BaseButton);
      expect(wrapper.element.tagName).toBe('BUTTON');
    });

    it('默认 type 为 button', () => {
      const wrapper = mount(BaseButton);
      expect(wrapper.attributes('type')).toBe('button');
    });
  });

  describe('Props', () => {
    it('variant="outline" 应用正确的 class', () => {
      const wrapper = mount(BaseButton, {
        props: { variant: 'outline' },
      });

      expect(wrapper.classes()).toContain('btn-outline');
      expect(wrapper.classes()).not.toContain('btn-primary');
    });

    it('variant="danger" 应用正确的 class', () => {
      const wrapper = mount(BaseButton, {
        props: { variant: 'danger' },
      });

      expect(wrapper.classes()).toContain('btn-danger');
    });

    it('variant="ghost" 应用正确的 class', () => {
      const wrapper = mount(BaseButton, {
        props: { variant: 'ghost' },
      });

      expect(wrapper.classes()).toContain('btn-ghost');
    });

    it('type="submit" 设置正确的 type 属性', () => {
      const wrapper = mount(BaseButton, {
        props: { type: 'submit' },
      });

      expect(wrapper.attributes('type')).toBe('submit');
    });
  });

  describe('交互', () => {
    it('点击时触发 click 事件', async () => {
      const wrapper = mount(BaseButton);

      await wrapper.trigger('click');

      expect(wrapper.emitted('click')).toBeTruthy();
      expect(wrapper.emitted('click')?.length).toBe(1);
    });

    it('disabled 状态时按钮不可点击', async () => {
      const wrapper = mount(BaseButton, {
        attrs: { disabled: true },
      });

      expect(wrapper.attributes('disabled')).toBeDefined();
    });
  });

  describe('插槽', () => {
    it('渲染默认插槽内容', () => {
      const wrapper = mount(BaseButton, {
        slots: {
          default: '<span class="icon">✓</span> 确认',
        },
      });

      expect(wrapper.find('.icon').exists()).toBe(true);
      expect(wrapper.text()).toContain('确认');
    });
  });
});
