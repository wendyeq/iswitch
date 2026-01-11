/**
 * ---
 * [INPUT]: {service facades}
 *     - services: source: ./appSettings.ts, ./claudeSettings.ts, ./configImport.ts, ./logs.ts, ./mcp.ts, ./skill.ts, ./version.ts ([POS]: 前端服务层)
 * [OUTPUT]: {服务层单元测试} - 确认各 facade 仅做 Tauri API 的透传
 * [POS]: iswitch-tauri/src/services/reexports.test.ts
 * [PROTOCOL]:
 * 1. 读取 tauri.ts 原始实现
 * 2. 逐一比较引用是否一致
 * 3. 确保新 facade 不引入逻辑偏差
 * ---
 */
import { describe, it, expect } from 'vitest';
import * as tauriService from './tauri';
import * as appSettingsService from './appSettings';
import * as claudeSettingsService from './claudeSettings';
import * as configImportService from './configImport';
import * as logsService from './logs';
import * as mcpService from './mcp';
import * as skillService from './skill';
import * as versionService from './version';

describe('service facade re-exports', () => {
  it('appSettings proxies fetch/save implementations', () => {
    expect(appSettingsService.fetchAppSettings).toBe(tauriService.fetchAppSettings);
    expect(appSettingsService.saveAppSettings).toBe(tauriService.saveAppSettings);
  });

  it('claudeSettings proxies proxy control helpers', () => {
    expect(claudeSettingsService.fetchProxyStatus).toBe(tauriService.fetchProxyStatus);
    expect(claudeSettingsService.enableProxy).toBe(tauriService.enableProxy);
    expect(claudeSettingsService.disableProxy).toBe(tauriService.disableProxy);
  });

  it('configImport keeps every import helper aligned with tauri adapters', () => {
    expect(configImportService.fetchConfigImportStatus).toBe(tauriService.fetchConfigImportStatus);
    expect(configImportService.fetchConfigImportStatusForFile).toBe(tauriService.fetchConfigImportStatusForFile);
    expect(configImportService.importFromCcSwitch).toBe(tauriService.importFromCcSwitch);
    expect(configImportService.importFromCustomFile).toBe(tauriService.importFromCustomFile);
    expect(configImportService.fetchCodeSwitchImportStatus).toBe(tauriService.fetchCodeSwitchImportStatus);
    expect(configImportService.importFromCodeSwitch).toBe(tauriService.importFromCodeSwitch);
  });

  it('logs service surface matches tauri layer', () => {
    expect(logsService.fetchRequestLogs).toBe(tauriService.fetchRequestLogs);
    expect(logsService.fetchLogProviders).toBe(tauriService.fetchLogProviders);
    expect(logsService.fetchLogStats).toBe(tauriService.fetchLogStats);
    expect(logsService.fetchProviderDailyStats).toBe(tauriService.fetchProviderDailyStats);
    expect(logsService.fetchHeatmapStats).toBe(tauriService.fetchHeatmapStats);
  });

  it('mcp service proxies list/save functions', () => {
    expect(mcpService.fetchMcpServers).toBe(tauriService.fetchMcpServers);
    expect(mcpService.saveMcpServers).toBe(tauriService.saveMcpServers);
  });

  it('skill service proxies repo + install operations', () => {
    expect(skillService.fetchSkills).toBe(tauriService.fetchSkills);
    expect(skillService.installSkill).toBe(tauriService.installSkill);
    expect(skillService.uninstallSkill).toBe(tauriService.uninstallSkill);
    expect(skillService.fetchSkillRepos).toBe(tauriService.fetchSkillRepos);
    expect(skillService.addSkillRepo).toBe(tauriService.addSkillRepo);
    expect(skillService.removeSkillRepo).toBe(tauriService.removeSkillRepo);
  });

  it('version service exposes the version fetcher', () => {
    expect(versionService.fetchCurrentVersion).toBe(tauriService.fetchCurrentVersion);
  });
});
