<!--
[INPUT]: 
  - source: openspec/specs/provider-capsules/spec.md ([POS]: 悬浮胶囊规范)
  - source: iswitch-tauri/src/data/cards.ts ([POS]: AutomationCard 数据结构)
  - source: iswitch-tauri/src/utils/providerDefaults.ts ([POS]: 默认配置查找)
  - source: iswitch-tauri/src/utils/providerDetector.ts ([POS]: 供应商检测逻辑)
  - source: tauri-fullstack-expert.md ([POS]: Vue 3 组件开发规范)
[OUTPUT]: 单个悬浮胶囊组件 - 支持 URL 图标与智能 Dashboard 链接
[PROTOCOL]: FractalFlow v1.0
[POS]: iswitch-tauri/src/components/Main/LevitatingCapsule.vue - 悬浮胶囊单项组件
-->
<template>
  <article
    :id="`capsule-${provider.id}`"
    class="levitating-capsule"
    :class="{
      'is-expanded': isExpanded,
      'is-active': isActive,
      'is-disabled': !provider.enabled,
      'is-dragging': isDragging,
      'is-drag-over': isDragOver,
    }"
    @click="onCapsuleClick"
  >
    <!-- 左侧：图标 + 信息 -->
    <div class="capsule-leading">
      <!-- 提供商图标 -->
      <div class="capsule-icon" :style="{ backgroundColor: provider.tint, color: provider.accent }">
        <img v-if="isIconUrl && !iconLoadError" :src="provider.icon" class="icon-img" alt="" @error="onIconError" />
        <span v-else-if="iconSvg" class="icon-svg" v-html="iconSvg" aria-hidden="true" />
        <span v-else class="icon-fallback">{{ initials }}</span>
      </div>

      <!-- 文字信息 -->
      <div class="capsule-info">
        <div class="capsule-header">
          <h3 class="capsule-name">{{ provider.name }}</h3>
          <!-- Auto/Manual Status Indicator -->
          <Transition name="fade">
            <div
              v-if="isActive && statusType"
              class="status-badge"
              :class="statusType"
              :title="
                statusType === 'auto'
                  ? t('components.main.capsule.autoSwitch')
                  : t('components.main.capsule.manualSelect')
              "
            >
              <svg v-if="statusType === 'auto'" viewBox="0 0 24 24" class="status-icon">
                <path
                  d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                />
              </svg>
              <svg v-else viewBox="0 0 24 24" class="status-icon">
                <path
                  d="M20 6L9 17l-5-5"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
              <span v-if="statusType === 'auto'" class="status-text">Auto</span>
            </div>
          </Transition>
          <!-- 迷你成功率徽章 -->
          <span v-if="stats?.successRate !== undefined && !isExpanded" class="capsule-badge" :class="successRateClass">
            {{ formatPercent(stats.successRate) }}
          </span>
        </div>
        <p v-if="!isExpanded" class="capsule-subtitle">
          {{ collapsedSummary }}
        </p>
        <div v-else class="range-indicator-inline">
          <svg viewBox="0 0 24 24" class="range-icon-mini">
            <path
              d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"
              stroke="currentColor"
              fill="none"
              stroke-width="2"
            />
            <path d="M12 6v6l4 2" stroke="currentColor" fill="none" stroke-width="2" stroke-linecap="round" />
          </svg>
          {{ t('components.main.capsule.last24h') }}
        </div>
      </div>
    </div>

    <!-- 右侧：控制按钮（不含 Toggle）-->
    <div class="capsule-actions">
      <!-- 官网按钮 (Compass: Discovery) -->
      <button
        v-if="effectiveDashboardUrl"
        class="capsule-action-btn"
        @click.stop="openOfficialSite"
        :aria-label="t('components.main.common.website') || 'Official Website'"
      >
        <svg
          viewBox="0 0 24 24"
          aria-hidden="true"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="10" />
          <polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76" />
        </svg>
      </button>

      <!-- 配置按钮 (Sliders: Tuning) -->
      <button
        class="capsule-action-btn"
        @click.stop="$emit('configure', provider)"
        :aria-label="t('components.main.common.configure')"
      >
        <svg
          viewBox="0 0 24 24"
          aria-hidden="true"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M4 21v-7" />
          <path d="M4 10V3" />
          <path d="M12 21v-9" />
          <path d="M12 8V3" />
          <path d="M20 21v-5" />
          <path d="M20 12V3" />
          <path d="M1 14h6" />
          <path d="M9 8h6" />
          <path d="M17 16h6" />
        </svg>
      </button>

      <!-- 删除按钮 (Trash: Clean) -->
      <button
        class="capsule-action-btn capsule-action-danger"
        @click.stop="$emit('remove', provider)"
        :aria-label="t('components.main.common.delete')"
      >
        <svg
          viewBox="0 0 24 24"
          aria-hidden="true"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="3 6 5 6 21 6" />
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          <line x1="10" y1="11" x2="10" y2="17" />
          <line x1="14" y1="11" x2="14" y2="17" />
        </svg>
      </button>

      <!-- Enable Toggle - 放在拖拽手柄左边 -->
      <label class="capsule-toggle" @click.stop>
        <input type="checkbox" :checked="provider.enabled" @change="onToggleEnabled" />
        <span class="toggle-track"><span class="toggle-thumb" /></span>
      </label>

      <!-- 拖拽手柄 -->
      <div
        class="capsule-drag-handle"
        @mousedown.stop="onDragHandleMouseDown"
        @touchstart.stop
        :aria-label="t('components.main.common.drag')"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M8 6h.01M8 12h.01M8 18h.01M16 6h.01M16 12h.01M16 18h.01"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
          />
        </svg>
      </div>
    </div>

    <!-- 展开后的 Dashboard -->
    <div v-if="isExpanded" class="capsule-dashboard" role="region" :aria-label="t('components.main.capsule.dashboard')">
      <div class="dashboard-grid">
        <!-- Metric 1: Success Rate (Neon Green) -->
        <div class="metric-card card-success">
          <div class="card-body">
            <div class="ring-chart-container">
              <div class="ring-wrapper">
                <svg class="ring-chart" viewBox="0 0 100 100">
                  <defs>
                    <filter id="neon-glow-green" x="-50%" y="-50%" width="200%" height="200%">
                      <feGaussianBlur stdDeviation="3" result="blur1" />
                      <feGaussianBlur stdDeviation="6" result="blur2" />
                      <feMerge>
                        <feMergeNode in="blur2" />
                        <feMergeNode in="blur1" />
                        <feMergeNode in="SourceGraphic" />
                      </feMerge>
                    </filter>
                  </defs>
                  <!-- Background ring -->
                  <circle class="ring-bg" cx="50" cy="50" r="42" />
                  <!-- Progress ring -->
                  <circle
                    class="ring-stroke"
                    :class="ringColorClass"
                    cx="50"
                    cy="50"
                    r="42"
                    :stroke-dasharray="`${(stats?.successRate ?? 0) * 264} 264`"
                  />
                </svg>
                <!-- Energy drip animation -->
                <div class="energy-drip"></div>
              </div>
              <div class="ring-text">
                <span class="ring-value">{{ formatPercent(stats?.successRate ?? 0) }}</span>
                <span class="ring-label">{{ t('components.main.capsule.successRate') }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Metric 2: Requests (Electric Blue) -->
        <div class="metric-card card-requests">
          <div class="card-body">
            <span class="big-metric">{{ formatNumber(stats?.requests ?? 0) }}</span>
            <span class="card-label">{{ t('components.main.capsule.requests') }}</span>
            <div class="sparkline-wrapper">
              <svg class="sparkline" :class="{ 'is-idle': isIdle }" viewBox="0 0 100 40" preserveAspectRatio="none">
                <defs>
                  <linearGradient id="spark-grad" x1="0" x2="0" y1="0" y2="1">
                    <stop offset="0%" stop-color="var(--metric-requests-text)" stop-opacity="0.4" />
                    <stop offset="100%" stop-color="var(--metric-requests-text)" stop-opacity="0" />
                  </linearGradient>
                  <filter id="spark-glow" x="-20%" y="-20%" width="140%" height="140%">
                    <feGaussianBlur stdDeviation="2" result="blur" />
                    <feMerge>
                      <feMergeNode in="blur" />
                      <feMergeNode in="SourceGraphic" />
                    </feMerge>
                  </filter>
                </defs>
                <path :d="sparklineAreaPath" fill="url(#spark-grad)" />
                <path
                  :d="sparklineLinePath"
                  fill="none"
                  stroke="var(--metric-requests-text)"
                  stroke-width="2"
                  filter="url(#spark-glow)"
                />
                <!-- Animated dots -->
                <circle
                  v-for="(point, i) in sparklinePoints"
                  :key="i"
                  v-show="!isIdle"
                  :cx="point.x"
                  :cy="point.y"
                  r="3"
                  fill="var(--metric-requests-text)"
                  class="spark-dot"
                  :style="{ animationDelay: `${i * 0.2}s` }"
                />
              </svg>
            </div>
          </div>
        </div>

        <!-- Metric 3: Tokens (Nebula Purple) -->
        <div class="metric-card card-tokens">
          <div class="card-body">
            <span class="big-metric">{{ formatNumber(stats?.tokens ?? 0) }}</span>
            <span class="card-label">{{ t('components.main.capsule.tokens') }}</span>
            <!-- Nebula particles -->
            <div class="nebula-container">
              <span v-for="n in 12" :key="n" class="nebula-particle" :class="`p${n}`"></span>
              <span v-for="n in 5" :key="`star-${n}`" class="star-sparkle" :class="`s${n}`"></span>
            </div>
          </div>
        </div>

        <!-- Metric 4: Cost (Solar Gold) -->
        <div class="metric-card card-cost">
          <div class="card-body">
            <span class="big-metric">{{ formatCost(stats?.cost ?? 0) }}</span>
            <span class="card-label">{{ t('components.main.capsule.cost') }}</span>
            <!-- SVG Stacked Coins -->
            <svg class="coins-svg" viewBox="0 0 64 64" aria-hidden="true">
              <defs>
                <linearGradient id="coin-grad" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" stop-color="#ffd700" />
                  <stop offset="50%" stop-color="#ffb800" />
                  <stop offset="100%" stop-color="#cc9900" />
                </linearGradient>
                <filter id="coin-glow" x="-50%" y="-50%" width="200%" height="200%">
                  <feGaussianBlur stdDeviation="2" result="blur" />
                  <feMerge>
                    <feMergeNode in="blur" />
                    <feMergeNode in="SourceGraphic" />
                  </feMerge>
                </filter>
              </defs>
              <!-- Bottom coin -->
              <ellipse cx="32" cy="52" rx="20" ry="6" fill="#996600" />
              <ellipse cx="32" cy="50" rx="20" ry="6" fill="url(#coin-grad)" filter="url(#coin-glow)" />
              <!-- Middle coin -->
              <ellipse cx="32" cy="42" rx="20" ry="6" fill="#996600" />
              <ellipse cx="32" cy="40" rx="20" ry="6" fill="url(#coin-grad)" filter="url(#coin-glow)" />
              <!-- Top coin -->
              <ellipse cx="32" cy="32" rx="20" ry="6" fill="#996600" />
              <ellipse cx="32" cy="30" rx="20" ry="6" fill="url(#coin-grad)" filter="url(#coin-glow)" />
              <!-- Dollar sign on top coin -->
              <text x="32" y="33" text-anchor="middle" font-size="10" font-weight="bold" fill="#664400">$</text>
            </svg>
          </div>
        </div>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { open } from '@tauri-apps/plugin-shell';
import lobeIcons from '../../icons/lobeIconMap';
import { getProviderDefaultOfficialSite } from '../../utils/providerDefaults';
import { detectProvider } from '../../utils/providerDetector';
import type { AutomationCard } from '../../data/cards';

interface ProviderStats {
  successRate: number; // 0-1
  requests: number;
  tokens: number;
  cost: number;
  hourlyRequests?: number[];
}

interface Props {
  provider: AutomationCard;
  index: number;
  isExpanded: boolean;
  isActive?: boolean;
  isDragging?: boolean;
  isDragOver?: boolean;
  stats?: ProviderStats | null;
  statusType?: 'auto' | 'manual' | null;
}

interface Emits {
  (e: 'toggle-expand'): void;
  (e: 'toggle-enabled', value: boolean): void;
  (e: 'configure', provider: AutomationCard): void;
  (e: 'remove', provider: AutomationCard): void;
  (e: 'drag-start', event: MouseEvent): void;
}

const props = withDefaults(defineProps<Props>(), {
  isActive: false,
  isDragging: false,
  isDragOver: false,
  stats: null,
  statusType: null,
});

const emit = defineEmits<Emits>();
const { t, locale } = useI18n();

// Check if provider is in "Idle" state (no traffic)
const isIdle = computed(() => {
  const raw = props.stats?.hourlyRequests;
  return !raw || raw.length === 0;
});

const isIconUrl = computed(() => {
  const icon = props.provider.icon;
  return icon && (icon.startsWith('http') || icon.startsWith('data:'));
});

const iconLoadError = ref(false);
const onIconError = () => {
  iconLoadError.value = true;
};

const effectiveDashboardUrl = computed(() => {
  // 1. Explicitly configured (Highest Priority)
  if (props.provider.officialSite) return props.provider.officialSite;

  // 2. Known Provider Type Default (Medium Priority)
  if (props.provider.type) {
    const defaultUrl = getProviderDefaultOfficialSite(props.provider.type);
    if (defaultUrl) return defaultUrl;
  }

  // 3. Auto-detected from API URL (if Type wasn't persisted or strictly known)
  // This handles legacy data where 'type' might be missing
  if (props.provider.apiUrl) {
    // Retrieve generic detection result
    const result = detectProvider(props.provider.apiUrl);
    if (result && result.officialSite) {
      return result.officialSite;
    }

    // 4. Generic Origin Fallback (Lowest Priority)
    try {
      const url = new URL(props.provider.apiUrl);
      return url.origin;
    } catch (e) {
      return '';
    }
  }
  return '';
});

// 计算属性
const iconSvg = computed(() => {
  if (isIconUrl.value && !iconLoadError.value) return '';
  const name = props.provider.icon?.toLowerCase();
  return name ? (lobeIcons[name] ?? '') : '';
});

const initials = computed(() => {
  const name = props.provider.name || '';
  return (
    name
      .split(/\s+/)
      .filter(Boolean)
      .map(word => word[0])
      .join('')
      .slice(0, 2)
      .toUpperCase() || 'AI'
  );
});

const successRateClass = computed(() => {
  const rate = props.stats?.successRate ?? 0;
  if (rate >= 0.95) return 'badge-healthy';
  if (rate >= 0.8) return 'badge-warning';
  return 'badge-critical';
});

// Ring Gauge 颜色根据成功率动态变化
const ringColorClass = computed(() => {
  const rate = props.stats?.successRate ?? 0;
  if (rate >= 0.95) return 'ring-healthy';
  if (rate >= 0.8) return 'ring-warning';
  return 'ring-critical';
});

const collapsedSummary = computed(() => {
  if (!props.stats) return t('components.main.capsule.firstRequest');
  const cost = formatCost(props.stats.cost);
  const tokens = formatNumber(props.stats.tokens);
  return `${cost} · ${tokens} tokens`;
});

// Sparkline data points
const sparklinePoints = computed(() => {
  const rawData = props.stats?.hourlyRequests;

  if (rawData && rawData.length > 1) {
    const maxVal = Math.max(...rawData, 1);
    const count = rawData.length;
    return rawData.map((val, i) => ({
      x: (i / (count - 1)) * 100,
      y: 40 - (val / maxVal) * 35, // Keep some padding at bottom (40 height)
    }));
  }

  // Fallback: Simulated Idle Signal ("Heartbeat of the Machine")
  // Maintains interface vitality even when there's no traffic.
  if (isIdle.value) {
    const idlePoints = [];
    const steps = 20;
    for (let i = 0; i <= steps; i++) {
      const t = i / steps;
      const x = t * 100;
      // Organic "idling" wave: subtle complex oscillation near bottom
      const signal = Math.sin(t * Math.PI * 4) * 0.8 + Math.sin(t * Math.PI * 8) * 0.3;
      const y = 38 - signal;
      idlePoints.push({ x, y });
    }
    return idlePoints;
  }

  return [];
});

const sparklineLinePath = computed(() => {
  const pts = sparklinePoints.value;
  if (pts.length === 0) return '';
  let d = `M${pts[0].x},${pts[0].y}`;
  for (let i = 1; i < pts.length; i++) {
    d += ` L${pts[i].x},${pts[i].y}`;
  }
  return d;
});

const sparklineAreaPath = computed(() => {
  const pts = sparklinePoints.value;
  if (pts.length === 0) return '';
  let d = `M${pts[0].x},40 L${pts[0].x},${pts[0].y}`;
  for (let i = 1; i < pts.length; i++) {
    d += ` L${pts[i].x},${pts[i].y}`;
  }
  d += ` L${pts[pts.length - 1].x},40 Z`;
  return d;
});

// 格式化函数
const currencyFormatter = computed(
  () =>
    new Intl.NumberFormat(locale.value || 'en', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })
);

