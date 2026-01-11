/**
 * ---
 * [INPUT]: {toast utility}
 *     - toast: source: ./toast.ts ([POS]: 通知工具)
 * [OUTPUT]: {Toast 测试} - 验证容器复用、样式与生命周期
 * [POS]: iswitch-tauri/src/utils/toast.test.ts
 * [PROTOCOL]:
 * 1. 模拟 requestAnimationFrame
 * 2. 使用假定时器触发自动销毁
 * 3. 确保空消息被忽略
 * ---
 */
import { describe, it, expect, beforeAll, beforeEach, afterEach, vi } from 'vitest';
import { showToast } from './toast';

let rafSpy: ReturnType<typeof vi.spyOn> | undefined;

describe('toast util', () => {
  beforeAll(() => {
    if (!window.requestAnimationFrame) {
      window.requestAnimationFrame = ((cb: FrameRequestCallback) => {
        cb(0);
        return 0;
      }) as typeof window.requestAnimationFrame;
    }
  });

  beforeEach(() => {
    document.body.innerHTML = '';
    vi.useFakeTimers();
    rafSpy = vi.spyOn(window, 'requestAnimationFrame').mockImplementation(cb => {
      cb(0);
      return 0;
    });
  });

  afterEach(() => {
    rafSpy?.mockRestore();
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  it('ignores empty messages', () => {
    showToast('');
    expect(document.querySelector('.mac-toast-container')).toBeNull();
  });

  it('creates macOS style toast and disposes it after animation', () => {
    showToast('Hello iSwitch');

    const container = document.querySelector('.mac-toast-container') as HTMLElement;
    expect(container).toBeTruthy();
    expect(container.childElementCount).toBe(1);

    const toast = container.firstElementChild as HTMLElement;
    expect(toast.className).toContain('mac-toast mac-toast-success');
    expect(toast.classList.contains('mac-toast-visible')).toBe(true);
    expect(toast.textContent).toBe('Hello iSwitch');

    vi.runAllTimers();
    toast.dispatchEvent(new Event('transitionend'));

    expect(document.querySelector('.mac-toast-container')).toBeNull();
  });

  it('applies error variant classes', () => {
    showToast('Failed', 'error');

    const toast = document.querySelector('.mac-toast') as HTMLElement;
    expect(toast).toBeTruthy();
    expect(toast.classList.contains('mac-toast-error')).toBe(true);
  });
});
