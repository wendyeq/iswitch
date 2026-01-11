/**
 * [INPUT]: source: openspec/changes/redesign-vendor-form/design.md ([POS]: Defaults)
 * [OUTPUT]: 供应商默认配置
 * [PROTOCOL]: FractalFlow v1.0
 * [POS]: iswitch-tauri/src/utils/providerDefaults.ts
 */

import { ProviderType } from '../types/provider';

interface ProviderDefault {
  apiUrl: string;
  officialSite?: string; // Optional override or useful for UI
  modelMapping?: Record<string, string>;
}

export const PROVIDER_DEFAULTS: Partial<Record<ProviderType, ProviderDefault>> = {
  [ProviderType.ZHIPU]: {
    apiUrl: 'https://open.bigmodel.cn/api/anthropic',
    officialSite: 'https://www.bigmodel.cn/usercenter/glm-coding/usage',
    modelMapping: {
      'claude-*': 'glm-4.7',
    },
  },
  [ProviderType.DEEPSEEK]: {
    apiUrl: 'https://api.deepseek.com/anthropic',
    officialSite: 'https://platform.deepseek.com',
    modelMapping: {
      'claude-*': 'deepseek-chat',
    },
  },
  [ProviderType.MINIMAX]: {
    apiUrl: 'https://api.minimaxi.com/anthropic',
    officialSite: 'https://platform.minimaxi.com/user-center/payment/coding-plan',
    modelMapping: {
      'claude-*': 'minimax-m2.1',
    },
  },
  [ProviderType.OPENAI]: {
    apiUrl: 'https://api.openai.com/v1',
    officialSite: 'https://platform.openai.com',
  },
  [ProviderType.ANTHROPIC]: {
    apiUrl: 'https://api.anthropic.com/v1',
    officialSite: 'https://console.anthropic.com',
  },
};

export const getProviderDefaultUrl = (type: ProviderType): string | undefined => {
  return PROVIDER_DEFAULTS[type]?.apiUrl;
};

export const getProviderDefaultMapping = (type: ProviderType): Record<string, string> | undefined => {
  return PROVIDER_DEFAULTS[type]?.modelMapping;
};

export const getProviderDefaultOfficialSite = (type: ProviderType): string | undefined => {
  return PROVIDER_DEFAULTS[type]?.officialSite;
};
