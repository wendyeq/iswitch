<!--
[INPUT]: 
  - source: openspec/specs/capsule-nav/spec.md ([POS]: 胶囊导航功能规范)
[OUTPUT]: 胶囊导航组件 - 简化版 (3 个核心按钮: 日志、主题、设置)
[PROTOCOL]: FractalFlow v1.0
[POS]: iswitch-tauri/src/components/Main/CapsuleNavigation.vue - 主页面胶囊导航组件
-->
<template>
  <nav class="capsule-navigation" role="navigation" :aria-label="t('components.main.capsule.ariaLabel')">
    <!-- === Context Controls Section === -->

    <!-- Tab Switcher: Claude / Codex -->
    <div class="nav-segment">
      <button
        class="capsule-tab"
        :class="{ active: viewMode === 'claude' }"
        @click="$emit('update:viewMode', 'claude')"
      >
        {{ t('components.main.tabs.claude') }}
      </button>
      <button class="capsule-tab" :class="{ active: viewMode === 'codex' }" @click="$emit('update:viewMode', 'codex')">
        {{ t('components.main.tabs.codex') }}
      </button>
    </div>

    <!-- Proxy Toggle -->
    <button
      class="capsule-item"
      :class="{ 'is-active': proxyEnabled, 'is-busy': proxyLoading }"
      :data-tooltip="
        proxyEnabled
          ? t('components.main.relayToggle.tooltip') + ' (On)'
          : t('components.main.relayToggle.tooltip') + ' (Off)'
      "
      @click="$emit('toggleProxy')"
      :disabled="proxyLoading"
    >
      <svg v-if="proxyLoading" class="animate-spin" viewBox="0 0 24 24">
        <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none" class="opacity-25" />
        <path fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" class="opacity-75" />
      </svg>
      <svg v-else viewBox="0 0 24 24" aria-hidden="true" :class="{ 'text-accent': proxyEnabled }">
        <path
          d="M13 10V3L4 14h7v7l9-11h-7z"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
          stroke-linejoin="round"
          :fill="proxyEnabled ? 'currentColor' : 'none'"
        />
      </svg>
    </button>

    <!-- Add Provider Button -->
    <button class="capsule-item" :data-tooltip="t('components.main.tabs.addCard')" @click="$emit('add')">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M12 5v14M5 12h14"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
          stroke-linejoin="round"
          fill="none"
        />
      </svg>
    </button>

    <!-- === Global Actions Section === -->

    <!-- 日志: 列表/终端图标 — 监控 -->
    <button class="capsule-item" :data-tooltip="t('components.main.logs.view')" @click="goToLogs">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M5 7h14M5 12h14M5 17h9"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
          stroke-linejoin="round"
          fill="none"
        />
      </svg>
    </button>

    <!-- [简化] Skill 和 MCP 按钮已移除 (simplify-ui-controls) -->
    <!-- 用户可通过设置页面访问这些功能 -->

    <!-- 切换主题: 太阳/月亮图标 — 视觉控制 -->
    <button class="capsule-item" :data-tooltip="t('components.main.controls.theme')" @click="toggleTheme">
      <svg v-if="themeIcon === 'sun'" viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="1.2" fill="none" />
        <path
          d="M12 3v2m0 14v2m9-9h-2M5 12H3m14.95 6.95-1.41-1.41M7.46 7.46 6.05 6.05m12.9 0-1.41 1.41M7.46 16.54l-1.41 1.41"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
        />
      </svg>
      <svg v-else viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M21 12.79A9 9 0 1111.21 3a7 7 0 109.79 9.79z"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>

    <!-- 设置: 齿轮图标 — 全局配置 -->
    <button class="capsule-item" :data-tooltip="t('components.main.controls.settings')" @click="goToSettings">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M12 15a3 3 0 100-6 3 3 0 000 6z"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
          stroke-linejoin="round"
          fill="none"
        />
        <path
          d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09a1.65 1.65 0 00-1-1.51 1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09a1.65 1.65 0 001.51-1 1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
          stroke-linejoin="round"
          fill="none"
        />
      </svg>
    </button>
  </nav>
</template>

<script setup lang="ts">
/**
 * [INPUT]: source: openspec/specs/capsule-nav/spec.md ([POS]: 胶囊导航功能规范)
 * [OUTPUT]: 胶囊导航响应式组件逻辑 (简化版: 3 按钮)
 */
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { getCurrentTheme, setTheme, type ThemeMode } from '../../utils/ThemeManager';

const { t } = useI18n();
const router = useRouter();

// Props definition
interface Props {
  viewMode?: 'claude' | 'codex';
  proxyEnabled?: boolean;
  proxyLoading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  viewMode: 'claude',
  proxyEnabled: false,
  proxyLoading: false,
});

// Emits definition
const emit = defineEmits<{
  (e: 'update:viewMode', mode: 'claude' | 'codex'): void;
  (e: 'toggleProxy'): void;
  (e: 'add'): void;
}>();

// 主题状态 - 使用 ref 而非 computed 以确保响应式更新
const themeMode = ref<ThemeMode>(getCurrentTheme());

// 解析当前实际主题（处理 systemdefault 情况）
const getResolvedTheme = () => {
  if (themeMode.value === 'systemdefault') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return themeMode.value;
};