const formatPercent = (value: number) => {
  const percent = Math.max(0, Math.min(1, value)) * 100;
  return `${percent.toFixed(percent >= 99.5 || percent === 0 ? 0 : 1)}%`;
};

const formatNumber = (value: number) => {
  if (value >= 1_000_000_000) return `${(value / 1_000_000_000).toFixed(1)}B`;
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return value.toLocaleString();
};

const formatCost = (value: number) => currencyFormatter.value.format(Math.max(0, value));

// 状态计算
const isExpanded = computed(() => props.isExpanded);

// 打开官网 (使用 Tauri shell API)
const openOfficialSite = async () => {
  if (effectiveDashboardUrl.value) {
    try {
      await open(effectiveDashboardUrl.value);
    } catch (e) {
      // Fallback to window.open for development/web
      window.open(effectiveDashboardUrl.value, '_blank');
    }
  }
};

// 事件处理
const onCapsuleClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (target.closest('.capsule-actions')) return;
  emit('toggle-expand');
};

const onToggleEnabled = (e: Event) => {
  const checked = (e.target as HTMLInputElement).checked;
  emit('toggle-enabled', checked);
};

// 拖拽手柄按下事件 (Tauri 兼容)
const onDragHandleMouseDown = (e: MouseEvent) => {
  emit('drag-start', e);
};
</script>

