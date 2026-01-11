/**
 * ---
 * [INPUT]: {Tauri Commands}
 *     - Commands: source: ../../src-tauri/src/commands/settings.rs ([POS]: 导入命令)
 * [OUTPUT]: {Config Import API} - 配置导入相关的前端 API
 * [POS]: 配置导入服务层，封装从旧版应用导入配置的 Tauri 调用
 * [PROTOCOL]:
 * 1. 重新导出 tauri.ts 中的导入相关函数
 * 2. 提供类型安全的导入接口
 * 3. 支持多个旧版应用格式
 * ---
 */

/**
 * 配置导入服务
 * 从旧版 cc-switch 和 code-switch 导入配置
 */
import {
  fetchConfigImportStatus as tauriFetchConfigImportStatus,
  fetchConfigImportStatusForFile as tauriFetchConfigImportStatusForFile,
  importFromCcSwitch as tauriImportFromCcSwitch,
  importFromCustomFile as tauriImportFromCustomFile,
  fetchCodeSwitchImportStatus as tauriFetchCodeSwitchImportStatus,
  importFromCodeSwitch as tauriImportFromCodeSwitch,
  type ConfigImportStatus,
  type ConfigImportResult,
} from './tauri';

export const fetchConfigImportStatus = tauriFetchConfigImportStatus;
export const fetchConfigImportStatusForFile = tauriFetchConfigImportStatusForFile;
export const importFromCcSwitch = tauriImportFromCcSwitch;
export const importFromCustomFile = tauriImportFromCustomFile;
export const fetchCodeSwitchImportStatus = tauriFetchCodeSwitchImportStatus;
export const importFromCodeSwitch = tauriImportFromCodeSwitch;
export type { ConfigImportStatus, ConfigImportResult };
