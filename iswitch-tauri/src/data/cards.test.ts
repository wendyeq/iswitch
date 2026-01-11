/**
 * ---
 * [INPUT]: {automationCardGroups, createAutomationCards}
 *     - automationCardGroups: source: ./cards.ts ([POS]: 默认卡片配置)
 *     - createAutomationCards: source: ./cards.ts ([POS]: 工具函数)
 * [OUTPUT]: {静态数据测试} - 验证卡片配置与补全逻辑
 * [POS]: iswitch-tauri/src/data/cards.test.ts
 * [PROTOCOL]:
 * 1. 默认配置不可变
 * 2. 缺省字段使用安全默认值
 * ---
 */
import { describe, expect, it } from 'vitest';
import { automationCardGroups, createAutomationCards, type AutomationCard } from './cards';

describe('automation cards data helpers', () => {
  it('exposes claude and codex card groups with stable structure', () => {
    expect(Object.keys(automationCardGroups)).toEqual(['claude', 'codex']);
    expect(automationCardGroups.claude).toHaveLength(4);
    expect(automationCardGroups.codex).toHaveLength(1);
    automationCardGroups.claude.forEach(card => {
      expect(card).toMatchObject({
        id: expect.any(Number),
        name: expect.any(String),
        apiUrl: expect.stringContaining('http'),
        icon: expect.any(String),
        enabled: expect.any(Boolean),
      });
    });
  });

  it('fills missing officialSite field when creating cards', () => {
    const payload: AutomationCard[] = [
      {
        id: 1,
        name: 'Custom Provider',
        apiUrl: 'https://provider.dev',
        apiKey: 'secret',
        officialSite: '',
        icon: 'custom',
        tint: '#fff',
        accent: '#000',
        enabled: true,
      },
      {
        id: 2,
        name: 'Fallback Provider',
        apiUrl: 'https://fallback.dev',
        apiKey: 'secret',
        officialSite: undefined as unknown as string,
        icon: 'fallback',
        tint: '#eee',
        accent: '#111',
        enabled: false,
      },
    ];

    const cards = createAutomationCards(payload);
    expect(cards).toHaveLength(2);
    expect(cards[0].officialSite).toBe('');
    expect(cards[1].officialSite).toBe('');
    expect(payload[1].officialSite).toBeUndefined();
  });
});
