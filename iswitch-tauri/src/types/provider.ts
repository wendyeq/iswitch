/**
 * [INPUT]: source: openspec/changes/redesign-vendor-form/design.md ([POS]: Types)
 * [OUTPUT]: 供应商相关类型定义
 * [PROTOCOL]: FractalFlow v1.0
 * [POS]: iswitch-tauri/src/types/provider.ts
 */

export enum ProviderType {
  OPENAI = 'openai',
  ANTHROPIC = 'anthropic',
  AZURE = 'azure',
  GOOGLE = 'google',
  OLLAMA = 'ollama',
  GEMINI = 'gemini',
  GROK = 'grok',
  DEEPSEEK = 'deepseek',
  MINIMAX = 'minimax',
  ZHIPU = 'zhipu',
  UNKNOWN = 'unknown',
}

export interface ProviderInfo {
  id: ProviderType;
  name: string;
  icon: string;
}

export interface DetectionResult {
  type: ProviderType;
  name: string;
  icon?: string;
  officialSite?: string; // New field for smart link
  confidence: number; // 0-1
  isAmbiguous?: boolean;
  isAutoCorrected?: boolean;
  originalUrl?: string;
  correctedUrl?: string; // 如果有自动纠正则返回
}
