import { mount } from '@vue/test-utils';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import PortConflict from './PortConflict.vue';

// Mock dependencies
const invokeMock = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: any) => (args !== undefined ? invokeMock(cmd, args) : invokeMock(cmd)),
}));

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, args?: any) => {
      if (key === 'components.modals.portConflict.message' && args?.app) {
        return `Port blocked by ${args.app}`;
      }
      if (key === 'components.modals.portConflict.unknownApp') {
        return 'Unknown Application';
      }
      return key;
    },
  }),
}));

describe('PortConflict.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders correctly with blocker app name', () => {
    const wrapper = mount(PortConflict, {
      props: {
        blockerApp: 'Nginx',
      },
      global: {
        stubs: {
          BaseButton: {
            template: '<button><slot /></button>',
          },
        },
      },
    });

    expect(wrapper.text()).toContain('Port blocked by Nginx');
    expect(wrapper.text()).toContain('components.modals.portConflict.title');
  });

  it('renders with default label when no blocker provided', () => {
    const wrapper = mount(PortConflict, {
      props: {
        blockerApp: null,
      },
      global: {
        stubs: {
          BaseButton: {
            template: '<button><slot /></button>',
          },
        },
      },
    });
    expect(wrapper.text()).toContain('Port blocked by Unknown Application');
  });

  it('emits dismiss event when dismiss button clicked', async () => {
    const wrapper = mount(PortConflict, {
      global: {
        stubs: {
          BaseButton: {
            template: '<button @click="$emit(\'click\')"><slot /></button>',
          },
        },
      },
    });

    const buttons = wrapper.findAll('button');
    await buttons[0].trigger('click');

    expect(wrapper.emitted('dismiss')).toBeTruthy();
  });

  it('calls quit_app when quit button clicked', async () => {
    const wrapper = mount(PortConflict, {
      global: {
        stubs: {
          BaseButton: {
            template: '<button @click="$emit(\'click\')"><slot /></button>',
          },
        },
      },
    });

    const buttons = wrapper.findAll('button');
    await buttons[1].trigger('click');

    expect(invokeMock).toHaveBeenCalledWith('quit_app');
  });
});
