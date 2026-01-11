/**
 * ---
 * [INPUT]: {SyncCrystal Component}
 *     - SyncCrystal: source: ./SyncCrystal.vue ([POS]: 配置同步组件)
 * [OUTPUT]: {测试结果} - 配置同步交互测试
 * [POS]: SyncCrystal 单元测试
 * [PROTOCOL]:
 * 1. Mock Services (configImport)
 * 2. 验证配置导入流程
 * 3. 验证状态显示逻辑
 * ---
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import SyncCrystal from './SyncCrystal.vue';
import {
  fetchConfigImportStatus,
  fetchCodeSwitchImportStatus,
  fetchConfigImportStatusForFile,
  importFromCcSwitch,
  importFromCodeSwitch,
  importFromCustomFile,
} from '../../services/configImport';
import { showToast } from '../../utils/toast';

// Mock Services
vi.mock('../../services/configImport', () => ({
  fetchConfigImportStatus: vi.fn(),
  fetchCodeSwitchImportStatus: vi.fn(),
  fetchConfigImportStatusForFile: vi.fn(),
  importFromCcSwitch: vi.fn(),
  importFromCodeSwitch: vi.fn(),
  importFromCustomFile: vi.fn(),
}));

vi.mock('../../utils/toast', () => ({
  showToast: vi.fn(),
}));

const dialogMocks = vi.hoisted(() => ({
  open: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: dialogMocks.open,
}));

// Stub
const BaseButtonStub = {
  template: '<button class="btn-stub" @click="$emit(\'click\')"><slot /></button>',
  props: ['variant', 'size', 'disabled'],
  emits: ['click'],
};

const baseStatus = () => ({
  config_exists: true,
  config_path: '/tmp/config.json',
  pending_providers: true,
  pending_mcp: true,
  pending_provider_count: 1,
  pending_mcp_count: 1,
});

const createStatus = (overrides: Partial<ReturnType<typeof baseStatus>> = {}) => ({
  ...baseStatus(),
  ...overrides,
});

const mountSyncCrystal = () =>
  mount(SyncCrystal, {
    global: {
      stubs: {
        BaseButton: BaseButtonStub,
      },
    },
  });

describe('Settings/SyncCrystal.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(fetchConfigImportStatus).mockResolvedValue(baseStatus());
    vi.mocked(fetchCodeSwitchImportStatus).mockResolvedValue(baseStatus());
    vi.mocked(fetchConfigImportStatusForFile).mockResolvedValue(baseStatus());
    vi.mocked(importFromCcSwitch).mockResolvedValue(null as any);
    vi.mocked(importFromCodeSwitch).mockResolvedValue(null as any);
    vi.mocked(importFromCustomFile).mockResolvedValue(null as any);
  });

  it('组件加载时获取导入状态', async () => {
    vi.mocked(fetchConfigImportStatus).mockResolvedValue({
      config_exists: true,
      pending_providers: true,
      pending_mcp: true,
      pending_provider_count: 2,
      pending_mcp_count: 3,
      config_path: '/test/path',
    });
    vi.mocked(fetchCodeSwitchImportStatus).mockResolvedValue({
      config_exists: false,
      config_path: '',
      pending_providers: false,
      pending_mcp: false,
      pending_provider_count: 0,
      pending_mcp_count: 0,
    });

    mount(SyncCrystal, {
      global: {
        stubs: {
          BaseButton: BaseButtonStub,
        },
      },
    });

    await flushPromises();

    expect(fetchConfigImportStatus).toHaveBeenCalled();
    expect(fetchCodeSwitchImportStatus).toHaveBeenCalled();
  });

  it('显示待同步配置时允许导入', async () => {
    vi.mocked(fetchConfigImportStatus).mockResolvedValue({
      config_exists: true,
      pending_providers: true,
      pending_mcp: true,
      pending_provider_count: 2,
      pending_mcp_count: 3,
      config_path: '/test/path',
    });
    vi.mocked(fetchCodeSwitchImportStatus).mockResolvedValue({
      config_exists: false,
      config_path: '',
      pending_providers: false,
      pending_mcp: false,
      pending_provider_count: 0,
      pending_mcp_count: 0,
    });

    const wrapper = mount(SyncCrystal, {
      global: {
        stubs: {
          BaseButton: BaseButtonStub,
        },
      },
    });

    await flushPromises();

    // 应该显示导入按钮
    const buttons = wrapper.findAll('.btn-stub');
    expect(buttons.length).toBeGreaterThan(0);
  });

  it('点击导入按钮触发导入', async () => {
    vi.mocked(fetchConfigImportStatus).mockResolvedValue({
      config_exists: true,
      pending_providers: true,
      pending_mcp: true,
      pending_provider_count: 2,
      pending_mcp_count: 3,
      config_path: '/test/path',
    });
    vi.mocked(fetchCodeSwitchImportStatus).mockResolvedValue({
      config_exists: false,
      config_path: '',
      pending_providers: false,
      pending_mcp: false,
      pending_provider_count: 0,
      pending_mcp_count: 0,
    });
    vi.mocked(importFromCcSwitch).mockResolvedValue({
      imported_providers: 2,
      imported_mcp: 3,
      status: {
        config_exists: true,
        pending_providers: false,
        pending_mcp: false,
        pending_provider_count: 0,
        pending_mcp_count: 0,
        config_path: '/test/path',
      },
    });

    const wrapper = mount(SyncCrystal, {
      global: {
        stubs: {
          BaseButton: BaseButtonStub,
        },
      },
    });

    await flushPromises();

    // 找到第一个导入按钮并点击
    const buttons = wrapper.findAll('.btn-stub');
    await buttons[0].trigger('click');

    await flushPromises();

    expect(importFromCcSwitch).toHaveBeenCalled();
    expect(showToast).toHaveBeenCalled();
  });

  it('已同步状态显示正确', async () => {
    vi.mocked(fetchConfigImportStatus).mockResolvedValue({
      config_exists: true,
      pending_providers: false,
      pending_mcp: false,
      pending_provider_count: 0,
      pending_mcp_count: 0,
      config_path: '/test/path',
    });
    vi.mocked(fetchCodeSwitchImportStatus).mockResolvedValue({
      config_exists: false,
      config_path: '',
      pending_providers: false,
      pending_mcp: false,
      pending_provider_count: 0,
      pending_mcp_count: 0,
    });

    const wrapper = mount(SyncCrystal, {
      global: {
        stubs: {
          BaseButton: BaseButtonStub,
        },
      },
    });

    await flushPromises();

    // 应该有 synced class
    const crystal = wrapper.find('.sync-crystal');
    expect(crystal.classes()).toContain('sync-crystal--synced');
  });

  it('缺失配置状态显示正确', async () => {
    vi.mocked(fetchConfigImportStatus).mockResolvedValue({
      config_exists: false,
      config_path: '',
      pending_providers: false,
      pending_mcp: false,
      pending_provider_count: 0,
      pending_mcp_count: 0,
    });
    vi.mocked(fetchCodeSwitchImportStatus).mockResolvedValue({
      config_exists: false,
      config_path: '',
      pending_providers: false,
      pending_mcp: false,
      pending_provider_count: 0,
      pending_mcp_count: 0,
    });

    const wrapper = mount(SyncCrystal, {
      global: {
        stubs: {
          BaseButton: BaseButtonStub,
        },
      },
    });

    await flushPromises();

    const crystal = wrapper.find('.sync-crystal');
    expect(crystal.classes()).toContain('sync-crystal--missing');
  });

  it('上传自定义配置时加载状态并切换到 custom 源', async () => {
    vi.mocked(fetchConfigImportStatus).mockResolvedValue({
      config_exists: false,
      config_path: '',
      pending_providers: false,
      pending_mcp: false,
      pending_provider_count: 0,
      pending_mcp_count: 0,
    });
    vi.mocked(fetchCodeSwitchImportStatus).mockResolvedValue({
      config_exists: false,
      config_path: '',
      pending_providers: false,
      pending_mcp: false,
      pending_provider_count: 0,
      pending_mcp_count: 0,
    });
    dialogMocks.open.mockResolvedValue('/tmp/config.json');
    vi.mocked(fetchConfigImportStatusForFile).mockResolvedValue({
      config_exists: true,
      config_path: '/tmp/config.json',
      pending_providers: true,
      pending_mcp: true,
      pending_provider_count: 1,
      pending_mcp_count: 0,
    });

    const wrapper = mount(SyncCrystal, {
      global: {
        stubs: {
          BaseButton: BaseButtonStub,
        },
      },
    });

    await flushPromises();

    const uploadBtn = wrapper.findAll('.btn-stub').find(btn => btn.text().includes('components.general.import.upload'));
    expect(uploadBtn).toBeDefined();
    await uploadBtn!.trigger('click');
    await flushPromises();

    expect(dialogMocks.open).toHaveBeenCalled();
    expect(fetchConfigImportStatusForFile).toHaveBeenCalledWith('/tmp/config.json');

    const crystal = wrapper.find('.sync-crystal');
    expect(crystal.classes()).toContain('sync-crystal--pending');
    expect(wrapper.text()).toContain('components.general.import.detail');
  });

  it('未检测到待同步任务时 handleImport 不会触发服务', async () => {
    vi.mocked(fetchConfigImportStatus).mockResolvedValue(
      createStatus({ pending_providers: false, pending_mcp: false, pending_provider_count: 0, pending_mcp_count: 0 })
    );
    const wrapper = mountSyncCrystal();
    await flushPromises();

    await (wrapper.vm as any).handleImport('cc-switch');
    expect(importFromCcSwitch).not.toHaveBeenCalled();
  });

  it('code-switch 导入无新增内容时提示 empty', async () => {
    vi.mocked(fetchCodeSwitchImportStatus).mockResolvedValue(createStatus());
    vi.mocked(importFromCodeSwitch).mockResolvedValue({
      imported_providers: 0,
      imported_mcp: 0,
      status: createStatus({ pending_providers: false, pending_mcp: false }),
    } as any);

    const wrapper = mountSyncCrystal();
    await flushPromises();

    await (wrapper.vm as any).handleImport('code-switch');
    expect(showToast).toHaveBeenCalledWith('components.main.importConfig.empty');
  });

  it('自定义导入使用文件路径并支持清除状态', async () => {
    dialogMocks.open.mockResolvedValue('/tmp/custom.json');
    vi.mocked(fetchConfigImportStatusForFile).mockResolvedValue(
      createStatus({ config_path: '/tmp/custom.json', pending_mcp_count: 2 })
    );
    const wrapper = mountSyncCrystal();
    await flushPromises();

    const uploadBtn = wrapper.findAll('.btn-stub').find(btn => btn.text().includes('components.general.import.upload'));
    expect(uploadBtn).toBeDefined();
    await uploadBtn!.trigger('click');
    await flushPromises();

    await (wrapper.vm as any).handleImport('custom');
    expect(importFromCustomFile).toHaveBeenCalledWith('/tmp/custom.json');

    const clearBtn = wrapper.findAll('.btn-stub').find(btn => btn.text().includes('components.general.import.clear'));
    expect(clearBtn).toBeDefined();
    await clearBtn!.trigger('click');
    expect((wrapper.vm as any).customStatus).toBeNull();
  });

  it('导入异常会显示错误提示并恢复忙状态', async () => {
    vi.mocked(importFromCodeSwitch).mockRejectedValueOnce(new Error('boom'));
    const wrapper = mountSyncCrystal();
    await flushPromises();

    await (wrapper.vm as any).handleImport('code-switch');
    await flushPromises();

    expect(showToast).toHaveBeenCalledWith('components.main.importConfig.error', 'error');
    expect((wrapper.vm as any).isBusy).toBe(false);
  });

  it('拖拽上传会提示暂不支持并重置 dragging 状态', async () => {
    const wrapper = mountSyncCrystal();
    await flushPromises();
    const preventDefault = vi.fn();

    await (wrapper.vm as any).handleDragOver({ preventDefault } as unknown as DragEvent);
    expect(wrapper.classes()).toContain('sync-crystal--dragging');

    await (wrapper.vm as any).handleDrop({
      preventDefault,
      dataTransfer: { files: [{ name: 'config.json' }] },
    } as unknown as DragEvent);
    expect(showToast).toHaveBeenCalledWith('components.settings.sync.dragNotSupported', 'error');
    expect(wrapper.classes()).not.toContain('sync-crystal--dragging');
  });

  it('loadStatuses 捕获异常时记录日志而不抛出', async () => {
    const wrapper = mountSyncCrystal();
    await flushPromises();
    const error = new Error('status boom');
    vi.mocked(fetchConfigImportStatus).mockRejectedValueOnce(error);
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    await (wrapper.vm as any).loadStatuses();

    expect(consoleSpy).toHaveBeenCalledWith('failed to load import statuses', error);
    consoleSpy.mockRestore();
  });
});
