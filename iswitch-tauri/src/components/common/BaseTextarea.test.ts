/**
 * ---
 * [INPUT]: {BaseTextarea Component}
 *     - BaseTextarea: source: ./BaseTextarea.vue ([POS]: 通用文本域)
 * [OUTPUT]: {测试结果} - 文本域双向绑定测试
 * [POS]: iswitch-tauri/src/components/common/BaseTextarea.test.ts
 * [PROTOCOL]:
 * 1. 验证初始值与基础样式
 * 2. 验证输入事件触发 v-model 更新
 * 3. 验证自定义属性透传
 * ---
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import BaseTextarea from './BaseTextarea.vue';

describe('common/BaseTextarea.vue', () => {
  it('renders with default value and base class', () => {
    const wrapper = mount(BaseTextarea);

    const textarea = wrapper.find('textarea');
    expect(textarea.classes()).toContain('base-textarea');
    expect((textarea.element as HTMLTextAreaElement).value).toBe('');
  });

  it('emits update event on user input', async () => {
    const wrapper = mount(BaseTextarea, {
      props: { modelValue: 'hello' },
    });

    const textarea = wrapper.find('textarea');
    await textarea.setValue('updated');

    expect(wrapper.emitted('update:modelValue')).toEqual([['updated']]);
  });

  it('passes through additional attributes', () => {
    const wrapper = mount(BaseTextarea, {
      attrs: { rows: 5, placeholder: 'type here' },
    });

    const textarea = wrapper.find('textarea');
    expect(textarea.attributes('rows')).toBe('5');
    expect(textarea.attributes('placeholder')).toBe('type here');
  });

  it('reflects external modelValue updates', async () => {
    const wrapper = mount(BaseTextarea, { props: { modelValue: 'initial' } });
    await wrapper.setProps({ modelValue: 'synced value' });

    const textarea = wrapper.find('textarea');
    expect((textarea.element as HTMLTextAreaElement).value).toBe('synced value');
  });
});
