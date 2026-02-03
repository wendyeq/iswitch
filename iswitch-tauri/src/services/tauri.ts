/**
 * ---
 * [INPUT]: {Tauri Commands}
 *     - Commands: source: ../../src-tauri/src/commands/ ([POS]: Rust 后端命令)
 * [OUTPUT]: {TypeScript API 函数} - 封装后的前端可调用函数
 * [POS]: Tauri IPC 调用适配层，为前端提供类型安全的后端 API
 * [PROTOCOL]:
 * 1. 统一错误处理
 * 2. 类型安全的接口定义
 * 3. 隐藏 Tauri invoke 实现细节
 * ---
 */

/**
 * Tauri 服务适配层
 *
 * 将原 Wails 调用模式 (Call.ByName) 转换为 Tauri invoke 模式
 * 这使得前端代码可以最小化改动
 */

import { invoke } from '@tauri-apps/api/core';

// === 版本服务 ===
export const fetchCurrentVersion = async (): Promise<string> => {
  return invoke<string>('get_version');
};

// === Claude/Codex 设置服务 ===
export type ClaudeProxyStatus = {
  enabled: boolean;
  proxy_url: string;
  status_message: string;
};

type Platform = 'claude' | 'codex';

export const fetchProxyStatus = async (platform: Platform): Promise<boolean> => {
  const command = platform === 'claude' ? 'get_claude_proxy_status' : 'get_codex_proxy_status';
  return invoke<boolean>(command);
};

export const enableProxy = async (platform: Platform): Promise<void> => {
  const command = platform === 'claude' ? 'enable_claude_proxy' : 'enable_codex_proxy';
  await invoke(command);
};

export const disableProxy = async (platform: Platform): Promise<void> => {
  const command = platform === 'claude' ? 'disable_claude_proxy' : 'disable_codex_proxy';
  await invoke(command);
};

// === 应用设置服务 ===
export type AppSettings = {
  show_heatmap: boolean;
  auto_start: boolean;
  proxy_port?: number;
  failover_threshold?: number;
  recovery_timeout_secs?: number;
};

// 前端fallback默认值（与后端 src/models/settings.rs 中的 Default 实现保持同步）
// 注意：这些值仅在无法从后端获取设置时使用（网络错误等极端情况）
const DEFAULT_SETTINGS: AppSettings = {
  show_heatmap: true,
  auto_start: false,
  proxy_port: 18099,
  failover_threshold: 5,
  recovery_timeout_secs: 300,
};

export const fetchAppSettings = async (): Promise<AppSettings> => {
  try {
    return await invoke<AppSettings>('get_app_settings');
  } catch (error) {
    console.error('[fetchAppSettings] Failed to fetch app settings:', error);
    return DEFAULT_SETTINGS;
  }
};

export const saveAppSettings = async (settings: AppSettings): Promise<AppSettings> => {
  return invoke<AppSettings>('save_app_settings', { settings });
};

// === 日志服务 ===
export type RequestLog = {
  id: number;
  platform: string;
  model: string;
  provider: string;
  http_code: number;
  input_tokens: number;
  output_tokens: number;
  cache_create_tokens: number;
  cache_read_tokens: number;
  reasoning_tokens: number;
  is_stream?: boolean | number;
  duration_sec?: number;
  created_at: string;
  total_cost?: number;
  input_cost?: number;
  output_cost?: number;
  cache_create_cost?: number;
  cache_read_cost?: number;
  ephemeral_5m_cost?: number;
  ephemeral_1h_cost?: number;
  has_pricing?: boolean;
};

type RequestLogQuery = {
  platform?: string;
  provider?: string;
  limit?: number;
};

export const fetchRequestLogs = async (query: RequestLogQuery = {}): Promise<RequestLog[]> => {
  const platform = query.platform ?? '';
  const provider = query.provider ?? '';
  const limit = query.limit ?? 100;
  return invoke<RequestLog[]>('list_request_logs', { platform, provider, limit });
};

