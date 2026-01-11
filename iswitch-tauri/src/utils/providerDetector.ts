/**
 * [INPUT]: source: openspec/changes/redesign-vendor-form/design.md ([POS]: Intelligence Logic)
 * [OUTPUT]: 供应商类型识别结果
 * [PROTOCOL]: FractalFlow v1.0
 * [POS]: iswitch-tauri/src/utils/providerDetector.ts
 */

import { ProviderType, DetectionResult } from '../types/provider';
import { getFaviconUrl } from './faviconFetcher';

interface ProviderPattern {
  type: ProviderType;
  name: string;
  patterns: RegExp[];
  pathSignatures: string[];
  officialSite: string;
}

const PROVIDERS: ProviderPattern[] = [
  {
    type: ProviderType.AZURE,
    name: 'Azure OpenAI',
    patterns: [/openai\.azure\.com/, /cognitiveservices\.azure\.com/, /azure/i],
    pathSignatures: [],
    officialSite: 'https://portal.azure.com',
  },
  {
    type: ProviderType.OPENAI,
    name: 'OpenAI',
    patterns: [/api\.openai\.com/, /openai/i],
    pathSignatures: ['/v1/chat/completions', '/v1/models'],
    officialSite: 'https://platform.openai.com',
  },
  {
    type: ProviderType.DEEPSEEK,
    name: 'Deepseek',
    patterns: [/api\.deepseek\.com/, /deepseek/i],
    pathSignatures: ['/chat/completions'],
    officialSite: 'https://platform.deepseek.com',
  },
  {
    type: ProviderType.ANTHROPIC,
    name: 'Anthropic',
    patterns: [/api\.anthropic\.com/, /anthropic/i, /claude/i],
    pathSignatures: ['/v1/messages', 'anthropic-version'],
    officialSite: 'https://console.anthropic.com',
  },
  {
    type: ProviderType.GOOGLE,
    name: 'Google Gemini',
    patterns: [/generativelanguage\.googleapis\.com/, /google/i, /gemini/i],
    pathSignatures: ['/v1beta/models', 'goog'],
    officialSite: 'https://aistudio.google.com',
  },
  {
    type: ProviderType.OLLAMA,
    name: 'Ollama',
    patterns: [/localhost:11434/, /ollama/i],
    pathSignatures: ['/api/chat'],
    officialSite: 'http://localhost:11434',
  },
  {
    type: ProviderType.MINIMAX,
    name: 'MiniMax',
    patterns: [/api\.minimaxi\.com/, /minimax/i],
    pathSignatures: ['/v1/text/chatcompletion_v2'],
    officialSite: 'https://platform.minimaxi.com/user-center/payment/coding-plan',
  },
  {
    type: ProviderType.ZHIPU,
    name: 'Zhipu AI',
    patterns: [/open\.bigmodel\.cn/, /zhipu/i, /bigmodel/i, /智谱/, /xn--usb765i/], // xn--usb765i is "智谱"
    pathSignatures: ['/api/paas/v4'],
    officialSite: 'https://www.bigmodel.cn/usercenter/glm-coding/usage',
  },
];

export const detectProvider = (url: string, contextTab?: string): DetectionResult => {
  let bestMatch: ProviderPattern | undefined;
  let confidence = 0;
  let isAutoCorrected = false;
  let correctedUrl: string | undefined;

  // 0. Pre-process URL
  const trimmedUrl = url.trim();

  // 1. Domain/Host Pattern Detection (High Priority)
  // We prioritize hostname matching over generic pattern matching to avoid path confusion
  // Optimization: If the input looks like a simple keyword (no dot, no slash, short),
  // skip expensive URL parsing and fall through to generic regex.
  const isSimpleKeyword = !trimmedUrl.includes('.') && !trimmedUrl.includes('/') && trimmedUrl.length < 50;

  if (!isSimpleKeyword) {
    try {
      // Create a URL object to extract hostname safely
      // If url doesn't start with http/https, we might need to prepend it for parsing,
      // but usually pasting a full URL includes protocol.
      // If not, we fall back to generic regex.
      const urlObj = new URL(trimmedUrl.startsWith('http') ? trimmedUrl : `https://${trimmedUrl}`);
      const hostname = urlObj.hostname;

      for (const provider of PROVIDERS) {
        // Check if any pattern matches the HOSTNAME specifically
        if (provider.patterns.some(p => p.test(hostname))) {
          bestMatch = provider;
          confidence = 1.0; // Higher than generic pattern match
          break;
        }
      }
    } catch (e) {
      // URL parsing failed, proceed to generic regex
    }
  }

  // 2. Path Signature Detection (Medium Priority)
  // Only if no domain match found
  if (!bestMatch) {
    for (const provider of PROVIDERS) {
      if (provider.pathSignatures.some(sig => trimmedUrl.includes(sig))) {
        bestMatch = provider;
        confidence = 0.9;
        break;
      }
    }
  }

  // 3. Generic Pattern Detection (Fallback)
  if (!bestMatch) {
    for (const provider of PROVIDERS) {
      if (provider.patterns.some(p => p.test(trimmedUrl))) {
        bestMatch = provider;
        confidence = 0.7;
        break;
      }
    }
  }

  // 4. Attempt to infer from Context if URL is ambiguous but not empty
  if (!bestMatch && trimmedUrl.length > 5 && contextTab) {
    const lowerContext = contextTab.toLowerCase();
    // Heuristic: if tab is 'claude' and URL is vaguely like an API key or partial URL
    // We don't have enough info here really without regex, but let's assume if it looks like a key starting with 'sk-' and context is openai
    // For now, simpler context hinting
    if (lowerContext.includes('claude') && trimmedUrl.includes('anthropic')) {
      // hint logic could go here
    }
  }

  // 5. Handle "localhost" without port -> Ollama? -> No, dangerous assumption.

  if (bestMatch) {
    // [Logic]: Use official site for favicon if available (Better Quality/Reliability)
    // Fallback to user URL if no official site defined
    const iconSource = bestMatch.officialSite || trimmedUrl;
    const icon = getFaviconUrl(iconSource);

    // Check for potential conflicts with context
    if (contextTab) {
      const lowerContext = contextTab.toLowerCase();
      // Simple mapping from context string to ProviderType
      // This relies on tab names containing the provider name
      if (
        (lowerContext.includes('claude') && bestMatch.type !== ProviderType.ANTHROPIC) ||
        (lowerContext.includes('openai') && bestMatch.type !== ProviderType.OPENAI) ||
        (lowerContext.includes('google') && bestMatch.type !== ProviderType.GOOGLE) ||
        (lowerContext.includes('gemini') && bestMatch.type !== ProviderType.GOOGLE) ||
        (lowerContext.includes('deepseek') && bestMatch.type !== ProviderType.DEEPSEEK)
      ) {
        isAutoCorrected = true;
      }
    }

    return {
      type: bestMatch.type,
      name: bestMatch.name,
      icon,
      officialSite: bestMatch.officialSite,
      confidence,
      isAmbiguous: false,
      isAutoCorrected,
      correctedUrl,
      originalUrl: trimmedUrl,
    };
  }

  return {
    type: ProviderType.UNKNOWN,
    name: 'Unknown Provider',
    icon: getFaviconUrl(trimmedUrl),
    confidence: 0.1,
    isAmbiguous: true, // Mark as ambiguous to trigger UI manual entry
    originalUrl: trimmedUrl,
  };
};
