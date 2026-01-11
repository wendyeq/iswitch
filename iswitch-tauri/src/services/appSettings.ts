/**
 * ---
 * [INPUT]: {Tauri Commands}
 *     - Commands: source: ../../src-tauri/src/commands/settings.rs ([POS]: 设置命令)
 * [OUTPUT]: {AppSettings API} - 应用设置相关的前端 API
 * [POS]: 应用设置服务层，封装设置相关的 Tauri 调用
 * [PROTOCOL]:
 * 1. 重新导出 tauri.ts 中的设置相关函数
 * 2. 提供类型安全的设置接口
 * ---
 */

/**
 * 应用设置服务
 * 管理热力图显示、自动启动等设置
 */
import {
  fetchAppSettings as tauriFetchAppSettings,
  saveAppSettings as tauriSaveAppSettings,
  type AppSettings,
} from './tauri';

export const fetchAppSettings = tauriFetchAppSettings;
export const saveAppSettings = tauriSaveAppSettings;
export type { AppSettings };
