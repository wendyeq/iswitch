/**
 * ---
 * [INPUT]: {Tauri Commands}
 *     - Commands: source: ../../src-tauri/src/commands/mcp.rs ([POS]: MCP 命令)
 * [OUTPUT]: {MCP API} - MCP 服务器管理相关的前端 API
 * [POS]: MCP 服务层，封装 MCP 服务器配置管理的 Tauri 调用
 * [PROTOCOL]:
 * 1. 重新导出 tauri.ts 中的 MCP 相关函数
 * 2. 提供类型安全的 MCP 接口
 * ---
 */

/**
 * MCP 服务
 * MCP Server 配置管理
 */
import {
  fetchMcpServers as tauriFetchMcpServers,
  saveMcpServers as tauriSaveMcpServers,
  type McpServer,
  type McpPlatform,
  type McpServerType,
} from './tauri';

export const fetchMcpServers = tauriFetchMcpServers;
export const saveMcpServers = tauriSaveMcpServers;
export type { McpServer, McpPlatform, McpServerType };
