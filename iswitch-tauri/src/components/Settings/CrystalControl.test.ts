/**
 * ---
 * [INPUT]: {CrystalControl Component}
 *     - CrystalControl: source: ./CrystalControl.vue ([POS]: 水晶控制台组件)
 * [OUTPUT]: {测试结果} - 水晶控制台交互测试
 * [POS]: Crystal Control 页面集成测试
 * [PROTOCOL]:
 * 1. Mock Services (appSettings)
 * 2. 验证设置加载与显示
 * 3. 验证设置修改与保存
 * 4. 验证 UI 交互逻辑
 * ---
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import CrystalControl from './CrystalControl.vue';
import { fetchAppSettings, saveAppSettings } from '../../services/appSettings';
import {
  fetchConfigImportStatus,
  importFromCcSwitch,
  fetchCodeSwitchImportStatus,
  importFromCodeSwitch,
} from '../../services/configImport';
import { showToast } from '../../utils/toast';
import { setupI18n } from '../../utils/i18n';
import { getVersion } from '@tauri-apps/api/app';
import { pushMock } from '../../test/setup';

// Mock Services
vi.mock('../../services/appSettings', () => ({
  fetchAppSettings: vi.fn(),
  saveAppSettings: vi.fn(),
}));

vi.mock('../../services/configImport', () => ({
  fetchConfigImportStatus: vi.fn(),
  importFromCcSwitch: vi.fn(),
  fetchCodeSwitchImportStatus: vi.fn(),
  importFromCodeSwitch: vi.fn(),
}));

vi.mock('../../utils/toast', () => ({
  showToast: vi.fn(),
}));

vi.mock('../../utils/i18n', () => ({
  setupI18n: vi.fn(),
}));

// Stubs
const PhysicSwitchStub = {
  template: '<div class="physic-switch-stub"><slot /></div>',
  props: ['modelValue', 'disabled'],
  emits: ['update:modelValue'],
};
const SettingsSectionStub = {
  template: '<div class="section-stub"><slot /></div>',
  props: ['title'],
};
const SyncCrystalStub = {
  template: '<div class="sync-crystal-stub">SyncCrystal</div>',
};
const LanguageSwitcherStub = {
  template: '<div>LanguageSwitcher</div>',
};

const mountCrystal = () =>
  mount(CrystalControl, {
    global: {
      stubs: {
        PhysicSwitch: PhysicSwitchStub,
        SettingsSection: SettingsSectionStub,
        SyncCrystal: SyncCrystalStub,
        LanguageSwitcher: LanguageSwitcherStub,
      },
      mocks: {
        $t: (key: string) => key,
      },
    },
  });

const defaultSettings = {
  show_heatmap: true,
  auto_start: false,
  proxy_port: 18099,
  failover_threshold: 3,
  recovery_timeout_secs: 300,
};

const defaultStatus = {
  config_exists: true,
  pending_providers: true,
  pending_mcp: true,
  pending_provider_count: 1,
  pending_mcp_count: 1,
};

describe('Settings/CrystalControl.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(fetchAppSettings).mockResolvedValue(defaultSettings);
    vi.mocked(saveAppSettings).mockResolvedValue(defaultSettings);
    vi.mocked(fetchConfigImportStatus).mockResolvedValue(defaultStatus as any);
    vi.mocked(fetchCodeSwitchImportStatus).mockResolvedValue({
      ...defaultStatus,
      config_exists: false,
      pending_providers: false,
      pending_mcp: false,
    } as any);
    vi.mocked(importFromCcSwitch).mockResolvedValue(null as any);
    vi.mocked(importFromCodeSwitch).mockResolvedValue(null as any);
    vi.mocked(getVersion).mockResolvedValue('0.2.0');
  });

  it('加载并渲染应用设置', async () => {
    const mockSettings = {
      show_heatmap: true,
      auto_start: true,
      proxy_port: 18099,
      failover_threshold: 3,
      recovery_timeout_secs: 300,
    };

    vi.mocked(fetchAppSettings).mockResolvedValue(mockSettings);

    const wrapper = mountCrystal();

    // 验证 fetchAppSettings 被调用
    expect(fetchAppSettings).toHaveBeenCalled();

    await flushPromises();

    // 验证热力图开关状态
    const heatmapSwitch = wrapper.findComponent(PhysicSwitchStub);
    expect(heatmapSwitch.props('modelValue')).toBe(true);

    // 验证自动启动开关状态
    const autoStartSwitch = wrapper.findAllComponents(PhysicSwitchStub)[1];
    expect(autoStartSwitch.props('modelValue')).toBe(true);
  });

  it('修改热力图设置触发保存', async () => {
    const mockSettings = {
      show_heatmap: true,
      auto_start: false,
      proxy_port: 18099,
      failover_threshold: 3,
      recovery_timeout_secs: 300,
    };
    vi.mocked(fetchAppSettings).mockResolvedValue(mockSettings);

    const wrapper = mountCrystal();

    await flushPromises();

    // 修改热力图开关
    const heatmapSwitch = wrapper.findComponent(PhysicSwitchStub);
    await heatmapSwitch.vm.$emit('update:modelValue', false);

    // 验证 saveAppSettings 被调用，且 show_heatmap 为 false
    expect(saveAppSettings).toHaveBeenCalledWith(
      expect.objectContaining({
        show_heatmap: false,
      })
    );
  });

  it('点击返回按钮导航到主页', async () => {
    vi.useFakeTimers();
    vi.mocked(fetchAppSettings).mockResolvedValue({
      show_heatmap: true,
      auto_start: false,
      proxy_port: 18099,
      failover_threshold: 3,
      recovery_timeout_secs: 300,
    });

    const wrapper = mountCrystal();

    await flushPromises();

    // 点击返回按钮
    const backButton = wrapper.find('.close-btn');
    await backButton.trigger('click');

    // 快进时间
    vi.advanceTimersByTime(400);

    // 验证导航到主页
    expect(pushMock).toHaveBeenCalledWith('/');
    vi.useRealTimers();
  });

  it('不会渲染被移除的高级设置字段', async () => {
    vi.mocked(fetchAppSettings).mockResolvedValue({
      show_heatmap: false,
      auto_start: true,
      proxy_port: 18100,
      failover_threshold: 8,
      recovery_timeout_secs: 900,
    });

    const wrapper = mountCrystal();

    await flushPromises();

    const html = wrapper.text();
    expect(html).not.toContain('components.general.label.proxyPort');
    expect(html).not.toContain('components.general.label.failoverThreshold');
    expect(html).not.toContain('components.general.label.recoveryTimeout');
    expect(wrapper.findAll('input[type="number"]').length).toBe(0);
  });

  it('同步 cc-switch 配置成功时提示并更新状态', async () => {
    vi.mocked(importFromCcSwitch).mockResolvedValue({
      imported_providers: 2,
      imported_mcp: 1,
      status: { ...defaultStatus, pending_providers: false, pending_mcp: false },
    } as any);

    const wrapper = mountCrystal();
    await flushPromises();

    const syncButton = wrapper.findAll('.import-link')[0];
    await syncButton.trigger('click');
    await flushPromises();

    expect(importFromCcSwitch).toHaveBeenCalledTimes(1);
    expect(showToast).toHaveBeenCalledWith('components.main.importConfig.success');
  });

  it('同步失败时展示错误提示并重置忙状态', async () => {
    vi.mocked(importFromCcSwitch).mockRejectedValueOnce(new Error('fail'));
    const wrapper = mountCrystal();
    await flushPromises();

    const syncButton = wrapper.findAll('.import-link')[0];
    await syncButton.trigger('click');
    await flushPromises();

    expect(showToast).toHaveBeenCalledWith('components.main.importConfig.error', 'error');
    expect(syncButton.attributes('disabled')).toBeUndefined();
  });

  it('code-switch 导入成功时更新状态并提示', async () => {
    vi.mocked(importFromCodeSwitch).mockResolvedValue({
      imported_providers: 0,
      imported_mcp: 0,
      status: { ...defaultStatus, pending_providers: false, pending_mcp: false },
    } as any);
    const wrapper = mountCrystal();
    await flushPromises();

    const codeButton = wrapper.findAll('.import-link')[1];
    await codeButton.trigger('click');
    await flushPromises();

    expect(importFromCodeSwitch).toHaveBeenCalledTimes(1);
    expect(showToast).toHaveBeenCalledWith('components.main.importConfig.success');
    const vm = wrapper.vm as any;
    expect(vm.codeSwitchImportStatus.pending_providers).toBe(false);
  });

  it('persist 在加载期间不会保存，并在成功后广播事件', async () => {
    const dispatchSpy = vi.spyOn(window, 'dispatchEvent');
    const wrapper = mountCrystal();

    // settingsLoading 仍为 true
    await (wrapper.vm as any).persist();
    expect(saveAppSettings).not.toHaveBeenCalled();

    await flushPromises();
    await (wrapper.vm as any).persist();
    expect(saveAppSettings).toHaveBeenCalledTimes(1);
    expect(dispatchSpy).toHaveBeenCalledWith(expect.any(CustomEvent));
    dispatchSpy.mockRestore();
  });

  it('切换语言调用 setupI18n', async () => {
    const wrapper = mountCrystal();
    await flushPromises();

    await (wrapper.vm as any).switchLang('en');
    expect(setupI18n).toHaveBeenCalledWith('en');
  });

  it('获取版本失败时显示默认版本号', async () => {
    vi.mocked(getVersion).mockRejectedValueOnce(new Error('boom'));
    const wrapper = mountCrystal();
    await flushPromises();

    expect(wrapper.find('.slate-version').text()).toBe('v0.0.0');
  });
});
