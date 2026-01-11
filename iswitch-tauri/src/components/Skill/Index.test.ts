/**
 * ---
 * [INPUT]: {Skill/Index Component}
 *     - Skill/Index: source: ./Index.vue ([POS]: Skill 管理页)
 * [OUTPUT]: {测试结果} - Skill 列表、安装流程与仓库弹窗
 * [POS]: iswitch-tauri/src/components/Skill/Index.test.ts
 * [PROTOCOL]:
 * 1. Mock i18n、路由、Tauri shell 与 skill 服务接口
 * 2. 覆盖加载状态、安装/卸载操作
 * 3. 验证仓库弹窗内的新增与移除流程
 * ---
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { defineComponent, h, nextTick } from 'vue';
import SkillIndex from './Index.vue';
import type { SkillRepoConfig, SkillSummary } from '../../services/skill';

const skillService = vi.hoisted(() => ({
  fetchSkills: vi.fn<() => Promise<SkillSummary[]>>(),
  installSkill: vi.fn<(payload: unknown) => Promise<void>>(),
  uninstallSkill: vi.fn<(directory: string) => Promise<void>>(),
  fetchSkillRepos: vi.fn<() => Promise<SkillRepoConfig[]>>(),
  addSkillRepo: vi.fn<(repo: SkillRepoConfig) => Promise<SkillRepoConfig[]>>(),
  removeSkillRepo: vi.fn<(owner: string, name: string) => Promise<SkillRepoConfig[]>>(),
}));

vi.mock('../../services/skill', () => skillService);
const { fetchSkills, installSkill, uninstallSkill, fetchSkillRepos, addSkillRepo, removeSkillRepo } = skillService;

const routerMock = vi.hoisted(() => ({ push: vi.fn() }));
vi.mock('vue-router', () => ({
  useRouter: () => routerMock,
}));

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}));

const shellModule = vi.hoisted(() => ({ open: vi.fn().mockResolvedValue(undefined) }));
vi.mock('@tauri-apps/plugin-shell', () => shellModule);

const BaseModalStub = defineComponent({
  name: 'BaseModalStub',
  props: {
    open: { type: Boolean, default: false },
    title: { type: String, default: '' },
  },
  emits: ['close'],
  setup(props, { slots }) {
    return () => (props.open ? h('div', { class: 'base-modal-stub' }, slots.default?.()) : null);
  },
});

const createSkill = (overrides: Partial<SkillSummary> = {}): SkillSummary => ({
  key: overrides.key ?? `skill-${Math.random().toString(36).slice(2, 7)}`,
  name: 'Alpha Skill',
  description: 'desc',
  directory: 'alpha',
  readme_url: 'https://example.com',
  installed: false,
  repo_owner: 'team',
  repo_name: 'alpha',
  repo_branch: 'main',
  ...overrides,
});

const createRepo = (overrides: Partial<SkillRepoConfig> = {}): SkillRepoConfig => ({
  owner: 'team',
  name: 'alpha',
  branch: 'main',
  enabled: true,
  ...overrides,
});

const mountSkillPage = () =>
  mount(SkillIndex, {
    global: {
      stubs: {
        BaseModal: BaseModalStub,
      },
    },
  });

const mountAndFlush = async () => {
  const wrapper = mountSkillPage();
  await flushPromises();
  return wrapper;
};

describe('Skill/Index.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    fetchSkills.mockResolvedValue([createSkill()]);
    installSkill.mockResolvedValue();
    uninstallSkill.mockResolvedValue();
    fetchSkillRepos.mockResolvedValue([]);
    addSkillRepo.mockResolvedValue([]);
    removeSkillRepo.mockResolvedValue([]);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('renders loading state before showing skills list', async () => {
    fetchSkills.mockResolvedValue([createSkill({ name: 'Skill A', directory: 'skill-a' })]);
    const wrapper = mountSkillPage();
    await flushPromises();

    expect(fetchSkills).toHaveBeenCalledTimes(1);
    expect(wrapper.findAll('.skill-card')).toHaveLength(1);
    expect(wrapper.text()).toContain('Skill A');
    expect(wrapper.find('.skill-empty').exists()).toBe(false);
  });

  it('installs and uninstalls skills via action buttons', async () => {
    fetchSkills.mockResolvedValue([
      createSkill({ name: 'Installable', directory: 'installable', installed: false }),
      createSkill({ name: 'Installed', directory: 'installed', installed: true }),
    ]);

    const wrapper = await mountAndFlush();
    const cards = wrapper.findAll('.skill-card');

    const installButton = cards[0].find('button[title="components.skill.actions.install"]');
    await installButton.trigger('click');
    await flushPromises();

    expect(installSkill).toHaveBeenCalledWith({
      directory: 'installable',
      repo_owner: 'team',
      repo_name: 'alpha',
      repo_branch: 'main',
    });
    expect(wrapper.findAll('.skill-card')[0].find('button[title="components.skill.actions.uninstall"]').exists()).toBe(
      true
    );

    const uninstallButton = wrapper
      .findAll('.skill-card')[1]
      .find('button[title="components.skill.actions.uninstall"]');
    await uninstallButton.trigger('click');
    await flushPromises();

    expect(uninstallSkill).toHaveBeenCalledWith('installed');
    expect(wrapper.findAll('.skill-card')[1].find('button[title="components.skill.actions.install"]').exists()).toBe(
      true
    );
  });

  it('opens repo modal to add a repository and refreshes lists', async () => {
    fetchSkillRepos.mockResolvedValue([]);
    addSkillRepo.mockResolvedValue([createRepo({ owner: 'octo', name: 'beta', branch: 'dev' })]);

    const wrapper = await mountAndFlush();
    await wrapper.find('button[title="components.skill.repos.open"]').trigger('click');
    await flushPromises();

    const urlInput = wrapper.find('.repo-input-field input');
    const branchInput = wrapper.find('.repo-form-actions input');
    await urlInput.setValue('https://github.com/octo/beta');
    await branchInput.setValue('dev');

    await wrapper.find('form.skill-repo-form').trigger('submit.prevent');
    await flushPromises();

    expect(addSkillRepo).toHaveBeenCalledWith({
      owner: 'octo',
      name: 'beta',
      branch: 'dev',
      enabled: true,
    });
    expect(fetchSkills).toHaveBeenCalledTimes(2);
    expect(wrapper.find('.repo-input-field input').element).toHaveProperty('value', '');
  });

  it('removes repo entries from the modal list', async () => {
    fetchSkillRepos.mockResolvedValue([createRepo({ owner: 'octo', name: 'gamma' })]);
    removeSkillRepo.mockResolvedValue([]);

    const wrapper = await mountAndFlush();
    await wrapper.find('button[title="components.skill.repos.open"]').trigger('click');
    await flushPromises();

    await wrapper.find('button[title="components.skill.repos.removeLabel"]').trigger('click');
    await flushPromises();

    expect(removeSkillRepo).toHaveBeenCalledWith('octo', 'gamma');
    expect(fetchSkills).toHaveBeenCalledTimes(2);
  });

  it('surfaces list errors when skill loading fails', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    fetchSkills.mockRejectedValueOnce(new Error('boom'));

    const wrapper = mountSkillPage();
    await flushPromises();

    expect(wrapper.find('.skill-error').text()).toBe('components.skill.list.error');
    consoleSpy.mockRestore();
  });

  it('refresh 按钮会重新拉取列表', async () => {
    const wrapper = await mountAndFlush();
    vi.clearAllMocks();

    await wrapper.find('button[title="components.skill.actions.refresh"]').trigger('click');
    await flushPromises();

    expect(fetchSkills).toHaveBeenCalled();
    expect(fetchSkillRepos).toHaveBeenCalled();
  });

  it('handleInstall 在缺少 repo 信息时提示错误', async () => {
    fetchSkills.mockResolvedValue([createSkill({ name: 'Broken', repo_owner: undefined, repo_name: undefined })]);
    const wrapper = await mountAndFlush();
    const vm = wrapper.vm as any;

    await vm.handleInstall(vm.skills[0]);
    expect(vm.skillsError).toBe('components.skill.list.missingRepo');
  });

  it('安装/卸载失败时展示错误消息', async () => {
    fetchSkills.mockResolvedValue([
      createSkill({ name: 'Installable', directory: 'installable', installed: false }),
      createSkill({ name: 'Installed', directory: 'installed', installed: true }),
    ]);
    installSkill.mockRejectedValueOnce(new Error('install boom'));
    uninstallSkill.mockRejectedValueOnce(new Error('uninstall boom'));

    const wrapper = await mountAndFlush();
    const cards = wrapper.findAll('.skill-card');

    await cards[0].find('button[title="components.skill.actions.install"]').trigger('click');
    await flushPromises();
    expect(wrapper.find('.skill-error').text()).toContain('components.skill.actions.installError');

    await cards[1].find('button[title="components.skill.actions.uninstall"]').trigger('click');
    await flushPromises();
    expect(wrapper.find('.skill-error').text()).toContain('components.skill.actions.uninstallError');
  });

  it('提交仓库表单会校验输入并处理服务错误', async () => {
    const wrapper = await mountAndFlush();
    await wrapper.find('button[title="components.skill.repos.open"]').trigger('click');
    await flushPromises();
    const vm = wrapper.vm as any;

    vm.repoForm.url = 'invalid';
    await vm.submitRepo();
    expect(vm.repoError).toBe('components.skill.repos.formError');

    vm.repoForm.url = 'https://github.com/octo/beta';
    vm.repoForm.branch = 'dev';
    addSkillRepo.mockRejectedValueOnce(new Error('add fail'));
    await vm.submitRepo();
    expect(vm.repoError).toBe('components.skill.repos.addError');
  });

  it('移除仓库失败时显示错误并保持 busy 状态为 false', async () => {
    fetchSkillRepos.mockResolvedValue([createRepo({ owner: 'octo', name: 'beta' })]);
    removeSkillRepo.mockRejectedValueOnce(new Error('remove fail'));

    const wrapper = await mountAndFlush();
    await wrapper.find('button[title="components.skill.repos.open"]').trigger('click');
    await flushPromises();

    await wrapper.find('button[title="components.skill.repos.removeLabel"]').trigger('click');
    await flushPromises();

    const error = wrapper.find('.skill-error');
    expect(error.exists()).toBe(true);
    expect(error.text()).toBe('components.skill.repos.removeError');
    expect((wrapper.vm as any).repoBusy).toBe(false);
  });

  it('openGithub 捕获 shell 错误不影响 UI', async () => {
    shellModule.open.mockRejectedValueOnce(new Error('open fail'));
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    const wrapper = await mountAndFlush();

    await (wrapper.vm as any).openGithub('https://github.com/octo/beta');
    expect(shellModule.open).toHaveBeenCalled();
    expect(consoleSpy).toHaveBeenCalledWith('failed to open link', 'https://github.com/octo/beta');
    consoleSpy.mockRestore();
  });

  it('点击查看按钮会打开技能文档链接', async () => {
    const wrapper = await mountAndFlush();
    const viewButton = wrapper.find('button[title="components.skill.actions.view"]');
    await viewButton.trigger('click');
    expect(shellModule.open).toHaveBeenCalled();
  });

  it('仓库列表中的查看按钮会打开仓库链接', async () => {
    fetchSkillRepos.mockResolvedValue([createRepo({ owner: 'team', name: 'delta' })]);
    const wrapper = await mountAndFlush();
    await wrapper.find('button[title="components.skill.repos.open"]').trigger('click');
    await flushPromises();

    const repoView = wrapper.find('button[title="components.skill.repos.viewLabel"]');
    await repoView.trigger('click');
    expect(shellModule.open).toHaveBeenCalledWith('https://github.com/team/delta');
  });

  it('openRepoGithub 忽略缺少 owner/name 的仓库', async () => {
    const wrapper = await mountAndFlush();
    await (wrapper.vm as any).openRepoGithub({ owner: '', name: '', branch: 'main' });
    expect(shellModule.open).not.toHaveBeenCalled();
  });

  it('loadRepos 失败时展示错误消息', async () => {
    fetchSkillRepos.mockRejectedValue(new Error('boom'));
    const wrapper = await mountAndFlush();
    await wrapper.find('button[title="components.skill.repos.open"]').trigger('click');
    await flushPromises();
    const error = wrapper.find('.skill-error');
    expect(error.exists()).toBe(true);
    expect(error.text()).toBe('components.skill.repos.loadError');
  });

  it('点击返回按钮会跳转首页', async () => {
    const wrapper = await mountAndFlush();
    await wrapper.find('button[title="components.skill.actions.back"]').trigger('click');
    expect(routerMock.push).toHaveBeenCalledWith('/');
  });

  it('BaseModal close 事件会关闭仓库弹窗', async () => {
    const wrapper = await mountAndFlush();
    await wrapper.find('button[title="components.skill.repos.open"]').trigger('click');
    await flushPromises();
    const modal = wrapper.findComponent(BaseModalStub);
    expect(modal.exists()).toBe(true);
    await modal.vm.$emit('close');
    await nextTick();
    expect((wrapper.vm as any).repoModalOpen).toBe(false);
  });
});
