<!--
[INPUT]: 
  - source: openspec/specs/provider-capsules/spec.md ([POS]: LevitatingProviderList 组件规范)
  - source: iswitch-tauri/src/components/Main/LevitatingCapsule.vue ([POS]: 单个胶囊组件)
  - source: tauri-fullstack-expert.md ([POS]: Vue 3 组件开发规范)
[OUTPUT]: 悬浮胶囊列表容器 - 包含 Flow Line 可视化、拖拽排序、列表状态管理
[PROTOCOL]: FractalFlow v1.0
[POS]: iswitch-tauri/src/components/Main/LevitatingProviderList.vue - 悬浮胶囊列表容器组件
-->
<template>
  <div class="levitating-provider-list" :class="{ 'is-dragging': draggingIndex !== null }">
    <!-- Flow Line 已移除 (Jobs Mode) -->

    <!-- 胶囊列表 -->
    <TransitionGroup name="capsule-list" tag="div" class="capsule-container" ref="containerRef">
      <LevitatingCapsule
        v-for="(provider, index) in providers"
        :key="provider.id"
        :provider="provider"
        :index="index"
        :is-expanded="expandedId === provider.id"
        :is-active="activeId !== null ? provider.id === activeId : index === 0"
        :is-dragging="draggingIndex === index"
        :is-drag-over="dragOverIndex === index"
        :stats="getProviderStats(provider.name)"
        :status-type="activeId === provider.id ? activeStatusType : null"
        @drag-start="onMouseDragStart(index, $event)"
        @toggle-expand="onToggleExpand(provider.id)"
        @toggle-enabled="onToggleEnabled(provider, $event)"
        @configure="$emit('configure', provider)"
        @remove="$emit('remove', provider)"
      />
    </TransitionGroup>

    <!-- 空状态 -->
    <div v-if="providers.length === 0" class="empty-state">
      <div class="empty-icon">
        <svg viewBox="0 0 48 48" aria-hidden="true">
          <circle cx="24" cy="24" r="20" fill="none" stroke="currentColor" stroke-width="2" opacity="0.3" />
          <path d="M24 16v16M16 24h16" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
      </div>
      <p class="empty-text">{{ t('components.main.capsule.emptyState') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import LevitatingCapsule from './LevitatingCapsule.vue';
import type { AutomationCard } from '../../data/cards';

interface ProviderStats {
  successRate: number;
  requests: number;
  tokens: number;
  cost: number;
  hourlyRequests?: number[];
}

interface Props {
  providers: AutomationCard[];
  statsMap?: Record<string, ProviderStats>;
  activeId?: number | null;
  activeStatusType?: 'auto' | 'manual' | null;
}

interface Emits {
  (e: 'reorder', providers: AutomationCard[]): void;
  (e: 'toggle-enabled', provider: AutomationCard, enabled: boolean): void;
  (e: 'configure', provider: AutomationCard): void;
  (e: 'remove', provider: AutomationCard): void;
}

const props = withDefaults(defineProps<Props>(), {
  statsMap: () => ({}),
  activeId: null,
  activeStatusType: null,
});

const emit = defineEmits<Emits>();
const { t } = useI18n();

// 展开状态
const expandedId = ref<number | null>(null);

// 拖拽状态
const draggingIndex = ref<number | null>(null);
const dragOverIndex = ref<number | null>(null);

// Flow Line 已移除 - 相关计算属性已清理

// 获取 Provider 统计数据
const getProviderStats = (name: string): ProviderStats | null => {
  const key = name?.trim().toLowerCase() ?? '';
  return props.statsMap[key] ?? null;
};

// 展开/收起
const onToggleExpand = (id: number) => {
  expandedId.value = expandedId.value === id ? null : id;
};

// 启用/禁用切换
const onToggleEnabled = (provider: AutomationCard, enabled: boolean) => {
  emit('toggle-enabled', provider, enabled);
};

// === 基于鼠标事件的拖拽实现 (Tauri 兼容) ===
const containerRef = ref<HTMLElement | null>(null);
const startY = ref(0);
const currentY = ref(0);

// 从子组件接收拖拽启动信号
const onMouseDragStart = (index: number, event: MouseEvent) => {
  draggingIndex.value = index;
  expandedId.value = null; // 收起所有展开的胶囊
  startY.value = event.clientY;
  currentY.value = event.clientY;

  // 添加全局事件监听
  document.addEventListener('mousemove', onMouseMove);
  document.addEventListener('mouseup', onMouseUp);
  document.body.style.userSelect = 'none';
  document.body.style.cursor = 'grabbing';
};

const onMouseMove = (event: MouseEvent) => {
  if (draggingIndex.value === null) return;

  currentY.value = event.clientY;

  // 获取容器内的所有胶囊元素
  const container = (containerRef.value as any)?.$el || containerRef.value;
  if (!container) return;

  const capsules = container.querySelectorAll('.levitating-capsule');

  // 确定鼠标在哪个胶囊上方
  for (let i = 0; i < capsules.length; i++) {
    if (i === draggingIndex.value) continue;

    const rect = capsules[i].getBoundingClientRect();
    const midY = rect.top + rect.height / 2;

    if (event.clientY < midY && event.clientY > rect.top - 20) {
      dragOverIndex.value = i;
      return;
    } else if (event.clientY >= midY && event.clientY < rect.bottom + 20) {
      dragOverIndex.value = i;
      return;
    }
  }
};

const onMouseUp = () => {
  // 执行排序
  if (draggingIndex.value !== null && dragOverIndex.value !== null && draggingIndex.value !== dragOverIndex.value) {
    const newProviders = [...props.providers];
    const [removed] = newProviders.splice(draggingIndex.value, 1);
    newProviders.splice(dragOverIndex.value, 0, removed);
    emit('reorder', newProviders);
  }

  // 重置状态
  draggingIndex.value = null;
  dragOverIndex.value = null;

  // 移除全局事件监听
  document.removeEventListener('mousemove', onMouseMove);
  document.removeEventListener('mouseup', onMouseUp);
  document.body.style.userSelect = '';
  document.body.style.cursor = '';
};

// 组件卸载时清理
onUnmounted(() => {
  document.removeEventListener('mousemove', onMouseMove);
  document.removeEventListener('mouseup', onMouseUp);
});

// 当 providers 变化时
watch(
  () => props.providers.length,
  async () => {
    await nextTick();
  }
);
</script>

<style scoped>
.levitating-provider-list {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 4px;
  /* 移除了左侧 padding，图标靠左 */
}

/* Flow Line 已移除 (Jobs Mode) */

/* Capsule Container */
.capsule-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
  position: relative;
  z-index: 1;
}

/* List Transitions */
.capsule-list-move,
.capsule-list-enter-active,
.capsule-list-leave-active {
  transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.capsule-list-enter-from {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}

.capsule-list-leave-to {
  opacity: 0;
  transform: scale(0.9);
}

.capsule-list-leave-active {
  position: absolute;
  width: 100%;
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  gap: 24px;
  border-radius: 32px;
  border: 2px dashed var(--capsule-border);
  background: linear-gradient(180deg, var(--capsule-bg) 0%, transparent 100%);
  backdrop-filter: blur(10px);
}

.empty-icon {
  width: 80px;
  height: 80px;
  color: var(--capsule-text-secondary);
  opacity: 0.4;
  animation: float-empty 6s ease-in-out infinite;
}

@keyframes float-empty {
  0%,
  100% {
    transform: translateY(0);
  }

  50% {
    transform: translateY(-10px);
  }
}

.empty-icon svg {
  width: 100%;
  height: 100%;
}

.empty-text {
  margin: 0;
  font-size: 1rem;
  font-weight: 500;
  color: var(--capsule-text-secondary);
  text-align: center;
  letter-spacing: 0.02em;
}

/* Responsive adjustments */
@media (max-width: 600px) {
  .levitating-provider-list {
    padding-left: 20px;
  }
}
</style>