export const fetchLogProviders = async (platform = ''): Promise<string[]> => {
  return invoke<string[]>('list_log_providers', { platform });
};

export type LogStatsSeries = {
  day: string;
  total_requests: number;
  input_tokens: number;
  output_tokens: number;
  reasoning_tokens: number;
  cache_create_tokens: number;
  cache_read_tokens: number;
  total_cost: number;
};

export type LogStats = {
  total_requests: number;
  input_tokens: number;
  output_tokens: number;
  reasoning_tokens: number;
  cache_create_tokens: number;
  cache_read_tokens: number;
  cost_total: number;
  cost_input: number;
  cost_output: number;
  cost_cache_create: number;
  cost_cache_read: number;
  series: LogStatsSeries[];
};

export const fetchLogStats = async (platform = '', provider = '', days = 30): Promise<LogStats> => {
  return invoke<LogStats>('get_log_stats', { platform, provider, days });
};

export type ProviderDailyStat = {
  provider: string;
  total_requests: number;
  successful_requests: number;
  failed_requests: number;
  success_rate: number;
  input_tokens: number;
  output_tokens: number;
  reasoning_tokens: number;
  cache_create_tokens: number;
  cache_read_tokens: number;
  cost_total: number;
  hourly_requests: number[];
};

export const fetchProviderDailyStats = async (days = 30): Promise<ProviderDailyStat[]> => {
  return invoke<ProviderDailyStat[]>('get_provider_daily_stats', { days });
};

export type HeatmapStat = {
  day: string;
  total_requests: number;
  input_tokens: number;
  output_tokens: number;
  reasoning_tokens: number;
  total_cost: number;
};

export const fetchHeatmapStats = async (days: number): Promise<HeatmapStat[]> => {
  const range = Number.isFinite(days) && days > 0 ? Math.floor(days) : 30;
  return invoke<HeatmapStat[]>('get_heatmap_stats', { days: range });
};

// === MCP 服务 ===
export type McpPlatform = 'claude-code' | 'codex';
export type McpServerType = 'stdio' | 'http';

export type McpServer = {
  name: string;
  type: McpServerType;
  command?: string;
  args: string[];
  env: Record<string, string>;
  url?: string;
  website?: string;
  tips?: string;
  enable_platform: McpPlatform[];
  enabled_in_claude: boolean;
  enabled_in_codex: boolean;
  missing_placeholders: string[];
};

export const fetchMcpServers = async (): Promise<McpServer[]> => {
  try {
    return await invoke<McpServer[]>('list_mcp_servers');
  } catch (error) {
    console.error('[fetchMcpServers] Failed to load MCP servers:', error);
    return [];
  }
};

export const saveMcpServers = async (servers: McpServer[]): Promise<void> => {
  await invoke('save_mcp_servers', { servers });
};

// === Skill 服务 ===
export type SkillSummary = {
  key: string;
  name: string;
  description: string;
  directory: string;
  readme_url: string;
  installed: boolean;
  repo_owner?: string;
  repo_name?: string;
  repo_branch?: string;
};

export type SkillRepoConfig = {
  owner: string;
  name: string;
  branch: string;
  enabled: boolean;
};

export type InstallSkillPayload = {
  directory: string;
  repo_owner?: string;
  repo_name?: string;
  repo_branch?: string;
};

export const fetchSkills = async (): Promise<SkillSummary[]> => {
  try {
    return await invoke<SkillSummary[]>('list_skills');
  } catch (error) {
    console.error('[fetchSkills] Failed to load skills:', error);
    return [];
  }
};

export const installSkill = async (payload: InstallSkillPayload): Promise<void> => {
  await invoke('install_skill', payload);
};

export const uninstallSkill = async (directory: string): Promise<void> => {
  await invoke('uninstall_skill', { directory });
};

