/**
 * ---
 * [INPUT]: {Vite 配置}
 *     - vite.config.ts: source: ./vite.config.ts ([POS]: Vite 构建配置)
 * [OUTPUT]: {Vitest 配置} - 测试运行器配置
 * [POS]: Vitest 测试配置，定义测试环境、覆盖率和全局设置
 * [PROTOCOL]:
 * 1. 使用 jsdom 模拟浏览器环境
 * 2. 配置 Tauri API Mock
 * 3. 启用覆盖率报告
 * ---
 */
import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
    plugins: [vue()],
    test: {
        // 测试环境配置
        environment: 'jsdom',

        // 全局设置文件
        setupFiles: ['./src/test/setup.ts'],

        // 测试文件匹配模式
        include: ['src/**/*.{test,spec}.{js,ts,vue}'],
        exclude: ['node_modules', 'dist', 'src-tauri'],

        // 全局 API (describe, it, expect 等无需导入)
        globals: true,

        // 覆盖率配置
        coverage: {
            provider: 'v8',
            reporter: ['text', 'json', 'html'],
            reportsDirectory: '../reports/coverage/frontend',
            include: ['src/**/*.{ts,vue}'],
            exclude: [
                'src/**/*.test.ts',
                'src/**/*.spec.ts',
                'src/test/**',
                'src/main.ts',
                'src/vite-env.d.ts',
            ],
            // 覆盖率阈值 (可选，后续启用)
            // thresholds: {
            //   lines: 60,
            //   branches: 60,
            //   functions: 60,
            //   statements: 60,
            // },
        },

        // 测试超时时间
        testTimeout: 10000,

        // CSS 处理
        css: true,
    },

    // 路径别名 (与 vite.config.ts 保持一致)
    resolve: {
        alias: {
            '@': resolve(__dirname, 'src'),
        },
    },
})