<style scoped>
/* === Base Capsule Styles === */
.levitating-capsule {
  position: relative;
  background: var(--capsule-bg);
  border-radius: 24px;
  border: 1px solid var(--capsule-border);
  box-shadow: var(--capsule-shadow);
  transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
  margin-bottom: 16px;
  overflow: hidden;
  backdrop-filter: var(--capsule-backdrop-blur);
  padding: 0;
  transform: translateZ(0);
  display: flex;
  flex-direction: column;
}

.levitating-capsule::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent 0%, rgba(255, 255, 255, 0.4) 50%, transparent 100%);
  opacity: 0.6;
  pointer-events: none;
}

.levitating-capsule:hover {
  transform: translateY(-2px);
  box-shadow: var(--capsule-shadow-hover);
}

.levitating-capsule.is-expanded {
  border-color: var(--capsule-border-active);
  /* Jobs Mode: 浮起 + 蓝色光环 */
  transform: translateY(-4px);
  box-shadow:
    var(--capsule-shadow-hover),
    0 0 0 1px var(--capsule-accent-azure),
    0 0 20px rgba(59, 130, 246, 0.15);
  z-index: 10;
}

/* === Active State (System Selected) === */
.levitating-capsule.is-active {
  animation: breathe-glow 3.2s ease-in-out infinite;
  border-color: rgba(59, 130, 246, 0.5);
  background: linear-gradient(to right, rgba(59, 130, 246, 0.03), transparent);
  z-index: 5;
}

