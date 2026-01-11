<template>
  <div class="main-shell">
    <!-- 胶囊导航组件 - 底部悬浮 -->
    <CapsuleNavigation
      :view-mode="activeTab"
      :proxy-enabled="activeProxyState"
      :proxy-loading="activeProxyBusy"
      @update:view-mode="selectedIndex = tabs.findIndex(t => t.id === $event)"
      @toggle-proxy="onProxyToggle"
      @add="openCreateModal"
    />

    <div class="global-actions"></div>
    <InlineNotification :visible="notificationState.visible" :message="notificationState.message" />
    <div class="contrib-page">
      <section class="contrib-hero">
        <p class="eyebrow">{{ t('components.main.hero.eyebrow') }}</p>
      </section>

      <section
        v-if="showHeatmap"
        ref="heatmapContainerRef"
        class="contrib-wall"
        :aria-label="t('components.main.heatmap.ariaLabel')"
      >
        <div class="contrib-legend">
          <span>{{ t('components.main.heatmap.legendLow') }}</span>
          <span v-for="level in 5" :key="level" :class="['legend-box', intensityClass(level - 1)]" />
          <span>{{ t('components.main.heatmap.legendHigh') }}</span>
        </div>

        <div class="contrib-grid">
          <div v-for="(week, weekIndex) in usageHeatmap" :key="weekIndex" class="contrib-column">
            <div
              v-for="(day, dayIndex) in week"
              :key="dayIndex"
              class="contrib-cell"
              :class="intensityClass(day.intensity)"
              @mouseenter="showUsageTooltip(day, $event)"
              @mousemove="showUsageTooltip(day, $event)"
              @mouseleave="hideUsageTooltip"
            />
          </div>
        </div>
        <div
          v-if="usageTooltip.visible"
          ref="tooltipRef"
          class="contrib-tooltip"
          :class="usageTooltip.placement"
          :style="{ left: `${usageTooltip.left}px`, top: `${usageTooltip.top}px` }"
        >
          <p class="tooltip-heading">{{ formattedTooltipLabel }}</p>
          <ul class="tooltip-metrics">
            <li v-for="metric in usageTooltipMetrics" :key="metric.key">
              <span class="metric-label">{{ metric.label }}</span>
              <span class="metric-value">{{ metric.value }}</span>
            </li>
          </ul>
        </div>
      </section>

      <section class="automation-section">
        <!-- 悬浮胶囊列表 - 替换旧的 automation-list -->
        <LevitatingProviderList
          :providers="activeCards"
          :stats-map="capsuleStatsMap"
          :active-id="smartActiveId"
          :active-status-type="activeStatusType"
          @reorder="onCapsuleReorder"
          @toggle-enabled="onCapsuleToggleEnabled"
          @configure="configure"
          @remove="requestRemove"
        />
      </section>

      <VendorReceptacle
        :open="modalState.open"
        :initial-tab="activeTab"
        :initial-data="editingCard"
        @close="closeModal"
        @submit="handleReceptacleSubmit"
      />
      <BaseModal
        :open="confirmState.open"
        :title="t('components.main.form.confirmDeleteTitle')"
        variant="confirm"
        @close="closeConfirm"
      >
        <div class="confirm-body">
          <p>
            {{ t('components.main.form.confirmDeleteMessage', { name: confirmState.card?.name ?? '' }) }}
          </p>
        </div>
        <footer class="form-actions confirm-actions">
          <BaseButton variant="outline" type="button" @click="closeConfirm">
            {{ t('components.main.form.actions.cancel') }}
          </BaseButton>
          <BaseButton variant="danger" type="button" @click="confirmRemove">
            {{ t('components.main.form.actions.delete') }}
          </BaseButton>
        </footer>
      </BaseModal>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
