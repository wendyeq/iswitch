<!--
/**
 * ---
 * [INPUT]: {HUD Events}
 *   source: ../../composables/useHUDState.ts ([POS]: HUD 状态管理)
 *   source: ../../../../openspec/changes/simplify-mini-hud/specs/desktop/spec.md ([POS]: HUD 规范)
 * [OUTPUT]: {JSX.Element} - Mini HUD 界面
 * [POS]: Mini HUD 浮动窗口组件，显示实时 Token 统计和速度信息
 * [PROTOCOL]: FractalFlow v1.0
 * ---
 */
-->
<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { useHUDState } from '../../composables/useHUDState';

// 使用 HUD 状态 composable
const { isStreaming, currentEvent, displayTokens, formattedSpeed, closeHud } = useHUDState();

// 锁定状态：锁定 = 禁止拖动（不是 Click-Through）
const isLocked = ref(false);

// 切换锁定状态
function toggleLock() {
  isLocked.value = !isLocked.value;
}

// 键盘事件
function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    closeHud();
  }
}

onMounted(() => {
  // [Fix] 强制透明背景 - 多重保障
  const html = document.documentElement;
  const body = document.body;
  const app = document.getElementById('app');

  // 1. 直接设置 inline style (最高优先级)
  html.style.setProperty('background', 'transparent', 'important');
  body.style.setProperty('background', 'transparent', 'important');
  if (app) app.style.setProperty('background', 'transparent', 'important');

  // 2. 覆盖 CSS 变量 (确保暗色模式也生效)
  html.style.setProperty('--app-background', 'transparent');

  // 3. 添加 class 用于 CSS 选择器
  body.classList.add('is-mini-hud');

  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  document.body.classList.remove('is-mini-hud');
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <div
    class="hud-glass-container"
    :class="{ locked: isLocked, streaming: isStreaming }"
    :data-tauri-drag-region="!isLocked || undefined"
  >
    <!-- Speed: Hero metric, no label needed -->
    <div class="metric-hero" :data-tauri-drag-region="!isLocked || undefined">
      <span class="hero-value">{{ formattedSpeed }}</span>
      <span class="hero-unit">tok/s</span>
    </div>

    <!-- Subtle divider -->
    <div class="divider"></div>

    <!-- Tokens: Secondary metric -->
    <div class="metric-secondary" :data-tauri-drag-region="!isLocked || undefined">
      <span class="secondary-value">{{ Math.round(displayTokens).toLocaleString() }}</span>
      <span class="secondary-unit">tokens</span>
    </div>

    <!-- Subtle divider -->
    <div class="divider"></div>

    <!-- Model: Tertiary, very subtle -->
    <div class="metric-tertiary" :data-tauri-drag-region="!isLocked || undefined">
      <span class="model-name" :title="currentEvent?.model">{{ currentEvent?.model || '—' }}</span>
    </div>

    <!-- Pin Button: visible with neon glow when pinned -->
    <button class="pin-button" :class="{ pinned: isLocked }" @click="toggleLock" :title="isLocked ? 'Unlock' : 'Lock'">
      <!-- Lock indicator: filled circle when locked -->
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="4" :fill="isLocked ? 'currentColor' : 'none'" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
/* ============================================
   Jony Ive Aesthetic: "Numbers floating on glass"
   - No labels (the numbers speak for themselves)
   - No inner boxes (text floats directly on frosted glass)
   - Subtle hairline dividers
   - Hero metric dominates visually
   - Pin button invisible until hover
   ============================================ */

/* Force transparent background in HUD mode */
:global(body.is-mini-hud) {
  background: transparent !important;
  --app-background: transparent !important;
}

:global(html:has(body.is-mini-hud)) {
  background: transparent !important;
  --app-background: transparent !important;
}

:global(body.is-mini-hud),
:global(html:has(body.is-mini-hud)),
:global(body.is-mini-hud #app) {
  width: 100vw !important;
  height: 100vh !important;
  margin: 0 !important;
  padding: 0 !important;
  overflow: hidden !important;
  border: none !important;
  outline: none !important;
  box-shadow: none !important;
}

:global(body.is-mini-hud #app) {
  background: transparent !important;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
}

/* === The Glass Container === */
.hud-glass-container {
  width: 100%;
  height: 100%;
  padding: 16px 20px;

  /* Pure frosted glass with neon border in dark mode */
  background: var(--hud-glass-bg);
  backdrop-filter: blur(24px) saturate(180%);
  -webkit-backdrop-filter: blur(24px) saturate(180%);
  border: 1px solid var(--hud-glass-border);
  border-radius: 20px;
  overflow: hidden;
  /* Inner glow for dark mode depth */
  box-shadow: var(--capsule-inner-glow, none);

  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;

  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Helvetica Neue', sans-serif;
  color: var(--hud-text-primary);
  cursor: move;
  user-select: none;
  position: relative;
  pointer-events: auto;

  /* Subtle breathing glow when streaming */
  transition: box-shadow 0.4s ease;
}

.hud-glass-container.streaming {
  box-shadow: 0 0 30px var(--hud-streaming-glow-soft);
  animation: container-breathing 2.5s ease-in-out infinite;
}

@keyframes container-breathing {
  0%,
  100% {
    box-shadow: 0 0 20px var(--hud-streaming-glow-soft);
  }

  50% {
    box-shadow: 0 0 35px var(--hud-streaming-glow);
  }
}

.hud-glass-container.locked {
  cursor: default;
}

/* === Hero Metric (Speed) === */
.metric-hero {
  display: flex;
  align-items: baseline;
  gap: 6px;
  cursor: inherit;
}

.hero-value {
  font-size: 42px;
  font-weight: 200;
  letter-spacing: -2px;
  line-height: 1;
  color: var(--hud-text-primary);
  font-variant-numeric: tabular-nums;
}

.hero-unit {
  font-size: 14px;
  font-weight: 400;
  color: var(--hud-text-secondary);
  opacity: 0.7;
}

/* === Hairline Divider === */
.divider {
  width: 60%;
  height: 1px;
  background: linear-gradient(90deg, transparent 0%, var(--hud-text-secondary) 50%, transparent 100%);
  opacity: 0.15;
}

/* === Secondary Metric (Tokens) === */
.metric-secondary {
  display: flex;
  align-items: baseline;
  gap: 5px;
  cursor: inherit;
}

.secondary-value {
  font-size: 22px;
  font-weight: 300;
  letter-spacing: -0.5px;
  color: var(--hud-text-primary);
  font-variant-numeric: tabular-nums;
}

.secondary-unit {
  font-size: 11px;
  font-weight: 400;
  color: var(--hud-text-secondary);
  opacity: 0.6;
}

/* === Tertiary Metric (Model) === */
.metric-tertiary {
  cursor: inherit;
}

.model-name {
  font-size: 11px;
  font-weight: 400;
  color: var(--hud-text-secondary);
  opacity: 0.5;
  letter-spacing: 0.3px;
  text-transform: uppercase;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 140px;
}

/* === Pin Button === */
.pin-button {
  position: absolute;
  top: 10px;
  right: 10px;
  width: 22px;
  height: 22px;
  padding: 4px;
  border: 1.5px solid currentColor;
  border-radius: 50%;
  background: transparent;
  color: var(--hud-text-secondary);
  cursor: pointer;
  opacity: 0.4;
  transition:
    opacity 0.2s,
    color 0.2s,
    box-shadow 0.2s,
    border-color 0.2s;
  z-index: 10;
}

.hud-glass-container:hover .pin-button {
  opacity: 0.7;
}

.pin-button:hover {
  opacity: 1 !important;
}

/* Pinned state: accent color with glow */
.pin-button.pinned {
  opacity: 1;
  color: var(--capsule-accent-azure, #0a84ff);
  border-color: var(--capsule-accent-azure, #0a84ff);
  box-shadow: 0 0 10px color-mix(in srgb, var(--capsule-accent-azure, #0a84ff) 50%, transparent);
}

/* Dark mode: use neon cyan */
:global(html.dark) .pin-button.pinned {
  color: var(--capsule-accent-teal, #00f3ff);
  border-color: var(--capsule-accent-teal, #00f3ff);
  box-shadow: 0 0 12px var(--capsule-accent-teal, #00f3ff);
}

.pin-button svg {
  width: 100%;
  height: 100%;
}
</style>
