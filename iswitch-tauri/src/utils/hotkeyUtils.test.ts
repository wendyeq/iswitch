/**
 * ---
 * [INPUT]: {hotkeyUtils}
 *     - hotkeyUtils: source: ./hotkeyUtils.ts ([POS]: 快捷键解析工具)
 * [OUTPUT]: {测试结果} - hotkeyUtils 单元测试
 * [POS]: 快捷键解析函数测试
 * [PROTOCOL]:
 * 1. 测试快捷键解析
 * 2. 测试格式化输出
 * ---
 */
import { describe, it, expect } from 'vitest';
import {
  parseHotkeyString,
  parseShortcutToHotkey,
  formatHotkeyString,
  formatHotkeyStringmac,
  MODIFIERS,
} from './hotkeyUtils';

describe('hotkeyUtils', () => {
  describe('MODIFIERS 常量', () => {
    it('包含正确的修饰键值', () => {
      expect(MODIFIERS.CMD).toBe(256);
      expect(MODIFIERS.SHIFT).toBe(512);
      expect(MODIFIERS.ALT).toBe(1024);
      expect(MODIFIERS.CTRL).toBe(2048);
    });
  });

  describe('parseHotkeyString', () => {
    it('解析 Control+A 返回键码和修饰符', () => {
      const result = parseHotkeyString('Control+A');
      expect(result).not.toBeNull();
      expect(result?.key).toBe(65); // A 的 ASCII 码
      expect(result?.modifier).toBe(256); // Control
    });

    it('解析 Shift+S 返回键码和修饰符', () => {
      const result = parseHotkeyString('Shift+S');
      expect(result).not.toBeNull();
      expect(result?.key).toBe(83); // S 的 ASCII 码
      expect(result?.modifier).toBe(1024); // Shift
    });

    it('不包含 + 分隔符时返回 null', () => {
      const result = parseHotkeyString('F1');
      expect(result).toBeNull();
    });

    it('解析空字符串返回 null', () => {
      const result = parseHotkeyString('');
      expect(result).toBeNull();
    });

    it('解析多个修饰键', () => {
      const result = parseHotkeyString('Control+Shift+K');
      expect(result).not.toBeNull();
      expect(result?.key).toBe(75); // K 的 ASCII 码
      // Control (256) + Shift (1024) = 1280
      expect(result?.modifier).toBe(256 + 1024);
    });
  });

  describe('formatHotkeyString', () => {
    it('格式化 keycode 和 modifiers 返回字符串', () => {
      const result = formatHotkeyString(0, 0);
      expect(typeof result).toBe('string');
    });

    it('格式化有修饰符的键', () => {
      const result = formatHotkeyString(0x00, 256); // Command + A
      expect(typeof result).toBe('string');
      expect(result.length).toBeGreaterThan(0);
    });

    it('未知键码时抛出错误', () => {
      expect(() => formatHotkeyString(999, 0)).toThrowError(/Unknown keycode/);
    });
  });

  describe('parseShortcutToHotkey', () => {
    it('支持多种修饰符写法', () => {
      const result = parseShortcutToHotkey('cmd+alt+K');
      expect(result.modifier).toBe(MODIFIERS.CMD + MODIFIERS.ALT);
      expect(result.key).toBeDefined();
    });

    it('支持 control/option/mixed 写法', () => {
      const result = parseShortcutToHotkey('control+option+V');
      expect(result.modifier).toBe(MODIFIERS.CTRL + MODIFIERS.ALT);
      expect(result.key).toBeDefined();
    });
  });

  describe('formatHotkeyStringmac', () => {
    it('格式化为 macOS 风格快捷键字符串', () => {
      const result = formatHotkeyStringmac(0, 256); // Command + A
      expect(typeof result).toBe('string');
    });

    it('格式化无修饰符的键', () => {
      const result = formatHotkeyStringmac(0, 0); // A
      expect(typeof result).toBe('string');
    });

    it('格式化包含多个修饰符的键', () => {
      const result = formatHotkeyStringmac(40, MODIFIERS.CMD | MODIFIERS.SHIFT | MODIFIERS.ALT);
      expect(result).toBe('Cmd+Shift+Alt+K');
    });
  });
});
