/**
 * [INPUT]: source: iswitch-tauri/src/utils/providerDetector.ts ([POS]: Test Suite)
 * [OUTPUT]: 供应商自动检测单元测试
 * [PROTOCOL]: FractalFlow v1.0
 * [POS]: iswitch-tauri/src/utils/providerDetector.test.ts
 */

import { describe, it, expect } from 'vitest';
import { detectProvider } from './providerDetector';
import { ProviderType } from '../types/provider';

describe('providerDetector', () => {
  it('detects OpenAI logic from path signature', () => {
    const url = 'https://api.openai.com/v1/chat/completions';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.OPENAI);
    expect(result.confidence).toBe(1.0);
    expect(result.officialSite).toBe('https://platform.openai.com');
  });

  it('detects MiniMax from domain', () => {
    const url = 'https://api.minimaxi.com/anthropic';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.MINIMAX);
    expect(result.name).toBe('MiniMax');
    expect(result.confidence).toBe(1.0);
  });

  it('detects Anthropic from domain', () => {
    const url = 'https://api.anthropic.com/v1/something';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.ANTHROPIC);
    expect(result.name).toBe('Anthropic');
  });

  it('detects Azure OpenAI from domain', () => {
    const url = 'https://my-resource.openai.azure.com/deployments';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.AZURE);
  });

  it('detects Google Gemini from domain', () => {
    const url = 'https://generativelanguage.googleapis.com';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.GOOGLE);
  });

  it('detects Ollama from local pattern', () => {
    const url = 'http://localhost:11434/api/chat';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.OLLAMA);
  });

  it('returns UNKNOWN for random URL', () => {
    const url = 'https://example.com/api';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.UNKNOWN);
    expect(result.isAmbiguous).toBe(true);
    expect(result.confidence).toBeLessThan(0.2);
  });

  it('detects context conflict (Auto-Correction)', () => {
    // User is in "Claude" tab but pastes an OpenAI URL
    const url = 'https://api.openai.com/v1/chat/completions';
    const contextTab = 'Claude Code'; // Contains "Claude"

    const result = detectProvider(url, contextTab);

    expect(result.type).toBe(ProviderType.OPENAI); // Should still identify as OpenAI
    expect(result.isAutoCorrected).toBe(true); // But flag as auto-corrected
  });

  it('detects context match (No Correction)', () => {
    // User is in "Claude" tab and pastes Anthropic URL
    const url = 'https://api.anthropic.com';
    const contextTab = 'Claude Code';

    const result = detectProvider(url, contextTab);

    expect(result.type).toBe(ProviderType.ANTHROPIC);
    expect(result.isAutoCorrected).toBe(false); // No correction needed
  });

  it('correctly distinguishes DeepSeek from Anthropic when URL is ambiguous', () => {
    // Scenario from user report: https://api.deepseek.com/anthropic
    // Should be DeepSeek because of the domain, despite "anthropic" in path
    const url = 'https://api.deepseek.com/anthropic';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.DEEPSEEK);
    expect(result.confidence).toBe(1.0);
  });

  it('detects Zhipu AI from domain', () => {
    const url = 'https://open.bigmodel.cn/api/paas/v4';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.ZHIPU);
    expect(result.confidence).toBe(1.0);
  });

  it('detects Zhipu AI from keyword', () => {
    const input = 'zhipu';
    const result = detectProvider(input);
    expect(result.type).toBe(ProviderType.ZHIPU);
  });

  it('detects Zhipu AI from Chinese keyword', () => {
    const input = '智谱';
    const result = detectProvider(input);
    expect(result.type).toBe(ProviderType.ZHIPU);
  });
  it('detects Azure OpenAI from cognitive services domain', () => {
    // Scenario from user report: https://example.cognitiveservices.azure.com/openai/v1
    const url = 'https://example.cognitiveservices.azure.com/openai/v1';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.AZURE);
    // [Expectation]: Should use official site (portal.azure.com) for reliable favicon
    expect(result.icon).toContain('portal.azure.com');
  });

  it('prioritizes Azure domain over Deepseek path signature', () => {
    // Azure URL containing /chat/completions which matches Deepseek path signature
    const url = 'https://my-resource.openai.azure.com/openai/deployments/gpt4/chat/completions?api-version=2024';
    const result = detectProvider(url);
    expect(result.type).toBe(ProviderType.AZURE);
  });
});
