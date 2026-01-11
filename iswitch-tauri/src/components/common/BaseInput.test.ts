/**
 * ---
 * [INPUT]: {BaseInput 组件}
 *     - BaseInput: source: ./BaseInput.vue ([POS]: 输入框组件)
 * [OUTPUT]: {测试结果} - BaseInput 组件测试
 * [POS]: 输入框组件单元测试
 * [PROTOCOL]:
 * 1. 测试 v-model 绑定
 * 2. 测试不同类型输入
 * ---
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import BaseInput from './BaseInput.vue';

describe('BaseInput', () => {
  describe('渲染', () => {
    it('正确渲染输入框', () => {
      const wrapper = mount(BaseInput);

      expect(wrapper.find('input').exists()).toBe(true);
    });

    it('应用 base-input class', () => {
      const wrapper = mount(BaseInput);

      expect(wrapper.find('input').classes()).toContain('base-input');
    });
  });

  describe('v-model', () => {
    it('显示传入的 modelValue', () => {
      const wrapper = mount(BaseInput, {
        props: { modelValue: '初始值' },
      });

      expect(wrapper.find('input').element.value).toBe('初始值');
    });

    it('输入时触发 update:modelValue 事件', async () => {
      const wrapper = mount(BaseInput, {
        props: { modelValue: '' },
      });

      const input = wrapper.find('input');
      await input.setValue('新值');

      expect(wrapper.emitted('update:modelValue')).toBeTruthy();
      expect(wrapper.emitted('update:modelValue')![0]).toEqual(['新值']);
    });
  });

  describe('Props', () => {
    it('type="password" 设置正确的类型', () => {
      const wrapper = mount(BaseInput, {
        props: { type: 'password' },
      });

      expect(wrapper.find('input').attributes('type')).toBe('password');
    });

    it('type="email" 设置正确的类型', () => {
      const wrapper = mount(BaseInput, {
        props: { type: 'email' },
      });

      expect(wrapper.find('input').attributes('type')).toBe('email');
    });

    it('placeholder 正确传递', () => {
      const wrapper = mount(BaseInput, {
        props: { placeholder: '请输入...' },
      });

      expect(wrapper.find('input').attributes('placeholder')).toBe('请输入...');
    });

    it('disabled 属性正确传递', () => {
      const wrapper = mount(BaseInput, {
        attrs: { disabled: true },
      });

      expect(wrapper.find('input').attributes('disabled')).toBeDefined();
    });
  });

  describe('交互', () => {
    it('focus 事件正常触发', async () => {
      const wrapper = mount(BaseInput);

      await wrapper.find('input').trigger('focus');

      expect(wrapper.emitted('focus')).toBeTruthy();
    });

    it('blur 事件正常触发', async () => {
      const wrapper = mount(BaseInput);

      await wrapper.find('input').trigger('blur');

      expect(wrapper.emitted('blur')).toBeTruthy();
    });
  });
});
