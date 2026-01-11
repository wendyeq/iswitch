/**
 * ---
 * [INPUT]: {App.vue, @tauri-apps/api/event}
 *     - App.vue: source: ./App.vue ([POS]: 根组件)
 *     - listen: source: @tauri-apps/api/event ([POS]: 事件订阅接口)
 * [OUTPUT]: {App 组件单测} - 验证代理冲突弹窗逻辑
 * [POS]: iswitch-tauri/src/App.test.ts
 * [PROTOCOL]:
 * 1. 捕获 proxy-error 事件并展示模态框
 * 2. 校验 dismiss 行为及卸载时的清理逻辑
 * ---
 */
import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App.vue';
import { listen } from '@tauri-apps/api/event';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

const listenMock = vi.mocked(listen);
const PortConflictStub = {
  name: 'PortConflictModal',
  props: {
    blockerApp: {
      type: String,
      default: null,
    },
  },
  emits: ['dismiss'],
  template: '<div class="port-conflict-stub" :data-blocker="blockerApp" @click="$emit(\'dismiss\')" />',
};

const RouterViewStub = {
  name: 'RouterView',
  template: '<div class="router-view-stub" />',
};

const waitForUpdates = () => Promise.resolve().then(() => Promise.resolve());

describe('App.vue', () => {
  beforeEach(() => {
    listenMock.mockReset();
  });

  it('renders router outlet and ignores unrelated proxy events', async () => {
    let capturedHandler: any = null;
    listenMock.mockImplementation(async (_event, handler) => {
      capturedHandler = handler;
      return () => {};
    });

    const wrapper = mount(App, {
      global: {
        stubs: {
          RouterView: RouterViewStub,
          PortConflictModal: PortConflictStub,
        },
      },
    });

    await waitForUpdates();
    expect(wrapper.find('.router-view-stub').exists()).toBe(true);
    expect(wrapper.find('.port-conflict-stub').exists()).toBe(false);

    capturedHandler?.({ event: 'proxy-error', id: 0, payload: { type: 'OTHER', port: 1234 } });
    await waitForUpdates();
    expect(wrapper.find('.port-conflict-stub').exists()).toBe(false);

    await wrapper.unmount();
  });

  it('shows conflict modal on proxy-error and cleans up listeners on unmount', async () => {
    let capturedHandler: any = null;
    const unlisten = vi.fn();
    listenMock.mockImplementation(async (_event, handler) => {
      capturedHandler = handler;
      return unlisten;
    });

    const wrapper = mount(App, {
      global: {
        stubs: {
          RouterView: RouterViewStub,
          PortConflictModal: PortConflictStub,
        },
      },
    });

    await waitForUpdates();
    expect(listenMock).toHaveBeenCalledWith('proxy-error', expect.any(Function));

    capturedHandler?.({
      event: 'proxy-error',
      id: 0,
      payload: {
        type: 'PORT_CONFLICT',
        port: 2233,
        blocker: { name: 'MockApp' },
      },
    });
    await waitForUpdates();
    const modal = wrapper.find('.port-conflict-stub');
    expect(modal.exists()).toBe(true);
    expect(modal.attributes('data-blocker')).toBe('MockApp');

    await modal.trigger('click');
    await waitForUpdates();
    expect(wrapper.find('.port-conflict-stub').exists()).toBe(false);

    await wrapper.unmount();
    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  it('logs an error when proxy listener registration fails', async () => {
    const error = new Error('fail to listen');
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    listenMock.mockRejectedValue(error);

    const wrapper = mount(App, {
      global: {
        stubs: {
          RouterView: RouterViewStub,
          PortConflictModal: PortConflictStub,
        },
      },
    });

    await waitForUpdates();
    expect(consoleSpy).toHaveBeenCalledWith('failed to register proxy-error listener', error);

    consoleSpy.mockRestore();
    await wrapper.unmount();
  });
});
