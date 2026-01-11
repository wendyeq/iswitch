/**
 * [INPUT]: source: iswitch-tauri/src/utils/faviconFetcher.ts ([POS]: Test Suite)
 * [OUTPUT]: Favicon 获取逻辑单元测试
 * [PROTOCOL]: FractalFlow v1.0
 * [POS]: iswitch-tauri/src/utils/faviconFetcher.test.ts
 */

import { describe, it, expect, vi } from 'vitest';
import { getFaviconUrl, getFaviconUrlFallback } from './faviconFetcher';

describe('faviconFetcher', () => {
  it('generates correct Unavatar favicon URL for standard domains', () => {
    const url = 'https://openai.com';
    const result = getFaviconUrl(url);
    expect(result).toBe('https://unavatar.io/openai.com');
  });

  it('strips "api." prefix for smarter fallback', () => {
    const url = 'https://api.minimaxi.com/anthropic';
    const result = getFaviconUrl(url);
    // Should strip api. -> minimaxi.com
    expect(result).toBe('https://unavatar.io/minimaxi.com');
  });

  it('strips "api." prefix for other domains', () => {
    const url = 'https://api.openai.com/v1/chat';
    const result = getFaviconUrl(url);
    expect(result).toBe('https://unavatar.io/openai.com');
  });

  it('handles http URLs', () => {
    const url = 'http://example.com';
    const result = getFaviconUrl(url);
    expect(result).toBe('https://unavatar.io/example.com');
  });

  it('handles empty input', () => {
    const result = getFaviconUrl('');
    expect(result).toBe('');
  });

  it('returns formatted URL for potential domains', () => {
    // "not-a-url" becomes "https://not-a-url", which is technically a valid URL structure
    const result = getFaviconUrl('not-a-url');
    expect(result).toContain('https://unavatar.io/not-a-url');
  });

  it('returns empty string for malformed URLs and warns once', () => {
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const result = getFaviconUrl('http:// ');
    expect(result).toBe('');
    expect(warnSpy).toHaveBeenCalledWith('[FaviconFetcher] Invalid URL:', 'http:// ');
    warnSpy.mockRestore();
  });

  it('fallback helper mirrors duckduckgo formatter', () => {
    expect(getFaviconUrlFallback('claude.ai')).toBe('https://icons.duckduckgo.com/ip3/claude.ai.ico');
    expect(getFaviconUrlFallback('https://sub.example.dev')).toBe(
      'https://icons.duckduckgo.com/ip3/sub.example.dev.ico'
    );
  });

  it('fallback helper returns empty string when parsing fails', () => {
    expect(getFaviconUrlFallback('://bad')).toBe('');
  });
});
