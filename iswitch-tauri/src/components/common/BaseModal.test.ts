/**
 * ---
 * [INPUT]: {BaseModal Component}
 *     - BaseModal: source: ./BaseModal.vue ([POS]: 通用模态框)
 * [OUTPUT]: {测试结果} - 模态框渲染与交互测试
 * [POS]: iswitch-tauri/src/components/common/BaseModal.test.ts
 * [PROTOCOL]:
 * 1. Mock HeadlessUI 组件
 * 2. 验证标题、插槽与变体样式
 * 3. 验证关闭按钮与 Dialog close 事件
 * ---
 */
import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { defineComponent, h } from 'vue';
import BaseModal from './BaseModal.vue';

function createHeadlessStub(name: string) {
  return defineComponent({
    name,
    props: {
      as: { type: String, default: 'div' },
      show: { type: Boolean, default: undefined },
      open: { type: Boolean, default: undefined },
    },
    emits: ['close'],
    inheritAttrs: false,
    setup(props, { slots, emit, attrs }) {
      return () => {
        const hiddenByShow = props.show !== undefined && !props.show;
        const hiddenByOpen = props.open !== undefined && !props.open;
        if (hiddenByShow || hiddenByOpen) {
          return null;
        }

        return h(
          props.as || 'div',
          {
            ...attrs,
            class: [name, attrs?.class].filter(Boolean),
            'data-headless': name,
            onClose: () => emit('close'),
          },
          slots.default?.() ?? []
        );
      };
    },
  });
}

vi.mock('@headlessui/vue', () => ({
  Dialog: createHeadlessStub('Dialog'),
  DialogPanel: createHeadlessStub('DialogPanel'),
  DialogTitle: createHeadlessStub('DialogTitle'),
  TransitionChild: createHeadlessStub('TransitionChild'),
  TransitionRoot: createHeadlessStub('TransitionRoot'),
}));

const mountModal = (props = {}) =>
  mount(BaseModal, {
    props: {
      open: true,
      title: 'Test Modal',
      ...props,
    },
    slots: {
      default: '<p class="slot-body">Hello world</p>',
    },
  });

describe('common/BaseModal.vue', () => {
  it('renders title and slot content when open', () => {
    const wrapper = mountModal();

    expect(wrapper.find('.modal-title').text()).toBe('Test Modal');
    expect(wrapper.find('.slot-body').exists()).toBe(true);
  });

  it('does not render Dialog panel when closed', () => {
    const wrapper = mountModal({ open: false });

    expect(wrapper.find('.modal').exists()).toBe(false);
  });

  it('applies confirm variant styling', () => {
    const wrapper = mountModal({ variant: 'confirm' });

    expect(wrapper.find('.modal').classes()).toContain('confirm-modal');
  });

  it('emits close when header button clicked', async () => {
    const wrapper = mountModal();

    await wrapper.find('.ghost-icon').trigger('click');
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('forwards Dialog close event to parent', async () => {
    const wrapper = mountModal();
    const dialog = wrapper.findComponent({ name: 'Dialog' });

    dialog.vm.$emit('close');
    await wrapper.vm.$nextTick();

    expect(wrapper.emitted('close')).toBeTruthy();
  });
});
