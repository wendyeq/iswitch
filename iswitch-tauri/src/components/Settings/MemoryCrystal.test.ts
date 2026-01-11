/**
 * ---
 * [INPUT]: {MemoryCrystal Component}
 *     - MemoryCrystal: source: ./MemoryCrystal.vue ([POS]: 配置状态提示）
 * [OUTPUT]: {测试结果} - 不同状态下的水晶展示
 * [POS]: iswitch-tauri/src/components/Settings/MemoryCrystal.test.ts
 * [PROTOCOL]:
 * 1. Mock i18n 输出原始 key
 * 2. 覆盖 synced / pending / missing 三种状态
 * 3. 验证 label/title 覆盖逻辑
 * ---
 */
import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import MemoryCrystal from './MemoryCrystal.vue';

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}));

type MemoryCrystalProps = {
  status: 'synced' | 'pending' | 'missing';
  label?: string;
  title?: string;
};

const mountCrystal = (props: MemoryCrystalProps) =>
  mount(MemoryCrystal, {
    props,
    global: {
      mocks: {
        $t: (key: string) => key,
      },
    },
  });

describe('Settings/MemoryCrystal.vue', () => {
  it('renders synced state with translated status text', () => {
    const wrapper = mountCrystal({ status: 'synced' });

    expect(wrapper.find('.crystal-orb').classes()).toContain('crystal--synced');
    expect(wrapper.text()).toContain('components.general.import.synced');
  });

  it('shows pending visuals and translation fallback', () => {
    const wrapper = mountCrystal({ status: 'pending' });

    expect(wrapper.find('.crystal-orb').classes()).toContain('crystal--pending');
    expect(wrapper.text()).toContain('components.general.import.cta');
  });

  it('allows overriding status label copy', () => {
    const wrapper = mountCrystal({ status: 'missing', label: 'Custom Label' });

    expect(wrapper.find('.crystal-status').text()).toBe('Custom Label');
    expect(wrapper.text()).toContain('components.general.import.label');
  });
});