const resolvedTheme = computed(() => getResolvedTheme());
const themeIcon = computed(() => (resolvedTheme.value === 'dark' ? 'moon' : 'sun'));

// 导航方法
const goToLogs = () => {
  router.push('/logs');
};

const goToSettings = () => {
  router.push('/settings');
};

// 切换主题 - 同时更新 ref 和 localStorage
const toggleTheme = () => {
  const currentResolved = getResolvedTheme();
  const next: ThemeMode = currentResolved === 'dark' ? 'light' : 'dark';
  themeMode.value = next; // 更新响应式状态
  setTheme(next); // 持久化并应用到 DOM
};
</script>

<style scoped>
/**
 * [INPUT]: source: design.md - Prism System 玻璃拟态样式规范
 * [OUTPUT]: 极致优化的胶囊导航样式 (Refined Jobsian Aesthetics)
 */

/* 胶囊进入动画 */
@keyframes capsuleSlideIn {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(30px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0) scale(1);
  }
}

/* 胶囊容器 */
.capsule-navigation {
  position: fixed;
  bottom: 2.5rem; /* 增加一点底部间距，更显悬浮 */
  left: 50%;
  transform: translateX(-50%);
  z-index: 1000;

  display: flex;
  align-items: center;
  gap: 0.75rem; /* 增加间距，让元素呼吸 */

  /* 极致玻璃拟态 (Crystal Prism) */
  background: var(--capsule-bg);
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid rgba(255, 255, 255, 0.4);

  padding: 6px 10px; /* 调整内边距 */
  border-radius: 999px;

  /* 多层 Jobs 风格阴影：环境光 + 核心阴影 */
  box-shadow:
    0 10px 30px rgba(0, 0, 0, 0.08),
    0 4px 12px rgba(0, 0, 0, 0.05),
    inset 0 0 0 1px rgba(255, 255, 255, 0.2);

  animation: capsuleSlideIn 0.6s cubic-bezier(0.23, 1, 0.32, 1);
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

:global(html.dark) .capsule-navigation {
  background: rgba(28, 28, 30, 0.7);
  border-color: rgba(255, 255, 255, 0.1);
  box-shadow:
    0 20px 40px rgba(0, 0, 0, 0.4),
    0 0 0 1px rgba(255, 255, 255, 0.05);
}

/* 移除 nav-divider，改用较大的 gap */

.nav-segment {
  display: flex;
  background: rgba(0, 0, 0, 0.03);
  border-radius: 999px;
  padding: 4px; /* 增加一点厚度感 */
  gap: 4px;
}

:global(html.dark) .nav-segment {
  background: rgba(255, 255, 255, 0.06);
}

.capsule-tab {
  border: none;
  background: transparent;
  padding: 7px 18px; /* 黄金比例呼吸空间 */
  border-radius: 999px;
  font-size: 0.82rem;
  font-weight: 500;
  color: var(--mac-text-secondary);
  cursor: pointer;
  transition: all 0.4s cubic-bezier(0.23, 1, 0.32, 1);
  white-space: nowrap;
}

.capsule-tab:hover {
  color: var(--mac-text);
}

.capsule-tab.active {
  background: var(--capsule-tab-active-bg, #fff);
  color: var(--capsule-tab-active-color, #000);
  font-weight: 600;
  box-shadow: var(--capsule-tab-active-shadow, 0 4px 12px rgba(0, 0, 0, 0.08));
}

/* 胶囊项目按钮 */
.capsule-item {
  position: relative;
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--mac-text-secondary);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
}

.capsule-item:hover {
  background: rgba(0, 0, 0, 0.04);
  color: var(--mac-text);
  transform: translateY(-2px);
}

:global(html.dark) .capsule-item:hover {
  background: rgba(255, 255, 255, 0.08);
}

/* 弱化突兀的蓝色激活态 */
.capsule-item.is-active {
  color: var(--mac-accent);
  background: color-mix(in srgb, var(--mac-accent) 6%, transparent); /* 极低饱和度背景 */
}

/* 柔和的阴影 glow */
.capsule-item.is-active svg {
  filter: drop-shadow(0 0 4px rgba(10, 132, 255, 0.3));
}

.text-accent {
  color: var(--mac-accent);
}

.animate-spin {
  animation: spin 1.2s cubic-bezier(0.5, 0, 0.5, 1) infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.capsule-item svg {
  width: 20px;
  height: 20px;
  display: block;
}

/* [简化] .mcp-icon 样式已移除 (simplify-ui-controls) */

/* 工具提示 (Tooltip) - 悬浮在胶囊上方，更精致 */
.capsule-item[data-tooltip]::after {
  content: attr(data-tooltip);
  position: absolute;
  bottom: calc(100% + 14px);
  left: 50%;
  transform: translateX(-50%) translateY(8px);
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(10px);
  color: #fff;
  font-size: 11px;
  letter-spacing: 0.02em;
  padding: 5px 10px;
  border-radius: 8px;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: all 0.3s cubic-bezier(0.23, 1, 0.32, 1);
  box-shadow: 0 5px 15px rgba(0, 0, 0, 0.15);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.capsule-item[data-tooltip]:hover::after {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}

:global(html.dark) .capsule-item[data-tooltip]::after {
  background: rgba(255, 255, 255, 0.9);
  color: #000;
  border: none;
}
</style>
