/**
 * ---
 * [INPUT]: {PhysicSwitch Component}
 *     - PhysicSwitch: source: ./PhysicSwitch.vue ([POS]: 物理开关组件)
 * [OUTPUT]: {测试结果} - 物理开关交互测试
 * [POS]: PhysicSwitch 单元测试
 * [PROTOCOL]:
 * 1. 测试开关切换逻辑
 * 2. 测试禁用状态
 * 3. 测试交互反馈
 * ---
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import PhysicSwitch from './PhysicSwitch.vue';

describe('Settings/PhysicSwitch.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('正确渲染初始状态', () => {
    const wrapper = mount(PhysicSwitch, {
      props: {
        modelValue: false,
      },
    });

    const track = wrapper.find('.physic-switch');
    expect(track.classes()).not.toContain('physic-switch--active');

    const knob = wrapper.find('.physic-knob');
    expect(knob.exists()).toBe(true);
  });

  it('激活状态显示正确的样式', () => {
    const wrapper = mount(PhysicSwitch, {
      props: {
        modelValue: true,
      },
    });

    const track = wrapper.find('.physic-switch');
    expect(track.classes()).toContain('physic-switch--active');
  });

  it('点击开关切换状态', async () => {
    const wrapper = mount(PhysicSwitch, {
      props: {
        modelValue: false,
      },
    });

    const track = wrapper.find('.physic-switch');
    await track.trigger('click');

    // 验证 emit update:modelValue 事件
    expect(wrapper.emitted('update:modelValue')).toBeTruthy();
    expect(wrapper.emitted('update:modelValue')![0]).toEqual([true]);
  });

  it('禁用状态不响应点击', async () => {
    const wrapper = mount(PhysicSwitch, {
      props: {
        modelValue: false,
        disabled: true,
      },
    });

    const track = wrapper.find('.physic-switch');
    expect(track.classes()).toContain('physic-switch--disabled');

    await track.trigger('click');

    // 不应该 emit 事件
    expect(wrapper.emitted('update:modelValue')).toBeFalsy();
  });

  it('禁用状态显示正确的样式', () => {
    const wrapper = mount(PhysicSwitch, {
      props: {
        modelValue: false,
        disabled: true,
      },
    });

    const track = wrapper.find('.physic-switch');
    expect(track.classes()).toContain('physic-switch--disabled');
  });

  it('支持双向绑定', async () => {
    const wrapper = mount(PhysicSwitch, {
      props: {
        modelValue: false,
      },
    });

    // 初始状态
    expect(wrapper.props('modelValue')).toBe(false);

    // 切换到 true
    await wrapper.find('.physic-switch').trigger('click');
    expect(wrapper.emitted('update:modelValue')![0]).toEqual([true]);

    // 更新 props
    await wrapper.setProps({ modelValue: true });
    expect(wrapper.props('modelValue')).toBe(true);
    expect(wrapper.find('.physic-switch').classes()).toContain('physic-switch--active');
  });

  it('包含光晕效果元素', () => {
    const wrapper = mount(PhysicSwitch, {
      props: {
        modelValue: true,
      },
    });

    const glow = wrapper.find('.physic-bloom');
    expect(glow.exists()).toBe(true);
  });
});