/* === 拖拽状态 (Dragging States) === */
.levitating-capsule.is-dragging {
  opacity: 0.6;
  transform: scale(0.98);
  box-shadow: var(--capsule-shadow);
  z-index: 100;
}

.levitating-capsule.is-drag-over {
  /* 拖拽目标位置指示器 */
  border-top: 3px solid var(--capsule-accent-azure);
  transform: translateY(2px);
  transition: all 0.15s ease-out;
}

.levitating-capsule.is-drag-over::after {
  content: '';
  position: absolute;
  top: -8px;
  left: 50%;
  transform: translateX(-50%);
  width: 40%;
  height: 4px;
  background: var(--capsule-accent-azure);
  border-radius: 2px;
  animation: drag-indicator-pulse 1s ease-in-out infinite;
}

@keyframes drag-indicator-pulse {
  0%,
  100% {
    opacity: 0.6;
  }

  50% {
    opacity: 1;
  }
}

@keyframes breathe-glow {
  0%,
  100% {
    box-shadow:
      0 0 20px rgba(14, 165, 233, 0.3),
      inset 0 0 15px rgba(14, 165, 233, 0.1);
  }

  50% {
    box-shadow:
      0 0 35px rgba(14, 165, 233, 0.5),
      inset 0 0 25px rgba(14, 165, 233, 0.2);
  }
}

/* === Header === */
.capsule-leading {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 24px;
  padding-right: 180px;
  /* 为右侧 actions + toggle 留空间 */
  width: 100%;
}