---
[INPUT]: source: openspec/specs/capsule-nav/spec.md ([POS]: 胶囊导航功能规范)
[OUTPUT]: 移动 Eyebrow 至 Hero 区域
[PROTOCOL]: FractalFlow v1.0
[POS]: 主页入口组件 (Main Page)
---
*/
import { computed, reactive, ref, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import {
  buildUsageHeatmapMatrix,
  generateFallbackUsageHeatmap,
  DEFAULT_HEATMAP_DAYS,
  calculateHeatmapDayRange,
  type UsageHeatmapWeek,
  type UsageHeatmapDay,
} from '../../data/usageHeatmap';
import { automationCardGroups, createAutomationCards, type AutomationCard } from '../../data/cards';
import lobeIcons from '../../icons/lobeIconMap';
import BaseButton from '../common/BaseButton.vue';
import BaseModal from '../common/BaseModal.vue';
import CapsuleNavigation from './CapsuleNavigation.vue';
import LevitatingProviderList from './LevitatingProviderList.vue';
import VendorReceptacle from './VendorReceptacle.vue';
import InlineNotification from '../common/InlineNotification.vue';

import { fetchProxyStatus, enableProxy, disableProxy } from '../../services/claudeSettings';
import {
  fetchHeatmapStats,
  fetchProviderDailyStats,
  fetchRequestLogs,
  type ProviderDailyStat,
} from '../../services/logs';
import { fetchCurrentVersion } from '../../services/version';
import { fetchAppSettings, type AppSettings } from '../../services/appSettings';

const { t, locale } = useI18n();

const HEATMAP_DAYS = DEFAULT_HEATMAP_DAYS;
// 记录本次会话启动时间，用于过滤历史日志 (避免重启后仍然高亮旧的故障转移结果)
const SESSION_START = new Date();
const usageHeatmap = ref<UsageHeatmapWeek[]>(generateFallbackUsageHeatmap(HEATMAP_DAYS));
const heatmapContainerRef = ref<HTMLElement | null>(null);
const tooltipRef = ref<HTMLElement | null>(null);
const proxyStates = reactive<Record<ProviderTab, boolean>>({
  claude: false,
  codex: false,
});
const proxyBusy = reactive<Record<ProviderTab, boolean>>({
  claude: false,
  codex: false,
});

const providerStatsMap = reactive<Record<ProviderTab, Record<string, ProviderDailyStat>>>({
  claude: {},
  codex: {},
} as Record<ProviderTab, Record<string, ProviderDailyStat>>);
const providerStatsLoading = reactive<Record<ProviderTab, boolean>>({
  claude: false,
  codex: false,
} as Record<ProviderTab, boolean>);
const providerStatsLoaded = reactive<Record<ProviderTab, boolean>>({
  claude: false,
  codex: false,
} as Record<ProviderTab, boolean>);
const dominantProviderMap = reactive<Record<ProviderTab, number | null>>({
  claude: null,
  codex: null,
});
let providerStatsTimer: number | undefined;

const showHeatmap = ref(true);
const appVersion = ref('');

// Notification State
const notificationState = reactive({
  visible: false,
  message: '',
  timer: undefined as number | undefined,
});

const showNotification = (message: string) => {
  notificationState.message = message;
  notificationState.visible = true;

  if (notificationState.timer) {
    clearTimeout(notificationState.timer);
  }

  notificationState.timer = window.setTimeout(() => {
    notificationState.visible = false;
  }, 3000); // 3 seconds as requested
};

const intensityClass = (value: number) => `gh-level-${value}`;

type TooltipPlacement = 'above' | 'below';

const usageTooltip = reactive({
  visible: false,
  label: '',
  dateKey: '',
  left: 0,
  top: 0,
  placement: 'above' as TooltipPlacement,
  requests: 0,
  inputTokens: 0,
  outputTokens: 0,
  reasoningTokens: 0,
  cost: 0,
});

const formatMetric = (value: number) => value.toLocaleString();

const tooltipDateFormatter = computed(
  () =>
    new Intl.DateTimeFormat(locale.value || 'en', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
);

const currencyFormatter = computed(
  () =>
    new Intl.NumberFormat(locale.value || 'en', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })
);

const formattedTooltipLabel = computed(() => {
  if (!usageTooltip.dateKey) return usageTooltip.label;
  const date = new Date(usageTooltip.dateKey);
  if (Number.isNaN(date.getTime())) {
    return usageTooltip.label;
  }
  return tooltipDateFormatter.value.format(date);
});

const formattedTooltipAmount = computed(() => currencyFormatter.value.format(Math.max(usageTooltip.cost, 0)));

const usageTooltipMetrics = computed(() => [
  {
    key: 'cost',
    label: t('components.main.heatmap.metrics.cost'),
    value: formattedTooltipAmount.value,
  },
  {
    key: 'requests',
    label: t('components.main.heatmap.metrics.requests'),
    value: formatMetric(usageTooltip.requests),
  },
  {
    key: 'inputTokens',
    label: t('components.main.heatmap.metrics.inputTokens'),
    value: formatMetric(usageTooltip.inputTokens),
  },
  {
    key: 'outputTokens',
    label: t('components.main.heatmap.metrics.outputTokens'),
    value: formatMetric(usageTooltip.outputTokens),
  },
  {
    key: 'reasoningTokens',
    label: t('components.main.heatmap.metrics.reasoningTokens'),
    value: formatMetric(usageTooltip.reasoningTokens),
  },
]);

const clamp = (value: number, min: number, max: number) => {
  if (max <= min) return min;
  return Math.min(Math.max(value, min), max);
};

const TOOLTIP_DEFAULT_WIDTH = 220;
const TOOLTIP_DEFAULT_HEIGHT = 120;
const TOOLTIP_VERTICAL_OFFSET = 12;
const TOOLTIP_HORIZONTAL_MARGIN = 20;
const TOOLTIP_VERTICAL_MARGIN = 24;

const getTooltipSize = () => {
  const rect = tooltipRef.value?.getBoundingClientRect();
  return {
    width: rect?.width ?? TOOLTIP_DEFAULT_WIDTH,
    height: rect?.height ?? TOOLTIP_DEFAULT_HEIGHT,
  };
};

const viewportSize = () => {
  if (typeof window !== 'undefined') {
    return { width: window.innerWidth, height: window.innerHeight };
  }
  if (typeof document !== 'undefined' && document.documentElement) {
    return {
      width: document.documentElement.clientWidth,
      height: document.documentElement.clientHeight,
    };
  }
  return {
    width: heatmapContainerRef.value?.clientWidth ?? 0,
    height: heatmapContainerRef.value?.clientHeight ?? 0,
  };
};

const showUsageTooltip = (day: UsageHeatmapDay, event: MouseEvent) => {
  const target = event.currentTarget as HTMLElement | null;
  const cellRect = target?.getBoundingClientRect();
  if (!cellRect) return;
  usageTooltip.label = day.label;
  usageTooltip.dateKey = day.dateKey;
  usageTooltip.requests = day.requests;
  usageTooltip.inputTokens = day.inputTokens;
  usageTooltip.outputTokens = day.outputTokens;
  usageTooltip.reasoningTokens = day.reasoningTokens;
  usageTooltip.cost = day.cost;
  const { width: tooltipWidth, height: tooltipHeight } = getTooltipSize();
  const { width: viewportWidth, height: viewportHeight } = viewportSize();
  const centerX = cellRect.left + cellRect.width / 2;
  const halfWidth = tooltipWidth / 2;
  const minLeft = TOOLTIP_HORIZONTAL_MARGIN + halfWidth;
  const maxLeft = viewportWidth > 0 ? viewportWidth - halfWidth - TOOLTIP_HORIZONTAL_MARGIN : centerX;
  usageTooltip.left = clamp(centerX, minLeft, maxLeft);

  const anchorTop = cellRect.top;
  const anchorBottom = cellRect.bottom;
  const canShowAbove = anchorTop - tooltipHeight - TOOLTIP_VERTICAL_OFFSET >= TOOLTIP_VERTICAL_MARGIN;
  const viewportBottomLimit =
    viewportHeight > 0 ? viewportHeight - tooltipHeight - TOOLTIP_VERTICAL_MARGIN : anchorBottom;
  const shouldPlaceBelow = !canShowAbove;
  usageTooltip.placement = shouldPlaceBelow ? 'below' : 'above';
  const desiredTop = shouldPlaceBelow
    ? anchorBottom + TOOLTIP_VERTICAL_OFFSET
    : anchorTop - tooltipHeight - TOOLTIP_VERTICAL_OFFSET;
  usageTooltip.top = clamp(desiredTop, TOOLTIP_VERTICAL_MARGIN, viewportBottomLimit);
  usageTooltip.visible = true;
};

const hideUsageTooltip = () => {
  usageTooltip.visible = false;
};

const loadAppSettings = async () => {
  try {
    const data: AppSettings = await fetchAppSettings();
    showHeatmap.value = data?.show_heatmap ?? true;
  } catch (error) {
    console.error('failed to load app settings', error);
    showHeatmap.value = true;
  }
};

const loadAppVersion = async () => {
  try {
    const version = await fetchCurrentVersion();
    appVersion.value = version || '';
  } catch (error) {
    console.error('failed to load app version', error);
  }
};

const handleAppSettingsUpdated = () => {
  void loadAppSettings();
};

const normalizeProviderKey = (value: string) => value?.trim().toLowerCase() ?? '';

const loadUsageHeatmap = async () => {
  try {
    const rangeDays = calculateHeatmapDayRange(HEATMAP_DAYS);
    const stats = await fetchHeatmapStats(rangeDays);
    usageHeatmap.value = buildUsageHeatmapMatrix(stats, HEATMAP_DAYS);
  } catch (error) {
    console.error('Failed to load usage heatmap stats', error);
  }
};

const tabs = [
  { id: 'claude', label: 'Claude Code' },
  { id: 'codex', label: 'Codex' },
] as const;
type ProviderTab = (typeof tabs)[number]['id'];
const providerTabIds = tabs.map(tab => tab.id) as ProviderTab[];

const cards = reactive<Record<ProviderTab, AutomationCard[]>>({
  claude: createAutomationCards(automationCardGroups.claude),
  codex: createAutomationCards(automationCardGroups.codex),
});

// Bug fix: 根据数组索引更新 level 字段，确保排序生效
// 公式: level = index + 1（索引 0 → level 1，索引 1 → level 2，以此类推）
const serializeProviders = (providers: AutomationCard[]) =>
  providers.map((provider, index) => ({
    ...provider,
    level: index + 1, // 优先级：数字越小越优先
  }));

const persistProviders = async (tabId: ProviderTab) => {
  // 获取带有正确 level 的列表
  const updatedProviders = serializeProviders(cards[tabId]);
  // 关键：立即更新本地状态，确保 UI 显示最新的 level，并没有破坏响应式
  // 注意 serializeProviders 返回的是新数组和新对象
  cards[tabId] = updatedProviders;

  try {
    await invoke('save_providers', { kind: tabId, providers: updatedProviders });
  } catch (error) {
    console.error('Failed to save providers', error);
  }
};

const replaceProviders = (tabId: ProviderTab, data: AutomationCard[]) => {
  cards[tabId].splice(0, cards[tabId].length, ...createAutomationCards(data));
};

const loadProvidersFromDisk = async () => {
  for (const tab of providerTabIds) {
    try {
      const saved = await invoke('load_providers', { kind: tab });
      if (Array.isArray(saved)) {
        replaceProviders(tab, saved as AutomationCard[]);
      } else {
        await persistProviders(tab);
      }
    } catch (error) {
      console.error('Failed to load providers', error);
    }
  }
};

const refreshProxyState = async (tab: ProviderTab) => {
  try {
    const status = await fetchProxyStatus(tab);
    proxyStates[tab] = Boolean(status);
  } catch (error) {
    console.error(`Failed to fetch proxy status for ${tab}`, error);
    proxyStates[tab] = false;
  }
};

const onProxyToggle = async () => {
  const tab = activeTab.value;
  if (proxyBusy[tab]) return;
  proxyBusy[tab] = true;
  const nextState = !proxyStates[tab];
  try {
    if (nextState) {
      await enableProxy(tab);
    } else {
      await disableProxy(tab);
    }
    proxyStates[tab] = nextState;
  } catch (error) {
    console.error(`Failed to toggle proxy for ${tab}`, error);
  } finally {
    proxyBusy[tab] = false;
  }
};

const loadProviderStats = async (tab: ProviderTab) => {
  providerStatsLoading[tab] = true;
  try {
    // 1. 获取每日统计 (用于显示)
    const stats = await fetchProviderDailyStats(1);
    const mapped: Record<string, ProviderDailyStat> = {};
    (stats ?? []).forEach(stat => {
      mapped[normalizeProviderKey(stat.provider)] = stat;
    });
    const hadExistingStats = Object.keys(providerStatsMap[tab] ?? {}).length > 0;
    if ((stats?.length ?? 0) > 0) {
      providerStatsMap[tab] = mapped;
    } else if (!hadExistingStats) {
      providerStatsMap[tab] = mapped;
    }

    // 2. [Logic V2] 基于最近20条日志判断 Dominant Provider
    // "按供应商分类，从最近的20条日志中统计，哪个占比最高"
    try {
      const recentLogs = await fetchRequestLogs({ platform: tab, limit: 20 });
      // 1. 仅统计本次会话产生的日志 (重启即重置状态，符合用户直觉)
      // 2. 仅统计成功请求 (http_code = 200)
      const validLogs = recentLogs.filter(l => {
        const logTime = new Date(l.created_at);
        return logTime >= SESSION_START && l.http_code === 200;
      });

      if (validLogs.length > 0) {
        const counts: Record<string, number> = {};
        for (const log of validLogs) {
          const key = normalizeProviderKey(log.provider);
          counts[key] = (counts[key] || 0) + 1;
        }

        let maxCount = -1;
        let maxKey = '';
        for (const [key, count] of Object.entries(counts)) {
          if (count > maxCount) {
            maxCount = count;
            maxKey = key;
          }
        }

        // 映射回 Card ID
        const tabCards = cards[tab] || [];
        const dominantCard = tabCards.find(c => normalizeProviderKey(c.name) === maxKey);

        // Check for Auto-Switch Event
        const newDomId = dominantCard ? dominantCard.id : null;
        const oldDomId = dominantProviderMap[tab];

        if (newDomId && oldDomId && newDomId !== oldDomId && tab === activeTab.value) {
          const oldCard = tabCards.find(c => c.id === oldDomId);
          const newCard = dominantCard;
          if (oldCard && newCard) {
            showNotification(t('components.main.notification.autoSwitch', { old: oldCard.name, new: newCard.name }));
          }
        }

        dominantProviderMap[tab] = newDomId;
      } else {
        dominantProviderMap[tab] = null; // 无近期成功记录
      }
    } catch (e) {
      console.error('Failed to fetch recent logs', e);
    }

    providerStatsLoaded[tab] = true;
  } catch (error) {
    console.error(`Failed to load provider stats for ${tab}`, error);
    if (!providerStatsLoaded[tab]) {
      providerStatsLoaded[tab] = true;
    }
  } finally {
    providerStatsLoading[tab] = false;
  }
};

const startProviderStatsTimer = () => {
  stopProviderStatsTimer();
  // 缩短轮询间隔到 5s 以提供更快的故障切换视觉反馈
  providerStatsTimer = window.setInterval(() => {
    providerTabIds.forEach(tab => {
      void loadProviderStats(tab);
    });
  }, 5_000);
};

const stopProviderStatsTimer = () => {
  if (providerStatsTimer) {
    clearInterval(providerStatsTimer);
    providerStatsTimer = undefined;
  }
};

onMounted(async () => {
  void loadUsageHeatmap();
  await loadProvidersFromDisk();
  await Promise.all(providerTabIds.map(refreshProxyState));
  await Promise.all(providerTabIds.map(tab => loadProviderStats(tab)));
  await loadAppSettings();
  await loadAppVersion();
  startProviderStatsTimer();
  window.addEventListener('app-settings-updated', handleAppSettingsUpdated);
});

onUnmounted(() => {
  stopProviderStatsTimer();
  window.removeEventListener('app-settings-updated', handleAppSettingsUpdated);
});

const selectedIndex = ref(0);
const activeTab = computed<ProviderTab>(() => tabs[selectedIndex.value]?.id ?? tabs[0].id);
const activeCards = computed(() => cards[activeTab.value] ?? []);

const activeProxyState = computed(() => proxyStates[activeTab.value]);
const activeProxyBusy = computed(() => proxyBusy[activeTab.value]);

// === Levitating Capsule 组件支持 ===

// 将 providerStatsMap 转换为 LevitatingProviderList 需要的格式
interface CapsuleStats {
  successRate: number;
  requests: number;
  tokens: number;
  cost: number;
  hourlyRequests: number[];
}

const capsuleStatsMap = computed<Record<string, CapsuleStats>>(() => {
  const tab = activeTab.value;
  const rawMap = providerStatsMap[tab] ?? {};
  const result: Record<string, CapsuleStats> = {};

  for (const [key, stat] of Object.entries(rawMap)) {
    result[key] = {
      successRate: Number.isFinite(stat.success_rate) ? clamp(stat.success_rate, 0, 1) : 0,
      requests: stat.total_requests ?? 0,
      tokens: (stat.input_tokens ?? 0) + (stat.output_tokens ?? 0),
      cost: stat.cost_total ?? 0,
      hourlyRequests: stat.hourly_requests ?? [],
    };
  }

  return result;
});

// 智能判断当前“生效”的 Provider ID
// 逻辑: 优先选择启用的、且成功率尚可 (> 20%) 的第一个 Provider
// 如果全部失败，则 fallback 到第一个启用的 Provider (即使它也挂了)
const smartActiveId = computed(() => {
  const list = activeCards.value;
  if (!list?.length) return null;

  // Priority 1: Dominant Provider (Based on Recent History)
  const tab = activeTab.value;
  const domId = dominantProviderMap[tab];
  if (domId && list.find(c => c.id === domId)) {
    return domId;
  }

  // Priority 2: Health Check Override (Removed)
  // 用户要求重启后强制重置为第一个，不依赖历史统计。
  // 只有当 Logic 1 (本次会话的成功记录) 明确指向其他 Provider 时才转移。

  // 3. Visual Fallback
  // 默认选中第一个启用的 Provider
  const firstEnabled = list.find(c => c.enabled);
  return firstEnabled ? firstEnabled.id : list[0].id;
});

const activeStatusType = computed<'auto' | 'manual' | null>(() => {
  const domId = dominantProviderMap[activeTab.value];
  if (smartActiveId.value && domId && smartActiveId.value === domId) {
    return 'auto';
  }
  // Fallback or explicit preference implies manual/default
  return 'manual';
});

// 处理拖拽排序
const onCapsuleReorder = (reorderedProviders: AutomationCard[]) => {
  const currentTab = activeTab.value;
  // 更新本地状态
  cards[currentTab] = reorderedProviders;
  // 持久化到后端
  void persistProviders(currentTab);
};

// 处理启用/禁用切换
const onCapsuleToggleEnabled = (provider: AutomationCard, enabled: boolean) => {
  provider.enabled = enabled;
  void persistProviders(activeTab.value);
};
type VendorForm = {
  name: string;
  apiUrl: string;
  apiKey: string;
  officialSite: string;
  icon: string;
  enabled: boolean;
  supportedModels?: Record<string, boolean>;
  modelMapping?: Record<string, string>;
};

const iconOptions = Object.keys(lobeIcons).sort((a, b) => a.localeCompare(b));
const defaultIconKey = iconOptions[0] ?? 'aicoding';

const defaultFormValues = (): VendorForm => ({
  name: '',
  apiUrl: '',
  apiKey: '',
  officialSite: '',
  icon: defaultIconKey,
  enabled: true,
  supportedModels: {},
  modelMapping: {},
});

const modalState = reactive({
  open: false,
  tabId: tabs[0].id as ProviderTab,
  editingId: null as number | null,
  form: defaultFormValues(),
  errors: {
    apiUrl: '',
  },
});

const editingCard = ref<AutomationCard | null>(null);
const confirmState = reactive({ open: false, card: null as AutomationCard | null, tabId: tabs[0].id as ProviderTab });

const openCreateModal = () => {
  modalState.tabId = activeTab.value;
  modalState.editingId = null;
  editingCard.value = null;
  Object.assign(modalState.form, defaultFormValues());
  modalState.errors.apiUrl = '';
  modalState.open = true;
};

const openEditModal = (card: AutomationCard) => {
  modalState.tabId = activeTab.value;
  modalState.editingId = card.id;
  editingCard.value = card;
  Object.assign(modalState.form, {
    name: card.name,
    apiUrl: card.apiUrl,
    apiKey: card.apiKey,
    officialSite: card.officialSite,
    icon: card.icon,
    enabled: card.enabled,
    supportedModels: card.supportedModels || {},
    modelMapping: card.modelMapping || {},
  });
  modalState.errors.apiUrl = '';
  modalState.open = true;
};

const closeModal = () => {
  modalState.open = false;
};

const closeConfirm = () => {
  confirmState.open = false;
  confirmState.card = null;
};

const handleReceptacleSubmit = (data: any) => {
  const list = cards[modalState.tabId];
  if (!list) return;

  if (editingCard.value) {
    Object.assign(editingCard.value, {
      name: data.name,
      apiUrl: data.apiUrl,
      apiKey: data.apiKey,
      icon: data.icon,
      enabled: data.enabled,
      modelMapping: data.modelMapping || {},
      // Keep existing fields if not in data
      officialSite: editingCard.value.officialSite,
      supportedModels: editingCard.value.supportedModels,
    });
    void persistProviders(modalState.tabId);
  } else {
    // Create new
    const newCard: AutomationCard = {
      id: Date.now(),
      name: data.name,
      apiUrl: data.apiUrl,
      apiKey: data.apiKey,
      officialSite: '',
      icon: data.icon,
      accent: '#0a84ff',
      tint: 'rgba(15, 23, 42, 0.12)',
      enabled: data.enabled,
      supportedModels: {},
      modelMapping: data.modelMapping || {},
    };
    list.push(newCard);
    void persistProviders(modalState.tabId);
  }
  closeModal();
};

const configure = (card: AutomationCard) => {
  openEditModal(card);
};

const remove = (id: number, tabId: ProviderTab = activeTab.value) => {
  const list = cards[tabId];
  if (!list) return;
  const index = list.findIndex(card => card.id === id);
  if (index > -1) {
    list.splice(index, 1);
    void persistProviders(tabId);
  }
};

// 获取卡片在当前列表中的索引（用于判断是否禁用按钮）

const requestRemove = (card: AutomationCard) => {
  confirmState.card = card;
  confirmState.tabId = activeTab.value;
  confirmState.open = true;
};

const confirmRemove = () => {
  if (!confirmState.card) return;
  remove(confirmState.card.id, confirmState.tabId);
  closeConfirm();
};
</script>

<style scoped>
.main-version {
  margin: 32px auto 12px;
  text-align: center;
  color: var(--mac-text-secondary);
  font-size: 0.85rem;
}

/* 拖拽时禁止选中文本 */
.automation-card {
  user-select: none;
  -webkit-user-select: none;
}
</style>
