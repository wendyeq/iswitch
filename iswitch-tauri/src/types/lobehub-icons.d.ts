/**
 * ---
 * [INPUT]:
 *   - @lobehub/icons-static-svg: source: node_modules/@lobehub/icons-static-svg ([POS]: 第三方图标库)
 * [OUTPUT]: LobeHub 图标模块类型声明
 * [POS]: TypeScript 模块声明，为第三方图标库提供类型支持
 * [PROTOCOL]: FractalFlow v1.0 - 分形自洽
 * ---
 */

declare module '@lobehub/icons-static-svg' {
  const icons: Record<string, string>;
  export default icons;
  export { icons };
}