/* 压缩展开时的间距 */
.levitating-capsule.is-expanded .capsule-leading {
  padding-bottom: 8px;
}

/* Flow Point 已移除 (Jobs Mode) */

.capsule-icon {
  width: 44px;
  height: 44px;
  border-radius: 14px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  font-size: 1rem;
  font-weight: 600;
  overflow: hidden;
}

.icon-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.capsule-icon .icon-svg :deep(svg) {
  width: 22px;
  height: 22px;
}

.capsule-info {
  flex: 1;
  min-width: 0;
}

.capsule-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.capsule-name {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--capsule-text-primary);
}

.capsule-subtitle {
  margin: 2px 0 0;
  font-size: 0.85rem;
  color: var(--capsule-text-secondary);
}

.capsule-badge {
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 600;
}

.capsule-badge.badge-healthy {
  background: color-mix(in srgb, var(--capsule-ring-healthy) 20%, transparent);
  color: var(--capsule-ring-healthy);
}

.capsule-badge.badge-warning {
  background: color-mix(in srgb, var(--capsule-accent-gold) 20%, transparent);
  color: var(--capsule-accent-gold);
}

.capsule-badge.badge-critical {
  background: color-mix(in srgb, #ef4444 20%, transparent);
  color: #ef4444;
}

/* === Actions === */
.capsule-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-right: 24px;
  position: absolute;
  right: 0;
  top: 22px;
}

.capsule-toggle {
  display: inline-flex;
  cursor: pointer;
}

.capsule-toggle input {
  display: none;
}

.toggle-track {
  width: 36px;
  height: 20px;
  border-radius: 999px;
  background: rgba(120, 120, 128, 0.3);
  position: relative;
  transition: background 0.2s ease;
}

.toggle-thumb {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.2s ease;
}

.capsule-toggle input:checked + .toggle-track {
  background: var(--capsule-accent-azure);
}

.capsule-toggle input:checked + .toggle-track .toggle-thumb {
  transform: translateX(16px);
}

.capsule-action-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.05);
  color: var(--capsule-text-secondary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s;
  /* Jobs Mode: 默认隐藏，Hover 时渐显 */
  opacity: 0;
  pointer-events: none;
  /* 默认不可点击 */
}

/* Jobs Mode: Hover/展开时显示按钮 */
.levitating-capsule:hover .capsule-action-btn,
.levitating-capsule.is-expanded .capsule-action-btn {
  opacity: 1;
  pointer-events: auto;
  /* 恢复可点击 */
}

.capsule-action-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--capsule-text-primary);
}

/* Status Badge */
.status-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
  border: 1px solid transparent;
}

.status-badge.auto {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
  border-color: rgba(245, 158, 11, 0.3);
}

.status-badge.manual {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
  border-color: rgba(16, 185, 129, 0.3);
}

.status-icon {
  width: 12px;
  height: 12px;
  fill: none;
}

.status-badge.auto .status-icon {
  animation: spin-slow 4s linear infinite;
}

.capsule-action-btn.capsule-action-danger:hover {
  color: var(--capsule-ring-critical);
  background: rgba(239, 68, 68, 0.1);
}

.capsule-action-btn svg {
  width: 16px;
  height: 16px;
}

.capsule-drag-handle {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--capsule-text-secondary);
  cursor: grab;
}

.capsule-drag-handle svg {
  width: 20px;
  height: 20px;
}

/* === Dashboard (Expanded) === */
.capsule-dashboard {
  padding: 0 24px 24px 24px;
  /* 左右对称，子卡片靠边 */
  animation: dashboard-expand 0.4s ease-out forwards;
}

.range-indicator-inline {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 0.7rem;
  font-weight: 500;
  color: var(--capsule-text-secondary);
  opacity: 0.6;
  margin-top: 2px;
}

.range-icon-mini {
  width: 10px;
  height: 10px;
}

@keyframes dashboard-expand {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-top: 4px;
  /* 从 12px 减小到 4px */
}

/* === Metric Cards (HIGH-FIDELITY) === */
.metric-card {
  border-radius: 16px;
  padding: 20px;
  min-height: 140px;
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  backdrop-filter: blur(12px);
  transition:
    transform 0.3s,
    box-shadow 0.3s;
}

.metric-card:hover {
  transform: translateY(-4px);
}

.card-body {
  display: flex;
  flex-direction: column;
  position: relative;
  flex: 1;
}

.big-metric {
  font-size: 2.2rem;
  font-weight: 800;
  letter-spacing: -0.03em;
  line-height: 1;
  margin-bottom: 4px;
  text-shadow:
    0 1px 2px rgba(0, 0, 0, 0.1),
    0 0 20px currentColor;
}

.card-label {
  font-size: 0.85rem;
  text-transform: capitalize;
  letter-spacing: 0.02em;
  opacity: 0.8;
  font-weight: 500;
}

/* --- Success Card --- */
.card-success {
  background:
    radial-gradient(
      ellipse at 50% 0%,
      color-mix(in srgb, var(--metric-success-text) 20%, transparent),
      transparent 60%
    ),
    rgba(0, 30, 20, 0.7);
  border: 1px solid color-mix(in srgb, var(--metric-success-text) 30%, transparent);
  box-shadow:
    inset 0 0 40px color-mix(in srgb, var(--metric-success-text) 8%, transparent),
    0 0 20px color-mix(in srgb, var(--metric-success-text) 20%, transparent);
}

