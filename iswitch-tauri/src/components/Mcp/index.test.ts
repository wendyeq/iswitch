/**
 * ---
 * [INPUT]: {Mcp/index Component}
 *     - Mcp/index: source: ./index.vue ([POS]: MCP 管理页)
 * [OUTPUT]: {测试结果} - MCP 服务列表与配置对话框覆盖
 * [POS]: iswitch-tauri/src/components/Mcp/index.test.ts
 * [PROTOCOL]:
 * 1. Mock 路由、i18n 与服务层依赖
 * 2. 覆盖加载 / 空状态、表单 CRUD、平台开关与错误提示
 * 3. 验证缺少占位符时的提示与保存流程
 * ---
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { defineComponent, h } from 'vue';
import McpPage from './index.vue';
import type { McpServer } from '../../services/mcp';

const mcpService = vi.hoisted(() => ({
  fetchMcpServers: vi.fn<() => Promise<McpServer[]>>(),
  saveMcpServers: vi.fn<(servers: McpServer[]) => Promise<void>>(),
}));

vi.mock('../../services/mcp', () => mcpService);
const { fetchMcpServers, saveMcpServers } = mcpService;

const toastModule = vi.hoisted(() => ({ showToast: vi.fn() }));
vi.mock('../../utils/toast', () => toastModule);
const { showToast } = toastModule;

const routerMock = vi.hoisted(() => ({ push: vi.fn() }));
vi.mock('vue-router', () => ({
  useRouter: () => routerMock,
}));

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock('../../icons/lobeIconMap', () => ({
  __esModule: true,
  default: { mcp: '<svg />', alpha: '<svg />' },
}));

const BaseModalStub = defineComponent({
  name: 'BaseModalStub',
  props: {
    open: { type: Boolean, default: false },
    title: { type: String, default: '' },
    variant: { type: String, default: 'default' },
  },
  emits: ['close'],
  setup(props, { slots }) {
    return () => (props.open ? h('div', { class: 'base-modal-stub' }, slots.default?.()) : null);
  },
});

const createServer = (overrides: Partial<McpServer> = {}): McpServer => ({
  name: 'Alpha',
  type: 'stdio',
  command: 'run.sh',
  args: [],
  env: {},
  url: '',
  website: '',
  tips: '',
  enable_platform: [],
  enabled_in_claude: false,
  enabled_in_codex: false,
  missing_placeholders: [],
  ...overrides,
});

const mountMcp = () =>
  mount(McpPage, {
    global: {
      stubs: {
        BaseModal: BaseModalStub,
      },
    },
  });

const mountAndFlush = async () => {
  const wrapper = mountMcp();
  await flushPromises();
  return wrapper;
};

describe('Mcp/index.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    fetchMcpServers.mockResolvedValue([createServer()]);
    saveMcpServers.mockResolvedValue();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('loads servers on mount and renders cards after loading state', async () => {
    fetchMcpServers.mockResolvedValue([createServer({ name: 'Server One' })]);

    const wrapper = mountMcp();
    await flushPromises();

    expect(fetchMcpServers).toHaveBeenCalledTimes(1);
    expect(wrapper.findAll('.automation-card')).toHaveLength(1);
    expect(wrapper.text()).toContain('Server One');
    expect(wrapper.find('.empty-state').exists()).toBe(false);
  });

  it('creates a new server through the modal form and persists to backend', async () => {
    fetchMcpServers.mockResolvedValue([]);
    const wrapper = await mountAndFlush();

    await wrapper.find('button[aria-label="components.mcp.controls.create"]').trigger('click');
    const vm = wrapper.vm as unknown as { modalState: { form: Record<string, any> } };
    vm.modalState.form.name = 'Delta';
    vm.modalState.form.command = 'delta.sh';

    await wrapper.find('form.vendor-form').trigger('submit.prevent');
    await flushPromises();

    expect(saveMcpServers).toHaveBeenCalledTimes(1);
    const payload = saveMcpServers.mock.calls[0][0] as McpServer[];
    expect(payload.some(server => server.name === 'Delta')).toBe(true);
  });

  it('handles platform toggles, blocking placeholders and persisting valid changes', async () => {
    fetchMcpServers.mockResolvedValue([
      createServer({ name: 'Blocked', missing_placeholders: ['TOKEN'] }),
      createServer({ name: 'Allowed' }),
    ]);
    const wrapper = await mountAndFlush();

    const cards = wrapper.findAll('.automation-card');
    expect(cards).toHaveLength(2);

    const blockedToggle = cards[0].find('input[type="checkbox"]');
    await blockedToggle.setValue(true);
    expect(showToast).toHaveBeenCalledWith('components.mcp.toast.placeholder', 'error');
    expect(saveMcpServers).not.toHaveBeenCalled();

    showToast.mockClear();

    const allowedToggle = cards[1].find('input[type="checkbox"]');
    await allowedToggle.setValue(true);
    await flushPromises();

    expect(saveMcpServers).toHaveBeenCalledTimes(1);
    const payload = saveMcpServers.mock.calls[0][0] as McpServer[];
    const saved = payload.find(server => server.name === 'Allowed');
    expect(saved?.enable_platform).toContain('claude-code');
  });

  it('shows error message when server loading fails', async () => {
    const error = new Error('fail');
    fetchMcpServers.mockRejectedValueOnce(error);
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    const wrapper = mountMcp();
    await flushPromises();

    expect(wrapper.find('.alert-error').text()).toBe('components.mcp.list.loadError');
    consoleSpy.mockRestore();
  });

  it('confirms deletion via confirm modal and saves the new list', async () => {
    fetchMcpServers.mockResolvedValueOnce([createServer({ name: 'RemoveMe' })]);
    fetchMcpServers.mockResolvedValueOnce([]);

    const wrapper = mountMcp();
    await flushPromises();

    await wrapper.find('button[aria-label="components.mcp.list.delete"]').trigger('click');
    expect(wrapper.findAll('.base-modal-stub')).toHaveLength(1);

    const confirmButton = wrapper
      .findAll('button')
      .find(button => button.text() === 'components.mcp.form.actions.delete');
    expect(confirmButton).toBeDefined();
    await confirmButton!.trigger('click');
    await flushPromises();

    expect(saveMcpServers).toHaveBeenCalledTimes(1);
    const payload = saveMcpServers.mock.calls[0][0] as McpServer[];
    expect(payload.some(server => server.name === 'RemoveMe')).toBe(false);
  });

  it('modal平台勾选会在检测到占位符时阻止切换', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;
    vm.modalState.open = true;
    vm.modalState.form.url = 'https://example.com/{TOKEN}';
    vm.modalState.form.enablePlatform = ['codex'];
    const event = { target: { checked: true } } as unknown as Event;

    await vm.onModalPlatformToggle('claude-code', event);
    expect(showToast).toHaveBeenCalledWith('components.mcp.toast.placeholder', 'error');
    showToast.mockClear();

    vm.modalState.form.url = '';
    await vm.onModalPlatformToggle('claude-code', { target: { checked: true } } as unknown as Event);
    expect(vm.modalState.form.enablePlatform).toContain('claude-code');
  });

  it('openEditModal 会填充 env、argsText 与 dashboard 信息', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;

    vm.openEditModal(
      createServer({
        name: 'Editable',
        type: 'http',
        url: 'https://api.example.com',
        args: ['--port', '9000'],
        env: { API_KEY: 'secret' },
        enable_platform: ['codex'],
      })
    );

    expect(vm.modalState.form.name).toBe('Editable');
    expect(vm.modalState.form.argsText).toBe('--port\n9000');
    expect(vm.modalState.form.envEntries[0].key).toBe('API_KEY');
    expect(vm.modalState.form.enablePlatform).toContain('codex');
  });

  it('removeEnvEntry 保留至少一行并支持删除额外行', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;
    const [first] = vm.modalState.form.envEntries;
    vm.removeEnvEntry(first.id);
    expect(vm.modalState.form.envEntries).toHaveLength(1);

    vm.addEnvEntry();
    expect(vm.modalState.form.envEntries).toHaveLength(2);
    const lastId = vm.modalState.form.envEntries[1].id;
    vm.removeEnvEntry(lastId);
    expect(vm.modalState.form.envEntries).toHaveLength(1);
  });

  it('submitModal 校验名称、命令和 URL', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;

    vm.openCreateModal();
    vm.modalState.form.name = '   ';
    await vm.submitModal();
    expect(vm.modalError).toBe('components.mcp.form.errors.name');

    vm.modalState.form.name = 'CLI';
    vm.modalState.form.type = 'stdio';
    vm.modalState.form.command = '   ';
    await vm.submitModal();
    expect(vm.modalError).toBe('components.mcp.form.errors.command');

    vm.modalState.form.command = 'run.sh';
    vm.modalState.form.type = 'http';
    vm.modalState.form.url = '   ';
    await vm.submitModal();
    expect(vm.modalError).toBe('components.mcp.form.errors.url');
  });

  it('submitModal 阻止重复名称并在编辑时覆盖记录', async () => {
    fetchMcpServers.mockResolvedValueOnce([createServer({ name: 'Alpha', enabled_in_claude: true })]);
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;

    vm.openCreateModal();
    vm.modalState.form.name = 'Alpha';
    vm.modalState.form.command = 'run.sh';
    await vm.submitModal();
    expect(vm.modalError).toBe('components.mcp.form.errors.duplicate');

    vm.openEditModal(createServer({ name: 'Alpha', enabled_in_claude: true }));
    vm.modalState.form.command = 'edited.sh';
    await vm.submitModal();
    expect(saveMcpServers).toHaveBeenCalled();
  });

  it('detectPlaceholders 合并 URL 与参数中的变量', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;
    const detected = vm.detectPlaceholders('https://api/{TOKEN}', '--key {API_KEY}\n--token {TOKEN}');
    expect(detected).toEqual(expect.arrayContaining(['TOKEN', 'API_KEY']));
  });

  it('icon 与 summary 辅助函数覆盖多种分支', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;
    expect(vm.iconSvg('Alpha')).toBe('<svg />');
    expect(vm.iconSvg('')).toBe('<svg />');
    expect(vm.serverInitials('Moon River Labs')).toBe('MR');
    expect(vm.serverInitials('')).toBe('MC');
    expect(vm.serverSummary(createServer({ type: 'http', url: 'https://mcp.dev' }))).toContain(
      'components.mcp.types.httpShort'
    );
    expect(vm.serverSummary(createServer({ type: 'stdio', command: 'python app.py' }))).toContain(
      'components.mcp.types.stdioShort'
    );
    expect(vm.typeLabel('http')).toBe('components.mcp.types.http');
    expect(vm.typeLabel('stdio')).toBe('components.mcp.types.stdio');
    expect(vm.platformActive(createServer({ enabled_in_claude: true }), 'claude-code')).toBe(true);
    expect(vm.platformEnabled(createServer({ enable_platform: ['codex'] }), 'codex')).toBe(true);
  });

  it('persistServers 捕获保存异常并显示错误', async () => {
    saveMcpServers.mockRejectedValueOnce(new Error('boom'));
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    await vm.persistServers();
    await flushPromises();

    const alert = wrapper.find('.alert-error');
    expect(alert.exists()).toBe(true);
    expect(alert.text()).toBe('components.mcp.list.saveError');
    consoleSpy.mockRestore();
  });

  it('confirmDelete removes the selected server and persists changes', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;

    vm.servers = [createServer({ name: 'Keep' }), createServer({ name: 'Drop' })];
    vm.requestDelete(vm.servers[1]);
    saveMcpServers.mockClear();
    await vm.confirmDelete();

    expect(vm.servers.some((server: McpServer) => server.name === 'Drop')).toBe(false);
    expect(saveMcpServers).toHaveBeenCalled();
  });

  it('parseEnv filters empty keys and navigation helpers push routes', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;
    const env = vm.parseEnv([
      { id: 1, key: '   ', value: 'ignored' },
      { id: 2, key: 'API_KEY', value: 'secret' },
    ]);
    expect(env).toEqual({ API_KEY: 'secret' });

    routerMock.push.mockClear();
    vm.goHome();
    expect(routerMock.push).toHaveBeenCalledWith('/');
    vm.goToSettings();
    expect(routerMock.push).toHaveBeenCalledWith('/settings');

    fetchMcpServers.mockClear();
    await vm.reload();
    expect(fetchMcpServers).toHaveBeenCalled();
  });

  it('serverSummary falls back to generic labels when url or command missing', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;
    expect(vm.serverSummary(createServer({ type: 'http', url: '', command: '' }))).toBe(
      'components.mcp.types.httpShort'
    );
    expect(vm.serverSummary(createServer({ type: 'stdio', command: '', url: '' }))).toBe(
      'components.mcp.types.stdioShort'
    );
  });

  it('onPlatformToggle handles missing targets, unknown servers, and removal flow', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;

    await vm.onPlatformToggle(createServer({ name: 'Ghost' }), 'claude-code', { target: null } as unknown as Event);
    await vm.onPlatformToggle(createServer({ name: 'Unknown' }), 'codex', {
      target: { checked: true },
    } as unknown as Event);

    vm.servers = [createServer({ name: 'Toggleable', enable_platform: ['claude-code'] })];
    saveMcpServers.mockClear();
    await vm.onPlatformToggle(vm.servers[0], 'claude-code', { target: { checked: false } } as unknown as Event);

    expect(vm.servers[0].enable_platform).not.toContain('claude-code');
    expect(saveMcpServers).toHaveBeenCalled();
  });

  it('onModalPlatformToggle safely ignores events without targets', async () => {
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;
    vm.modalState.open = true;
    vm.modalState.form.enablePlatform = [];

    await vm.onModalPlatformToggle('claude-code', { target: null } as unknown as Event);
    expect(vm.modalState.form.enablePlatform).toHaveLength(0);
  });
});
