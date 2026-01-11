/**
 * ---
 * [INPUT]: {Tauri Commands}
 *     - Commands: source: ../../src-tauri/src/lib.rs ([POS]: 版本命令)
 * [OUTPUT]: {版本号字符串} - 当前应用版本
 * [POS]: 版本服务层，封装版本获取的 Tauri 调用
 * [PROTOCOL]:
 * 1. 重新导出 tauri.ts 中的版本函数
 * 2. 提供类型安全的版本接口
 * ---
 */

/**
 * 版本服务
 * 从 Tauri 后端获取应用版本号
 */
import { fetchCurrentVersion as tauriFetchCurrentVersion } from './tauri';

export const fetchCurrentVersion = tauriFetchCurrentVersion;
