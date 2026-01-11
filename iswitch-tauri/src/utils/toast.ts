/**
 * ---
 * [INPUT]: {message, type}
 * [OUTPUT]: {Toast UI} - macOS 风格的通知提示
 * [POS]: 通知提示工具，提供统一的消息反馈接口
 * [PROTOCOL]:
 * 1. 使用 macOS 原生风格 UI
 * 2. 自动管理容器和生命周期
 * 3. 支持 success/error 类型
 * ---
 */
type ToastType = 'success' | 'error';

const TOAST_DURATION = 2400;

let toastContainer: HTMLElement | null = null;

function getContainer() {
  if (toastContainer) return toastContainer;

  toastContainer = document.createElement('div');
  toastContainer.className = 'mac-toast-container';
  document.body.appendChild(toastContainer);
  return toastContainer;
}

export function showToast(message: string, type: ToastType = 'success') {
  if (!message) return;

  const container = getContainer();
  const toast = document.createElement('div');
  toast.className = `mac-toast mac-toast-${type}`;
  toast.textContent = message;

  container.appendChild(toast);

  requestAnimationFrame(() => {
    toast.classList.add('mac-toast-visible');
  });

  const remove = () => {
    toast.classList.remove('mac-toast-visible');
    toast.classList.add('mac-toast-hide');
    const handler = () => {
      toast.removeEventListener('transitionend', handler);
      toast.remove();
      if (toastContainer && toastContainer.childElementCount === 0) {
        toastContainer.remove();
        toastContainer = null;
      }
    };
    toast.addEventListener('transitionend', handler);
  };

  setTimeout(remove, TOAST_DURATION);
}
