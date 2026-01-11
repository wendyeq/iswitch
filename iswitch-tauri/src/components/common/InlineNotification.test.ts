import { mount } from '@vue/test-utils';
import { describe, it, expect } from 'vitest';
import InlineNotification from './InlineNotification.vue';

/**
 * ---
 * [INPUT]:
 *   - source: iswitch-tauri/src/components/common/InlineNotification.vue ([POS]: Unit Test Target)
 * [OUTPUT]: {TestSuite} - InlineNotification Component Tests
 * [POS]: iswitch-tauri/src/components/common/InlineNotification.test.ts
 * [PROTOCOL]: FractalFlow v1.0
 * ---
 */

describe('InlineNotification', () => {
  it('renders nothing when visible is false', () => {
    const wrapper = mount(InlineNotification, {
      props: {
        visible: false,
        message: 'Test Message',
      },
      global: {
        stubs: {
          Teleport: true,
        },
      },
    });

    expect(wrapper.find('.inline-notification').exists()).toBe(false);
  });

  it('renders correctly when visible is true', () => {
    const wrapper = mount(InlineNotification, {
      props: {
        visible: true,
        message: 'Success Message',
      },
      global: {
        stubs: {
          Teleport: true,
        },
      },
    });

    const notification = wrapper.find('.inline-notification');
    expect(notification.exists()).toBe(true);
    expect(notification.text()).toContain('Success Message');
  });

  it('displays correct message prop', async () => {
    const wrapper = mount(InlineNotification, {
      props: {
        visible: true,
        message: 'Initial Message',
      },
      global: {
        stubs: {
          Teleport: true,
        },
      },
    });

    expect(wrapper.text()).toContain('Initial Message');

    await wrapper.setProps({ message: 'Updated Message' });
    expect(wrapper.text()).toContain('Updated Message');
  });
});
