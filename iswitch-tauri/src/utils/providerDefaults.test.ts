/**
 * ---
 * [INPUT]: {provider defaults}
 *     - defaults: source: ./providerDefaults.ts ([POS]: 供应商默认配置)
 * [OUTPUT]: {默认值测试} - 验证常见供应商的默认 URL、映射与官网
 * [POS]: iswitch-tauri/src/utils/providerDefaults.test.ts
 * [PROTOCOL]:
 * 1. 覆盖所有导出函数
 * 2. 校验具体供应商与未知分支
 * ---
 */
import { describe, it, expect } from 'vitest';
import {
  PROVIDER_DEFAULTS,
  getProviderDefaultMapping,
  getProviderDefaultOfficialSite,
  getProviderDefaultUrl,
} from './providerDefaults';
import { ProviderType } from '../types/provider';

describe('providerDefaults helpers', () => {
  it('returns canonical API endpoints for known providers', () => {
    expect(getProviderDefaultUrl(ProviderType.ZHIPU)).toBe('https://open.bigmodel.cn/api/anthropic');
    expect(getProviderDefaultUrl(ProviderType.DEEPSEEK)).toBe('https://api.deepseek.com/anthropic');
  });

  it('exposes model mappings for vendors that require remapping', () => {
    const zhipuMapping = getProviderDefaultMapping(ProviderType.ZHIPU);
    expect(zhipuMapping).toEqual({ 'claude-*': 'glm-4.7' });

    const minimaxMapping = getProviderDefaultMapping(ProviderType.MINIMAX);
    expect(minimaxMapping).toEqual({ 'claude-*': 'minimax-m2.1' });
  });

  it('provides official site links to drive favicon fetching', () => {
    expect(getProviderDefaultOfficialSite(ProviderType.OPENAI)).toBe('https://platform.openai.com');
    expect(getProviderDefaultOfficialSite(ProviderType.ANTHROPIC)).toBe('https://console.anthropic.com');
  });

  it('falls back to undefined for unknown providers', () => {
    expect(getProviderDefaultUrl(ProviderType.UNKNOWN)).toBeUndefined();
    expect(getProviderDefaultMapping(ProviderType.UNKNOWN)).toBeUndefined();
    expect(getProviderDefaultOfficialSite(ProviderType.UNKNOWN)).toBeUndefined();
  });

  it('keeps exported constant in sync with helper outputs', () => {
    expect(PROVIDER_DEFAULTS[ProviderType.DEEPSEEK]?.officialSite).toBe(
      getProviderDefaultOfficialSite(ProviderType.DEEPSEEK)
    );
  });
});
