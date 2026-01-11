/**
 * ---
 * [INPUT]: {usageHeatmap}
 *     - usageHeatmap: source: ./usageHeatmap.ts ([POS]: 热力图数据处理)
 * [OUTPUT]: {测试结果} - 热力图数据处理单元测试
 * [POS]: 热力图数据转换和计算测试
 * [PROTOCOL]:
 * 1. 测试数据转换
 * 2. 测试边界条件
 * ---
 */
import { describe, it, expect } from 'vitest';
import {
  buildUsageHeatmapMatrix,
  generateFallbackUsageHeatmap,
  calculateHeatmapDayRange,
  HEATMAP_ROWS,
  BUCKETS_PER_DAY,
  DEFAULT_HEATMAP_DAYS,
} from '../data/usageHeatmap';

describe('usageHeatmap', () => {
  describe('常量', () => {
    it('HEATMAP_ROWS 应该是 8', () => {
      expect(HEATMAP_ROWS).toBe(8);
    });

    it('BUCKETS_PER_DAY 应该是 3', () => {
      expect(BUCKETS_PER_DAY).toBe(3);
    });

    it('DEFAULT_HEATMAP_DAYS 应该是 14', () => {
      expect(DEFAULT_HEATMAP_DAYS).toBe(14);
    });
  });

  describe('calculateHeatmapDayRange', () => {
    it('返回默认天数当未提供参数', () => {
      const result = calculateHeatmapDayRange();
      expect(result).toBe(DEFAULT_HEATMAP_DAYS);
    });

    it('返回指定天数', () => {
      expect(calculateHeatmapDayRange(7)).toBe(7);
      expect(calculateHeatmapDayRange(30)).toBe(30);
    });

    it('处理无效天数返回默认值', () => {
      expect(calculateHeatmapDayRange(0)).toBe(DEFAULT_HEATMAP_DAYS);
      expect(calculateHeatmapDayRange(-1)).toBe(DEFAULT_HEATMAP_DAYS);
    });

    it('小数天数会向下取整', () => {
      expect(calculateHeatmapDayRange(7.9)).toBe(7);
    });
  });

  describe('generateFallbackUsageHeatmap', () => {
    it('生成正确数量的列', () => {
      const result = generateFallbackUsageHeatmap();
      // 默认 14 天，每天 3 个 bucket
      expect(result.length).toBe(DEFAULT_HEATMAP_DAYS * BUCKETS_PER_DAY);
    });

    it('每列包含正确数量的行', () => {
      const result = generateFallbackUsageHeatmap();
      result.forEach(column => {
        expect(column.length).toBe(HEATMAP_ROWS);
      });
    });

    it('所有单元格都有 0 请求数', () => {
      const result = generateFallbackUsageHeatmap();
      result.forEach(column => {
        column.forEach(cell => {
          expect(cell.requests).toBe(0);
          expect(cell.intensity).toBe(0);
        });
      });
    });

    it('可以生成自定义天数的热力图', () => {
      const days = 7;
      const result = generateFallbackUsageHeatmap(days);
      expect(result.length).toBe(days * BUCKETS_PER_DAY);
    });
  });

  describe('buildUsageHeatmapMatrix', () => {
    it('处理空数据返回空热力图', () => {
      const result = buildUsageHeatmapMatrix([]);
      expect(result.length).toBe(DEFAULT_HEATMAP_DAYS * BUCKETS_PER_DAY);
      result.forEach(column => {
        column.forEach(cell => {
          expect(cell.requests).toBe(0);
        });
      });
    });

    it('处理 undefined 返回空热力图', () => {
      const result = buildUsageHeatmapMatrix(undefined);
      expect(result.length).toBe(DEFAULT_HEATMAP_DAYS * BUCKETS_PER_DAY);
    });

    it('正确处理有数据的情况', () => {
      const now = new Date();
      const hour = now.getHours().toString().padStart(2, '0');
      const month = (now.getMonth() + 1).toString().padStart(2, '0');
      const day = now.getDate().toString().padStart(2, '0');
      const year = now.getFullYear();

      const stats = [
        {
          day: `${year}-${month}-${day} ${hour}`,
          total_requests: 10,
          input_tokens: 100,
          output_tokens: 50,
          reasoning_tokens: 0,
          cache_create_tokens: 0,
          cache_read_tokens: 0,
          total_cost: 0.01,
        },
      ];

      const result = buildUsageHeatmapMatrix(stats);

      // 应该有数据被填入
      const hasData = result.some(column => column.some(cell => cell.requests > 0));
      expect(hasData).toBe(true);
    });

    it('每个单元格有正确的结构', () => {
      const result = buildUsageHeatmapMatrix([]);
      const cell = result[0][0];

      expect(cell).toHaveProperty('label');
      expect(cell).toHaveProperty('dateKey');
      expect(cell).toHaveProperty('requests');
      expect(cell).toHaveProperty('inputTokens');
      expect(cell).toHaveProperty('outputTokens');
      expect(cell).toHaveProperty('reasoningTokens');
      expect(cell).toHaveProperty('cost');
      expect(cell).toHaveProperty('intensity');
    });

    it('计算正确的 intensity', () => {
      const now = new Date();
      const hour = now.getHours().toString().padStart(2, '0');
      const month = (now.getMonth() + 1).toString().padStart(2, '0');
      const day = now.getDate().toString().padStart(2, '0');
      const year = now.getFullYear();

      const stats = [
        {
          day: `${year}-${month}-${day} ${hour}`,
          total_requests: 100,
          input_tokens: 1000,
          output_tokens: 500,
          reasoning_tokens: 0,
          cache_create_tokens: 0,
          cache_read_tokens: 0,
          total_cost: 0.1,
        },
      ];

      const result = buildUsageHeatmapMatrix(stats);

      // 最大请求数的单元格应该有最高 intensity
      const maxIntensityCell = result.flat().find(cell => cell.requests === 100);
      if (maxIntensityCell) {
        expect(maxIntensityCell.intensity).toBe(4); // 最大等级
      }
    });

    it('同一时间段的统计会累加请求与 Token', () => {
      const now = new Date();
      const hour = now.getHours().toString().padStart(2, '0');
      const month = (now.getMonth() + 1).toString().padStart(2, '0');
      const day = now.getDate().toString().padStart(2, '0');
      const year = now.getFullYear();
      const bucketKey = `${year}-${month}-${day} ${hour}`;

      const stats = [
        {
          day: bucketKey,
          total_requests: 5,
          input_tokens: 10,
          output_tokens: 5,
          reasoning_tokens: 0,
          total_cost: 0.01,
        },
        {
          day: bucketKey,
          total_requests: 7,
          input_tokens: 20,
          output_tokens: 10,
          reasoning_tokens: 0,
          total_cost: 0.02,
        },
      ] as any;

      const result = buildUsageHeatmapMatrix(stats, 1);
      const merged = result.flat().find(cell => cell.requests === 12);
      expect(merged?.inputTokens).toBe(30);
      expect(merged?.outputTokens).toBe(15);
    });

    it('支持 MM-DD HH 日期格式并忽略非法日期', () => {
      const now = new Date();
      const hour = now.getHours().toString().padStart(2, '0');
      const month = (now.getMonth() + 1).toString().padStart(2, '0');
      const day = now.getDate().toString().padStart(2, '0');
      const shortKey = `${month}-${day} ${hour}`;

      const stats = [
        {
          day: shortKey,
          total_requests: 4,
          input_tokens: 40,
          output_tokens: 20,
          reasoning_tokens: 0,
          total_cost: 0.04,
        },
        {
          day: 'invalid-key',
          total_requests: 100,
          input_tokens: 0,
          output_tokens: 0,
          reasoning_tokens: 0,
          total_cost: 0,
        },
      ] as any;

      const result = buildUsageHeatmapMatrix(stats, 1);
      const validCell = result.flat().find(cell => cell.requests === 4);
      expect(validCell).toBeTruthy();
      const hasInvalid = result.flat().some(cell => cell.requests === 100);
      expect(hasInvalid).toBe(false);
    });
  });
});
