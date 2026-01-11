/**
 * [INPUT]: source: iswitch-tauri/src/utils/.folder.md ([POS]: Utils)
 * [OUTPUT]: Favicon URL (支持智能域名降级)
 * [PROTOCOL]: FractalFlow v1.0
 * [POS]: iswitch-tauri/src/utils/faviconFetcher.ts
 */

export const getFaviconUrl = (url: string): string => {
  if (!url) return '';

  try {
    // 尝试添加协议如果缺失
    const urlToParse = url.startsWith('http') ? url : `https://${url}`;
    const urlObj = new URL(urlToParse);
    let hostname = urlObj.hostname;

    // [Logic]: 智能降级 - 如果是 api. 开头的域名，通常没有 favicon，尝试去掉 api. 前缀
    // 例如: api.minimaxi.com -> minimaxi.com
    if (hostname.startsWith('api.')) {
      hostname = hostname.replace(/^api\./, '');
    }

    // 优先尝试 Google (质量最好)
    // [Fallback Strategy]: 如果 Google 不可用 (如国内环境)，可以替换为:
    // 1. DuckDuckGo: `https://icons.duckduckgo.com/ip3/${hostname}.ico`
    // 2. Unavatar: `https://unavatar.io/${hostname}`
    // 3. 官方默认: `https://${hostname}/favicon.ico`

    // [Active]: 使用 Unavatar (支持多源回退，且国内访问较好)
    return `https://unavatar.io/${hostname}`;

    // [Inactive]: Google Source
    // return `https://www.google.com/s2/favicons?domain=${hostname}&sz=64`;
  } catch (e) {
    console.warn('[FaviconFetcher] Invalid URL:', url);
    return '';
  }
};

/**
 * [Optional]: 获取备用源 (国内友好)
 */
export const getFaviconUrlFallback = (url: string): string => {
  try {
    const urlToParse = url.startsWith('http') ? url : `https://${url}`;
    const hostname = new URL(urlToParse).hostname;
    return `https://icons.duckduckgo.com/ip3/${hostname}.ico`;
  } catch {
    return '';
  }
};