.card-success .ring-value,
.card-success .ring-label {
  color: var(--capsule-ring-healthy);
}

.ring-chart-container {
  display: flex;
  align-items: center;
  gap: 12px;
}

.ring-wrapper {
  position: relative;
  width: 80px;
  height: 80px;
}

.ring-chart {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}

.ring-bg {
  fill: none;
  stroke: rgba(0, 255, 136, 0.15);
  stroke-width: 8;
}

.ring-stroke {
  fill: none;
  stroke: var(--capsule-ring-healthy);
  stroke-width: 8;
  stroke-linecap: round;
  transition:
    stroke-dasharray 1s ease,
    stroke 0.3s ease;
}

/* Ring colors based on success rate */
.ring-stroke.ring-healthy {
  stroke: var(--capsule-ring-healthy);
  filter: drop-shadow(0 0 6px color-mix(in srgb, var(--capsule-ring-healthy) 60%, transparent));
}

.ring-stroke.ring-warning {
  stroke: var(--capsule-accent-gold, #f59e0b);
  filter: drop-shadow(0 0 6px rgba(245, 158, 11, 0.6));
}

.ring-stroke.ring-critical {
  stroke: #ef4444;
  filter: drop-shadow(0 0 6px rgba(239, 68, 68, 0.6));
}

.energy-drip {
  position: absolute;
  bottom: -8px;
  left: 50%;
  width: 6px;
  height: 12px;
  background: var(--capsule-ring-healthy);
  border-radius: 50%;
  filter: blur(2px);
  animation: drip 2s infinite ease-in-out;
  transform: translateX(-50%);
}

@keyframes drip {
  0%,
  100% {
    opacity: 0;
    transform: translateX(-50%) translateY(-5px);
  }

  50% {
    opacity: 1;
    transform: translateX(-50%) translateY(5px);
  }
}

.ring-text {
  display: flex;
  flex-direction: column;
}

.ring-value {
  font-size: 1.8rem;
  font-weight: 800;
  text-shadow: 0 0 10px currentColor;
}

.ring-label {
  font-size: 0.8rem;
  opacity: 0.8;
}

.success-arrow {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 24px;
  height: 24px;
  color: var(--capsule-ring-healthy);
  filter: drop-shadow(0 0 4px var(--capsule-ring-healthy));
}

/* --- Requests Card --- */
.card-requests {
  background: radial-gradient(ellipse at 50% 0%, rgba(59, 130, 246, 0.2), transparent 60%), rgba(10, 20, 40, 0.7);
  border: 1px solid rgba(59, 130, 246, 0.3);
  box-shadow:
    inset 0 0 40px rgba(59, 130, 246, 0.08),
    0 0 20px rgba(59, 130, 246, 0.2);
}

.card-requests .big-metric,
.card-requests .card-label {
  color: #60a5fa;
}

.card-requests .big-metric {
  text-shadow: 0 0 12px rgba(96, 165, 250, 0.6);
}

.sparkline-wrapper {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 50px;
  opacity: 0.9;
}

.sparkline {
  width: 100%;
  height: 100%;
}

.spark-dot {
  animation: pulse-dot 1.5s infinite ease-in-out;
}

@keyframes pulse-dot {
  0%,
  100% {
    opacity: 0.4;
    r: 3;
  }

  50% {
    opacity: 1;
    r: 5;
  }
}

/* --- Tokens Card --- */
.card-tokens {
  background: radial-gradient(ellipse at 50% 0%, rgba(139, 92, 246, 0.2), transparent 60%), rgba(20, 10, 40, 0.7);
  border: 1px solid rgba(139, 92, 246, 0.3);
  box-shadow:
    inset 0 0 40px rgba(139, 92, 246, 0.08),
    0 0 20px rgba(139, 92, 246, 0.2);
}

.card-tokens .big-metric,
.card-tokens .card-label {
  color: #a78bfa;
}

.card-tokens .big-metric {
  text-shadow: 0 0 12px rgba(167, 139, 250, 0.6);
}

.nebula-container {
  position: absolute;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
}

.nebula-particle {
  position: absolute;
  background: #a78bfa;
  border-radius: 50%;
  filter: blur(1px);
  animation: float-nebula 4s infinite ease-in-out;
}

.p1 {
  width: 4px;
  height: 4px;
  left: 70%;
  top: 60%;
  animation-delay: 0s;
}

.p2 {
  width: 6px;
  height: 6px;
  left: 80%;
  top: 40%;
  animation-delay: 0.5s;
}

.p3 {
  width: 3px;
  height: 3px;
  left: 60%;
  top: 75%;
  animation-delay: 1s;
}

.p4 {
  width: 5px;
  height: 5px;
  left: 85%;
  top: 55%;
  animation-delay: 1.5s;
}

.p5 {
  width: 4px;
  height: 4px;
  left: 75%;
  top: 30%;
  animation-delay: 2s;
}

.p6 {
  width: 3px;
  height: 3px;
  left: 90%;
  top: 70%;
  animation-delay: 0.3s;
}

.p7 {
  width: 5px;
  height: 5px;
  left: 65%;
  top: 45%;
  animation-delay: 0.8s;
}

.p8 {
  width: 4px;
  height: 4px;
  left: 78%;
  top: 65%;
  animation-delay: 1.2s;
}

.p9 {
  width: 3px;
  height: 3px;
  left: 88%;
  top: 35%;
  animation-delay: 1.8s;
}

.p10 {
  width: 6px;
  height: 6px;
  left: 72%;
  top: 50%;
  animation-delay: 2.2s;
}

.p11 {
  width: 4px;
  height: 4px;
  left: 82%;
  top: 25%;
  animation-delay: 0.6s;
}

.p12 {
  width: 3px;
  height: 3px;
  left: 68%;
  top: 80%;
  animation-delay: 1.4s;
}

@keyframes float-nebula {
  0%,
  100% {
    transform: translate(0, 0);
    opacity: 0.3;
  }

  50% {
    transform: translate(-8px, -12px);
    opacity: 0.8;
  }
}

.star-sparkle {
  position: absolute;
  width: 2px;
  height: 2px;
  background: white;
  border-radius: 50%;
  animation: twinkle 2s infinite;
}

.s1 {
  left: 75%;
  top: 40%;
  animation-delay: 0s;
}

.s2 {
  left: 85%;
  top: 60%;
  animation-delay: 0.4s;
}

.s3 {
  left: 65%;
  top: 55%;
  animation-delay: 0.8s;
}

.s4 {
  left: 80%;
  top: 30%;
  animation-delay: 1.2s;
}

.s5 {
  left: 70%;
  top: 70%;
  animation-delay: 1.6s;
}

@keyframes twinkle {
  0%,
  100% {
    opacity: 0;
    transform: scale(0.5);
  }

  50% {
    opacity: 1;
    transform: scale(1.5);
  }
}

/* --- Cost Card --- */
.card-cost {
  background: radial-gradient(ellipse at 50% 0%, rgba(245, 158, 11, 0.2), transparent 60%), rgba(40, 30, 10, 0.7);
  border: 1px solid rgba(245, 158, 11, 0.3);
  box-shadow:
    inset 0 0 40px rgba(245, 158, 11, 0.08),
    0 0 20px rgba(245, 158, 11, 0.2);
}

.card-cost .big-metric,
.card-cost .card-label {
  color: #fbbf24;
}

.card-cost .big-metric {
  text-shadow:
    0 0 15px rgba(255, 215, 0, 0.5),
    0 0 30px rgba(255, 215, 0, 0.3);
}

.coins-svg {
  position: absolute;
  bottom: 4px;
  right: 4px;
  width: 56px;
  height: 56px;
  filter: drop-shadow(0 0 6px rgba(255, 215, 0, 0.6));
}

@media (max-width: 600px) {
  .dashboard-grid {
    grid-template-columns: 1fr 1fr;
  }

  .capsule-dashboard {
    padding: 0 16px 20px 16px;
  }

  .big-metric {
    font-size: 1.6rem;
  }
}

/* === Light Mode Overrides (Refined "Jobs" Polish) === */
/* 
  Design Philosophy: "Retina Clarity"
  - Typography: Sharp, no shadows, high contrast. "Ink on paper".
  - Borders: Crisp 1px physical edges. "Cut glass".
  - Backgrounds: Subtle, crystalline, breathable.
*/

/* Badge colors for Light Mode - ensure proper contrast */
:root .capsule-badge.badge-healthy,
html:not(.dark) .capsule-badge.badge-healthy {
  background: rgba(13, 148, 136, 0.15);
  color: #0f766e;
}

:root .capsule-badge.badge-warning,
html:not(.dark) .capsule-badge.badge-warning {
  background: rgba(217, 119, 6, 0.15);
  color: #b45309;
}

:root .capsule-badge.badge-critical,
html:not(.dark) .capsule-badge.badge-critical {
  background: rgba(220, 38, 38, 0.15);
  color: #dc2626;
}

:root .card-success,
html:not(.dark) .card-success {
  background: radial-gradient(ellipse at 50% 0%, rgba(13, 148, 136, 0.06), transparent 50%), rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(13, 148, 136, 0.3);
  box-shadow:
    0 4px 12px rgba(13, 148, 136, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
  /* Inner top highlight */
  backdrop-filter: blur(20px);
}

:root .card-success .ring-value,
:root .card-success .ring-label,
html:not(.dark) .card-success .ring-value,
html:not(.dark) .card-success .ring-label {
  color: #0f766e;
  /* Darker teal for contrast */
  text-shadow: none !important;
  /* Force remove default glow */
}

:root .ring-stroke,
html:not(.dark) .ring-stroke {
  stroke: #0d9488;
  filter: none;
  /* Remove glow for crispness */
}

:root .ring-stroke.ring-healthy,
html:not(.dark) .ring-stroke.ring-healthy {
  stroke: #0d9488;
  filter: none;
}

:root .ring-stroke.ring-warning,
html:not(.dark) .ring-stroke.ring-warning {
  stroke: #d97706;
  filter: none;
}

:root .ring-stroke.ring-critical,
html:not(.dark) .ring-stroke.ring-critical {
  stroke: #dc2626;
  filter: none;
}

:root .ring-bg,
html:not(.dark) .ring-bg {
  stroke: rgba(13, 148, 136, 0.08);
}

:root .energy-drip,
html:not(.dark) .energy-drip {
  background: #0d9488;
  filter: none;
  opacity: 0.6;
}

:root .card-requests,
html:not(.dark) .card-requests {
  background: radial-gradient(ellipse at 50% 0%, rgba(2, 132, 199, 0.06), transparent 50%), rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(2, 132, 199, 0.3);
  box-shadow:
    0 4px 12px rgba(2, 132, 199, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(20px);
}

:root .card-requests .big-metric,
:root .card-requests .card-label,
html:not(.dark) .card-requests .big-metric,
html:not(.dark) .card-requests .card-label {
  color: #0369a1;
  /* Darker blue */
}

:root .card-requests .big-metric,
html:not(.dark) .card-requests .big-metric {
  text-shadow: none;
  /* Razor sharp */
}

/* Sharpen sparkline in light mode by removing filter */
html:not(.dark) .card-requests .sparkline path[stroke] {
  filter: none !important;
  stroke-width: 2.5px;
}

:root .card-tokens,
html:not(.dark) .card-tokens {
  background: radial-gradient(ellipse at 50% 0%, rgba(71, 85, 105, 0.05), transparent 50%), rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(71, 85, 105, 0.25);
  box-shadow:
    0 4px 12px rgba(71, 85, 105, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(20px);
}

:root .card-tokens .big-metric,
:root .card-tokens .card-label,
html:not(.dark) .card-tokens .big-metric,
html:not(.dark) .card-tokens .card-label {
  color: #334155;
}

:root .card-tokens .big-metric,
html:not(.dark) .card-tokens .big-metric {
  text-shadow: none;
}

:root .nebula-particle,
html:not(.dark) .nebula-particle {
  background: #94a3b8;
  opacity: 0.5;
}

:root .card-cost,
html:not(.dark) .card-cost {
  background: radial-gradient(ellipse at 50% 0%, rgba(180, 83, 9, 0.05), transparent 50%), rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(180, 83, 9, 0.25);
  box-shadow:
    0 4px 12px rgba(180, 83, 9, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(20px);
}

:root .card-cost .big-metric,
:root .card-cost .card-label,
html:not(.dark) .card-cost .big-metric,
html:not(.dark) .card-cost .card-label {
  color: #b45309;
}

:root .card-cost .big-metric,
html:not(.dark) .card-cost .big-metric {
  text-shadow: none;
}

/* Sharpen coins shadow in light mode */
html:not(.dark) .coins-svg {
  filter: drop-shadow(0 2px 4px rgba(180, 83, 9, 0.2));
}

/* Dark mode - restore and refine original styles (Precision Mode) */
html.dark .card-success {
  background: linear-gradient(180deg, rgba(34, 197, 94, 0.08) 0%, rgba(10, 20, 25, 0.4) 100%), rgba(5, 10, 15, 0.85);
  border: 1px solid rgba(34, 197, 94, 0.25);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
  /* Top lighting */
}

html.dark .card-success .ring-value,
html.dark .card-success .ring-label {
  color: #4ade80;
  /* Brighter green for readability */
  text-shadow: none;
  /* Anti-radioactive */
}

html.dark .ring-stroke {
  stroke: #22c55e;
  filter: drop-shadow(0 0 2px rgba(34, 197, 94, 0.5));
  /* Subtle crisp glow */
}

html.dark .ring-bg {
  stroke: rgba(34, 197, 94, 0.1);
}

html.dark .energy-drip {
  background: #4ade80;
  box-shadow: 0 0 8px rgba(34, 197, 94, 0.6);
}

html.dark .card-requests {
  background: linear-gradient(180deg, rgba(59, 130, 246, 0.08) 0%, rgba(10, 15, 30, 0.4) 100%), rgba(5, 10, 20, 0.85);
  border: 1px solid rgba(59, 130, 246, 0.25);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

html.dark .card-requests .big-metric,
html.dark .card-requests .card-label {
  color: #60a5fa;
}

html.dark .card-requests .big-metric {
  text-shadow: none;
  /* Sharpness priority */
}

/* Refine sparkline usage in dark mode */
html.dark .card-requests .sparkline path[stroke] {
  stroke-width: 2px;
  filter: drop-shadow(0 0 4px rgba(59, 130, 246, 0.4));
}

html.dark .card-tokens {
  background: linear-gradient(180deg, rgba(139, 92, 246, 0.08) 0%, rgba(15, 10, 30, 0.4) 100%), rgba(10, 5, 20, 0.85);
  border: 1px solid rgba(139, 92, 246, 0.25);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

html.dark .card-tokens .big-metric,
html.dark .card-tokens .card-label {
  color: #a78bfa;
}

html.dark .card-tokens .big-metric {
  text-shadow: none;
}

html.dark .nebula-particle {
  background: #c4b5fd;
  opacity: 0.6;
}

html.dark .card-cost {
  background: linear-gradient(180deg, rgba(245, 158, 11, 0.08) 0%, rgba(30, 20, 10, 0.4) 100%), rgba(20, 15, 5, 0.85);
  border: 1px solid rgba(245, 158, 11, 0.25);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}

html.dark .card-cost .big-metric,
html.dark .card-cost .card-label {
  color: #fbbf24;
}

html.dark .card-cost .big-metric {
  text-shadow: none;
}

html.dark .coins-svg {
  filter: drop-shadow(0 0 8px rgba(245, 158, 11, 0.3));
}
</style>

<style scoped>
/* Idle State Animation (Simulated Heartbeat) */
.sparkline.is-idle path {
  opacity: 0.6;
  animation: idle-breathe 6s ease-in-out infinite;
}

@keyframes idle-breathe {
  0%,
  100% {
    opacity: 0.4;
    stroke-width: 1.5px;
  }

  50% {
    opacity: 0.7;
    stroke-width: 2px;
  }
}

html.dark .sparkline.is-idle path {
  filter: drop-shadow(0 0 2px rgba(96, 165, 250, 0.3));
}
</style>
