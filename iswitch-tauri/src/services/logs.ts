/**
 * ---
 * [INPUT]: {Tauri Commands}
 *     - Commands: source: ../../src-tauri/src/commands/log.rs ([POS]: 日志命令)
 * [OUTPUT]: {Logs API} - 日志查询和统计相关的前端 API
 * [POS]: 日志服务层，封装请求日志和统计的 Tauri 调用
 * [PROTOCOL]:
 * 1. 重新导出 tauri.ts 中的日志相关函数
 * 2. 提供类型安全的日志接口
 * ---
 */

/**
 * 日志服务
 * 请求日志记录与统计查询
 */
import {
  fetchRequestLogs as tauriFetchRequestLogs,
  fetchLogProviders as tauriFetchLogProviders,
  fetchLogStats as tauriFetchLogStats,
  fetchProviderDailyStats as tauriFetchProviderDailyStats,
  fetchHeatmapStats as tauriFetchHeatmapStats,
  type RequestLog,
  type LogStats,
  type LogStatsSeries,
  type ProviderDailyStat,
  type HeatmapStat,
} from './tauri';

export const fetchRequestLogs = tauriFetchRequestLogs;
export const fetchLogProviders = tauriFetchLogProviders;
export const fetchLogStats = tauriFetchLogStats;
export const fetchProviderDailyStats = tauriFetchProviderDailyStats;
export const fetchHeatmapStats = tauriFetchHeatmapStats;
export type { RequestLog, LogStats, LogStatsSeries, ProviderDailyStat, HeatmapStat };