export const fetchSkillRepos = async (): Promise<SkillRepoConfig[]> => {
  try {
    return await invoke<SkillRepoConfig[]>('list_skill_repos');
  } catch (error) {
    console.error('[fetchSkillRepos] Failed to load skill repos:', error);
    return [];
  }
};

export const addSkillRepo = async (repo: Partial<SkillRepoConfig>): Promise<SkillRepoConfig[]> => {
  const payload = {
    owner: repo.owner ?? '',
    name: repo.name ?? '',
    branch: repo.branch ?? 'main',
    enabled: repo.enabled ?? true,
  };
  try {
    return await invoke<SkillRepoConfig[]>('add_skill_repo', payload);
  } catch (error) {
    console.error('[addSkillRepo] Failed to add skill repo:', error);
    return [];
  }
};

export const removeSkillRepo = async (owner: string, name: string): Promise<SkillRepoConfig[]> => {
  try {
    return await invoke<SkillRepoConfig[]>('remove_skill_repo', { owner, name });
  } catch (error) {
    console.error('[removeSkillRepo] Failed to remove skill repo:', error);
    return [];
  }
};

// === 配置导入服务 ===
export type ConfigImportStatus = {
  config_exists: boolean;
  config_path: string;
  pending_providers: boolean;
  pending_mcp: boolean;
  pending_provider_count: number;
  pending_mcp_count: number;
};

export type ConfigImportResult = {
  status: ConfigImportStatus;
  imported_providers: number;
  imported_mcp: number;
};

const emptyStatus: ConfigImportStatus = {
  config_exists: false,
  config_path: '',
  pending_providers: false,
  pending_mcp: false,
  pending_provider_count: 0,
  pending_mcp_count: 0,
};

export const fetchConfigImportStatus = async (): Promise<ConfigImportStatus> => {
  try {
    return await invoke<ConfigImportStatus>('get_import_status');
  } catch {
    return emptyStatus;
  }
};

export const fetchConfigImportStatusForFile = async (path: string): Promise<ConfigImportStatus> => {
  try {
    return await invoke<ConfigImportStatus>('get_import_status_for_file', { path });
  } catch {
    return emptyStatus;
  }
};

export const importFromCcSwitch = async (): Promise<ConfigImportResult> => {
  return invoke<ConfigImportResult>('import_config');
};

export const importFromCustomFile = async (path: string): Promise<ConfigImportResult> => {
  return invoke<ConfigImportResult>('import_config_from_file', { path });
};

// === Code-Switch 导入服务 ===

/**
 * 获取 Code-Switch 导入状态
 *
 * 检查 ~/.code-switch/ 目录下的配置文件
 */
export const fetchCodeSwitchImportStatus = async (): Promise<ConfigImportStatus> => {
  try {
    return await invoke<ConfigImportStatus>('get_code_switch_import_status');
  } catch {
    return emptyStatus;
  }
};

/**
 * 从 Code-Switch 导入配置
 *
 * 聚合导入 ~/.code-switch/ 目录下的所有配置文件
 */
export const importFromCodeSwitch = async (): Promise<ConfigImportResult> => {
  return invoke<ConfigImportResult>('import_from_code_switch');
};

// === Provider 服务 ===
export type ProviderKind = 'claude' | 'codex';

export type Provider = {
  id: number;
  name: string;
  apiUrl: string;
  apiKey: string;
  supportedModels: string[];
  model_mapping: Record<string, string>;
  enabled: boolean;
  priority: number;
  officialSite?: string;
};

export const fetchProviders = async (kind: ProviderKind): Promise<Provider[]> => {
  try {
    return await invoke<Provider[]>('load_providers', { kind });
  } catch {
    return [];
  }
};

export const saveProviders = async (kind: ProviderKind, providers: Provider[]): Promise<void> => {
  await invoke('save_providers', { kind, providers });
};

export type ProxyStatus = {
  running: boolean;
  address: string;
  port: number;
};

export const getProxyStatus = async (): Promise<ProxyStatus> => {
  return invoke<ProxyStatus>('get_proxy_status');
};
