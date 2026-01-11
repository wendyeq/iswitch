/**
 * ---
 * [INPUT]: {Tauri Commands}
 *     - Commands: source: ../../src-tauri/src/commands/claude.rs ([POS]: Claude 设置命令)
 *     - Commands: source: ../../src-tauri/src/commands/codex.rs ([POS]: Codex 设置命令)
 * [OUTPUT]: {Claude/Codex API} - Claude 和 Codex 代理设置相关的前端 API
 * [POS]: Claude/Codex 设置服务层，封装平台代理配置的 Tauri 调用
 * [PROTOCOL]:
 * 1. 重新导出 tauri.ts 中的代理设置函数
 * 2. 提供类型安全的代理接口
 * ---
 */

/**
 * Claude/Codex 设置服务
 * 管理 Claude Code 和 Codex 的代理配置
 */
import {
  fetchProxyStatus as tauriFetchProxyStatus,
  enableProxy as tauriEnableProxy,
  disableProxy as tauriDisableProxy,
  type ClaudeProxyStatus,
} from './tauri';

export const fetchProxyStatus = tauriFetchProxyStatus;
export const enableProxy = tauriEnableProxy;
export const disableProxy = tauriDisableProxy;
export type { ClaudeProxyStatus };
