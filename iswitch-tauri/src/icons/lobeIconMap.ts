/**
 * ---
 * [INPUT]: {@lobehub/icons-static-svg, fallbackLobeIcons}
 *     - LobeHub Icons: source: node_modules/@lobehub/icons-static-svg ([POS]: 官方图标库)
 *     - fallback: source: ./fallbackLobeIcons.ts ([POS]: 回退图标)
 * [OUTPUT]: {getIcon 函数} - 根据 Provider 名称获取对应图标
 * [POS]: 图标映射服务，提供 Provider 到 LobeHub Icon 的映射和回退机制
 * [PROTOCOL]:
 * 1. 优先使用官方图标
 * 2. 回退到自定义图标
 * 3. 名称匹配不区分大小写
 * ---
 */
import fallbackIcons from './fallbackLobeIcons';

const globIcons = import.meta.glob('../../node_modules/@lobehub/icons-static-svg/icons/*.svg', {
  eager: true,
  import: 'default',
  query: '?raw',
}) as Record<string, string>;

const normalize = (source: Record<string, string>) => {
  return Object.entries(source).reduce<Record<string, string>>((acc, [key, value]) => {
    const name = key.split('/').pop()?.replace('.svg', '')?.toLowerCase();
    if (name) {
      acc[name] = value;
    }
    return acc;
  }, {});
};

const normalizedFallback = Object.keys(fallbackIcons).reduce<Record<string, string>>((acc, key) => {
  acc[key.toLowerCase()] = fallbackIcons[key];
  return acc;
}, {});

const normalizedGlob = normalize(globIcons);

const lobeIconMap: Record<string, string> = {
  ...normalizedFallback,
  ...normalizedGlob,
};

export default lobeIconMap;
