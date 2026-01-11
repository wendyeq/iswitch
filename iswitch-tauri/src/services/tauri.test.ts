/**
 * ---
 * [INPUT]: {tauri service}
 *     - tauri: source: ./tauri.ts ([POS]: Tauri API 调用封装)
 * [OUTPUT]: {测试结果} - 服务层单元测试
 * [POS]: 核心服务层 API 调用测试
 * [PROTOCOL]:
 * 1. Mock Tauri invoke
 * 2. 测试各种 API 调用的返回值处理
 * ---
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  fetchCurrentVersion,
  fetchProxyStatus,
  enableProxy,
  disableProxy,
  fetchAppSettings,
  saveAppSettings,
  fetchProviders,
  saveProviders,
  fetchMcpServers,
  saveMcpServers,
  fetchRequestLogs,
  fetchLogProviders,
  fetchLogStats,
  fetchProviderDailyStats,
  fetchHeatmapStats,
  fetchSkills,
  installSkill,
  uninstallSkill,
  fetchSkillRepos,
  addSkillRepo,
  removeSkillRepo,
  fetchConfigImportStatus,
  fetchConfigImportStatusForFile,
  fetchCodeSwitchImportStatus,
  importFromCcSwitch,
  importFromCustomFile,
  importFromCodeSwitch,
  getProxyStatus,
} from './tauri';

// Mock invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('tauri service', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('fetchCurrentVersion', () => {
    it('返回版本号', async () => {
      vi.mocked(invoke).mockResolvedValueOnce('1.0.0');

      const version = await fetchCurrentVersion();

      expect(invoke).toHaveBeenCalledWith('get_version');
      expect(version).toBe('1.0.0');
    });

    it('失败时抛出异常', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed'));

      await expect(fetchCurrentVersion()).rejects.toThrow('Failed');
    });
  });

  describe('fetchProxyStatus', () => {
    it('获取代理状态(boolean)', async () => {
      vi.mocked(invoke).mockResolvedValueOnce(true);

      const enabled = await fetchProxyStatus('claude');

      expect(invoke).toHaveBeenCalledWith('get_claude_proxy_status');
      expect(enabled).toBe(true);
    });

    it('失败时抛出异常', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed'));

      await expect(fetchProxyStatus('claude')).rejects.toThrow('Failed');
    });
  });

  describe('fetchAppSettings', () => {
    it('获取应用设置成功', async () => {
      const mockSettings = {
        show_heatmap: true,
        auto_start: false,
        proxy_port: 18099,
        failover_threshold: 3,
        recovery_timeout_secs: 300,
      };
      vi.mocked(invoke).mockResolvedValueOnce(mockSettings);

      const settings = await fetchAppSettings();

      expect(invoke).toHaveBeenCalledWith('get_app_settings');
      expect(settings).toEqual(mockSettings);
    });

    it('失败时返回默认设置', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed'));

      const settings = await fetchAppSettings();

      expect(settings.show_heatmap).toBe(true);
      expect(settings.proxy_port).toBe(18099);
    });
  });

  describe('fetchProviders', () => {
    it('获取 Providers 成功', async () => {
      const mockProviders = [{ id: 1, name: 'Test' }];
      vi.mocked(invoke).mockResolvedValueOnce(mockProviders);

      const providers = await fetchProviders('claude');

      expect(invoke).toHaveBeenCalledWith('load_providers', { kind: 'claude' });
      expect(providers).toEqual(mockProviders);
    });

    it('失败时返回空数组', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed'));

      const providers = await fetchProviders('claude');

      expect(providers).toEqual([]);
    });
  });

  describe('fetchMcpServers', () => {
    it('获取 MCP 服务器列表成功', async () => {
      const mockServers = [{ name: 'server1' }];
      vi.mocked(invoke).mockResolvedValueOnce(mockServers);

      const servers = await fetchMcpServers();

      expect(invoke).toHaveBeenCalledWith('list_mcp_servers');
      expect(servers).toEqual(mockServers);
    });

    it('失败时返回空数组', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed'));

      const servers = await fetchMcpServers();

      expect(servers).toEqual([]);
    });
  });
});

describe('Proxy Control', () => {
  it('enableProxy calls correct command', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    await enableProxy('claude');
    expect(invoke).toHaveBeenCalledWith('enable_claude_proxy');
    await enableProxy('codex');
    expect(invoke).toHaveBeenCalledWith('enable_codex_proxy');
  });

  it('disableProxy calls correct command', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    await disableProxy('claude');
    expect(invoke).toHaveBeenCalledWith('disable_claude_proxy');
    await disableProxy('codex');
    expect(invoke).toHaveBeenCalledWith('disable_codex_proxy');
  });

  it('getProxyStatus calls correct command', async () => {
    const mockStatus = { running: true, address: '127.0.0.1', port: 18099 };
    vi.mocked(invoke).mockResolvedValue(mockStatus);
    const status = await getProxyStatus();
    expect(invoke).toHaveBeenCalledWith('get_proxy_status');
    expect(status).toEqual(mockStatus);
  });
});

describe('saveAppSettings', () => {
  it('saves settings successfully', async () => {
    const mockSettings = {
      show_heatmap: true,
      auto_start: false,
      proxy_port: 18099,
      failover_threshold: 3,
      recovery_timeout_secs: 300,
    };
    vi.mocked(invoke).mockResolvedValue(mockSettings);
    const result = await saveAppSettings(mockSettings);
    expect(invoke).toHaveBeenCalledWith('save_app_settings', { settings: mockSettings });
    expect(result).toEqual(mockSettings);
  });
});

describe('Log Services', () => {
  it('fetchRequestLogs calls list_request_logs', async () => {
    const mockLogs = [{ id: 1 }];
    vi.mocked(invoke).mockResolvedValue(mockLogs);
    const logs = await fetchRequestLogs({ limit: 50 });
    expect(invoke).toHaveBeenCalledWith('list_request_logs', { platform: '', provider: '', limit: 50 });
    expect(logs).toEqual(mockLogs);
  });

  it('fetchLogProviders calls list_log_providers', async () => {
    const mockProviders = ['p1', 'p2'];
    vi.mocked(invoke).mockResolvedValue(mockProviders);
    const providers = await fetchLogProviders('claude');
    expect(invoke).toHaveBeenCalledWith('list_log_providers', { platform: 'claude' });
    expect(providers).toEqual(mockProviders);
  });

  it('fetchLogStats calls get_log_stats', async () => {
    const mockStats = { total_requests: 100 };
    vi.mocked(invoke).mockResolvedValue(mockStats);
    const stats = await fetchLogStats();
    expect(invoke).toHaveBeenCalledWith('get_log_stats', { platform: '' });
    expect(stats).toEqual(mockStats);
  });

  it('fetchProviderDailyStats calls get_provider_daily_stats', async () => {
    const mockStats = [{ provider: 'p1' }];
    vi.mocked(invoke).mockResolvedValue(mockStats);
    const stats = await fetchProviderDailyStats(7);
    expect(invoke).toHaveBeenCalledWith('get_provider_daily_stats', { days: 7 });
    expect(stats).toEqual(mockStats);
  });

  it('fetchHeatmapStats calls get_heatmap_stats', async () => {
    const mockStats = [{ day: '2023-01-01' }];
    vi.mocked(invoke).mockResolvedValue(mockStats);
    const stats = await fetchHeatmapStats(7);
    expect(invoke).toHaveBeenCalledWith('get_heatmap_stats', { days: 7 });
    expect(stats).toEqual(mockStats);
  });
});

describe('saveMcpServers', () => {
  it('saves MCP servers', async () => {
    const servers: any[] = [];
    vi.mocked(invoke).mockResolvedValue(undefined);
    await saveMcpServers(servers);
    expect(invoke).toHaveBeenCalledWith('save_mcp_servers', { servers });
  });
});

describe('Skill Services', () => {
  it('fetchSkills handles success and error', async () => {
    vi.mocked(invoke).mockResolvedValue([{ key: 's1' }]);
    const skills = await fetchSkills();
    expect(skills).toHaveLength(1);

    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    const empty = await fetchSkills();
    expect(empty).toEqual([]);
  });

  it('installSkill calls correct command', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const payload = { directory: 'd1' };
    await installSkill(payload);
    expect(invoke).toHaveBeenCalledWith('install_skill', payload);
  });

  it('uninstallSkill calls correct command', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    await uninstallSkill('d1');
    expect(invoke).toHaveBeenCalledWith('uninstall_skill', { directory: 'd1' });
  });

  it('repo management functions', async () => {
    vi.mocked(invoke).mockResolvedValue([]);

    await fetchSkillRepos();
    expect(invoke).toHaveBeenCalledWith('list_skill_repos');

    await addSkillRepo({ owner: 'o' });
    expect(invoke).toHaveBeenCalledWith('add_skill_repo', expect.objectContaining({ owner: 'o' }));

    await removeSkillRepo('o', 'n');
    expect(invoke).toHaveBeenCalledWith('remove_skill_repo', { owner: 'o', name: 'n' });
  });

  it('repo management falls back to空数组 when调用失败', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('boom'));

    expect(await fetchSkillRepos()).toEqual([]);
    expect(await addSkillRepo({ owner: 'o', name: 'k' })).toEqual([]);
    expect(await removeSkillRepo('o', 'n')).toEqual([]);
  });
});

describe('Config Import', () => {
  it('fetchConfigImportStatus handles success and error', async () => {
    const status = { config_exists: true };
    vi.mocked(invoke).mockResolvedValue(status);
    expect(await fetchConfigImportStatus()).toEqual(status);

    vi.mocked(invoke).mockRejectedValue(new Error('fail'));
    expect(await fetchConfigImportStatus()).toEqual(expect.objectContaining({ config_exists: false }));
  });

  it('importFromCcSwitch calls correct command', async () => {
    vi.mocked(invoke).mockResolvedValue({ status: {}, imported_providers: 1 });
    await importFromCcSwitch();
    expect(invoke).toHaveBeenCalledWith('import_config');
  });

  it('importFromCustomFile calls correct command', async () => {
    vi.mocked(invoke).mockResolvedValue({ status: {}, imported_providers: 1 });
    await importFromCustomFile('path');
    expect(invoke).toHaveBeenCalledWith('import_config_from_file', { path: 'path' });
  });

  it('fetchConfigImportStatusForFile handles错误时返回 emptyStatus', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));
    const status = await fetchConfigImportStatusForFile('config.json');
    expect(status).toEqual(expect.objectContaining({ config_exists: false }));
  });

  it('fetchCodeSwitchImportStatus handles error 并 fallback', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('nope'));
    const status = await fetchCodeSwitchImportStatus();
    expect(status).toEqual(expect.objectContaining({ config_exists: false }));
  });

  it('importFromCodeSwitch invokes tauri 命令', async () => {
    vi.mocked(invoke).mockResolvedValue({ status: {}, imported_providers: 0, imported_mcp: 0 });
    await importFromCodeSwitch();
    expect(invoke).toHaveBeenCalledWith('import_from_code_switch');
  });
});

describe('saveProviders', () => {
  it('saves providers', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    await saveProviders('claude', []);
    expect(invoke).toHaveBeenCalledWith('save_providers', { kind: 'claude', providers: [] });
  });
});
