<!--
[INPUT]:
  source: ../../services/logs.ts ([POS]: 日志 API)
  source: ../common/BaseButton.vue ([POS]: 通用按钮)
[OUTPUT]:
  - 日志列表表格
  - 统计折线图
[POS]: 日志页面主组件
[PROTOCOL]: FractalFlow v1.0 - 分形自洽
-->
<template>
  <div class="main-shell logs-shell">
    <div class="global-actions">
      <button
        class="ghost-icon"
        type="button"
        :aria-label="t('components.logs.back')"
        :title="t('components.logs.back')"
        @click="backToHome"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M15 18l-6-6 6-6"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
      <button
        class="ghost-icon"
        type="button"
        :aria-label="t('components.logs.refresh')"
        :title="`${t('components.logs.refresh')} (${countdown}s)`"
        :disabled="loading"
        @click="manualRefresh"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M20.5 8a8.5 8.5 0 10-2.38 7.41"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M20.5 4v4h-4"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    </div>

    <div class="contrib-page logs-page">
      <header class="logs-hero">
        <div class="logs-hero-text">
          <h1>{{ t('components.logs.title') }}</h1>
          <p class="logs-hero-lead">
            {{ t('components.logs.subtitle') }}
          </p>
        </div>
      </header>

      <section class="logs-summary" v-if="statsCards.length">
        <article v-for="card in statsCards" :key="card.key" class="summary-card">
          <div class="summary-card__label">{{ card.label }}</div>
          <div class="summary-card__value">{{ card.value }}</div>
          <div class="summary-card__hint">{{ card.hint }}</div>
        </article>
      </section>

      <section class="logs-chart">
        <Line :data="chartData" :options="chartOptions" />
      </section>

      <div class="logs-filter-row">
        <div class="filter-fields">
          <label class="filter-field">
            <span>{{ t('components.logs.filters.timeRange') }}</span>
            <select v-model="filters.days" class="mac-select" @change="applyFilters">
              <option :value="1">{{ t('components.logs.filters.last24Hours') }}</option>
              <option :value="7">{{ t('components.logs.filters.last7Days') }}</option>
              <option :value="30">{{ t('components.logs.filters.last30Days') }}</option>
            </select>
          </label>
          <label class="filter-field">
            <span>{{ t('components.logs.filters.platform') }}</span>
            <select v-model="filters.platform" class="mac-select" @change="handlePlatformChange">
              <option value="">{{ t('components.logs.filters.allPlatforms') }}</option>
              <option value="claude">Claude</option>
              <option value="codex">Codex</option>
            </select>
          </label>
          <label class="filter-field">
            <span>{{ t('components.logs.filters.provider') }}</span>
            <select v-model="filters.provider" class="mac-select" @change="applyFilters">
              <option value="">{{ t('components.logs.filters.allProviders') }}</option>
              <option v-for="provider in providerOptions" :key="provider" :value="provider">
                {{ provider }}
              </option>
            </select>
          </label>
        </div>
      </div>

      <section class="logs-table-wrapper">
        <table class="logs-table">
          <thead>
            <tr>
              <th class="col-time">{{ t('components.logs.table.time') }}</th>
              <th class="col-model">{{ t('components.logs.table.model') }}</th>
              <th class="col-http">{{ t('components.logs.table.httpCode') }}</th>
              <th class="col-stream">{{ t('components.logs.table.stream') }}</th>
              <th class="col-duration">{{ t('components.logs.table.duration') }}</th>
              <th class="col-tokens">{{ t('components.logs.table.tokens') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in pagedLogs" :key="item.id">
              <td>{{ formatTime(item.created_at) }}</td>
              <td>
                <div class="model-cell">
                  <div class="model-name">{{ item.model || '—' }}</div>
                  <div class="model-meta">
                    <span class="capitalize">{{ item.provider || '—' }}</span>
                    <span class="sep">·</span>
                    <span class="capitalize">{{ item.platform || '—' }}</span>
                  </div>
                </div>
              </td>
              <td :class="['code', httpCodeClass(item.http_code)]">{{ item.http_code }}</td>
              <td>
                <span :class="['stream-tag', item.is_stream ? 'on' : 'off']">{{ formatStream(item.is_stream) }}</span>
              </td>
              <td>
                <span :class="['duration-tag', durationColor(item.duration_sec)]">{{
                  formatDuration(item.duration_sec)
                }}</span>
              </td>
              <td class="token-cell">
                <div class="token-total-wrapper">
                  <span class="token-total">{{
                    formatNumber((item.input_tokens || 0) + (item.output_tokens || 0))
                  }}</span>

                  <!-- Hover Detail Popup -->
                  <div class="token-popup">
                    <div class="popup-row">
                      <span class="popup-label">{{ t('components.logs.tokenLabels.input') }}</span>
                      <span class="popup-value">{{ formatNumber(item.input_tokens) }}</span>
                    </div>
                    <div class="popup-row">
                      <span class="popup-label">{{ t('components.logs.tokenLabels.output') }}</span>
                      <span class="popup-value">{{ formatNumber(item.output_tokens) }}</span>
                    </div>
                    <div class="popup-divider" v-if="item.reasoning_tokens || item.cache_read_tokens"></div>
                    <div class="popup-row sub" v-if="item.reasoning_tokens">
                      <span class="popup-label">{{ t('components.logs.tokenLabels.reasoning') }}</span>
                      <span class="popup-value">{{ formatNumber(item.reasoning_tokens) }}</span>
                    </div>
                    <div class="popup-row sub" v-if="item.cache_read_tokens">
                      <span class="popup-label">{{ t('components.logs.tokenLabels.cacheRead') }}</span>
                      <span class="popup-value">{{ formatNumber(item.cache_read_tokens) }}</span>
                    </div>
                    <div class="popup-row sub" v-if="item.cache_create_tokens">
                      <span class="popup-label">{{ t('components.logs.tokenLabels.cacheWrite') }}</span>
                      <span class="popup-value">{{ formatNumber(item.cache_create_tokens) }}</span>
                    </div>
                  </div>
                </div>
              </td>
            </tr>
            <tr v-if="!pagedLogs.length && !loading">
              <td colspan="6" class="empty">{{ t('components.logs.empty') }}</td>
            </tr>
          </tbody>
        </table>
        <p v-if="loading" class="empty">{{ t('components.logs.loading') }}</p>
      </section>

      <div class="logs-pagination">
        <span>{{ page }} / {{ totalPages }}</span>
        <div class="pagination-actions">
          <BaseButton variant="outline" size="sm" :disabled="page === 1 || loading" @click="prevPage"> ‹ </BaseButton>
          <BaseButton variant="outline" size="sm" :disabled="page >= totalPages || loading" @click="nextPage">
            ›
          </BaseButton>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, onMounted, watch, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import BaseButton from '../common/BaseButton.vue';
import {
  fetchRequestLogs,
  fetchLogProviders,
  fetchLogStats,
  type RequestLog,
  type LogStats,
  type LogStatsSeries,
} from '../../services/logs';
import { Chart, CategoryScale, LinearScale, PointElement, LineElement, Tooltip, Legend } from 'chart.js';
import type { ChartOptions } from 'chart.js';
import { Line } from 'vue-chartjs';
import { useThemeTokens } from '../../composables/useThemeTokens';

Chart.register(CategoryScale, LinearScale, PointElement, LineElement, Tooltip, Legend);

const { t } = useI18n();
const router = useRouter();

const logs = ref<RequestLog[]>([]);
const stats = ref<LogStats | null>(null);
const loading = ref(false);
const filters = reactive({ platform: '', provider: '', days: 1 });
const page = ref(1);
const PAGE_SIZE = 5;
const providerOptions = ref<string[]>([]);
const statsSeries = computed<LogStatsSeries[]>(() => stats.value?.series ?? []);

const isBrowser = typeof window !== 'undefined' && typeof document !== 'undefined';

const chartTokenNames = [
  '--mac-text',
  '--mac-text-secondary',
  '--chart-grid',
  '--chart-axis',
  '--chart-axis-strong',
  '--chart-cost-line',
  '--chart-cost-fill',
  '--chart-input-line',
  '--chart-input-fill',
  '--chart-output-line',
  '--chart-output-fill',
  '--chart-reasoning-line',
  '--chart-reasoning-fill',
  '--chart-cache-write-line',
  '--chart-cache-write-fill',
  '--chart-cache-read-line',
  '--chart-cache-read-fill',
];
const { tokens: themeTokens } = useThemeTokens(chartTokenNames);

const chartPalette = computed(() => {
  const theme = themeTokens.value;
  const color = (name: string, fallback: string) => theme[name] || fallback;
  return {
    cost: {
      line: color('--chart-cost-line', '#f97316'),
      fill: color('--chart-cost-fill', 'rgba(249, 115, 22, 0.2)'),
    },
    input: {
      line: color('--chart-input-line', '#34d399'),
      fill: color('--chart-input-fill', 'rgba(52, 211, 153, 0.25)'),
    },
    output: {
      line: color('--chart-output-line', '#60a5fa'),
      fill: color('--chart-output-fill', 'rgba(96, 165, 250, 0.2)'),
    },
    reasoning: {
      line: color('--chart-reasoning-line', '#f472b6'),
      fill: color('--chart-reasoning-fill', 'rgba(244, 114, 182, 0.2)'),
    },
    cacheWrite: {
      line: color('--chart-cache-write-line', '#fbbf24'),
      fill: color('--chart-cache-write-fill', 'rgba(251, 191, 36, 0.2)'),
    },
    cacheRead: {
      line: color('--chart-cache-read-line', '#38bdf8'),
      fill: color('--chart-cache-read-fill', 'rgba(56, 189, 248, 0.15)'),
    },
  };
});

const chartAxisColors = computed(() => {
  const theme = themeTokens.value;
  return {
    legend: theme['--mac-text'] || '#0f172a',
    axis: theme['--chart-axis'] || 'rgba(148, 163, 184, 0.75)',
    axisStrong: theme['--chart-axis-strong'] || '#475569',
    grid: theme['--chart-grid'] || 'rgba(148, 163, 184, 0.2)',
  };
});

// For chart labels (already converted to local string by backend)
const parseLogDate = (value?: string) => {
  if (!value) return null;
  const normalize = value.replace(' ', 'T');
  // Try parsing as local time first (since backend sends local time string)
  const attempts = [value, `${normalize}`];
  for (const candidate of attempts) {
    const parsed = new Date(candidate);
    if (!Number.isNaN(parsed.getTime())) {
      return parsed;
    }
  }
  return null;
};

// For raw logs (UTC string from DB)
const parseUtcDate = (value?: string) => {
  if (!value) return null;
  const normalize = value.replace(' ', 'T');
  // Valid ISO string for UTC must end with Z
  const utcString = normalize.endsWith('Z') ? normalize : `${normalize}Z`;
  const parsed = new Date(utcString);
  if (!Number.isNaN(parsed.getTime())) {
    return parsed;
  }
  return null;
};

const getLast24Hours = () => {
  const hours: string[] = [];
  const now = new Date();
  now.setMinutes(0, 0, 0); // Align to current hour

  for (let i = 23; i >= 0; i--) {
    const d = new Date(now);
    d.setHours(d.getHours() - i);
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    const hour = String(d.getHours()).padStart(2, '0');
    hours.push(`${year}-${month}-${day} ${hour}:00`);
  }
  return hours;
};

const getLastDays = (days: number) => {
  const dates: string[] = [];
  for (let i = days - 1; i >= 0; i--) {
    const d = new Date();
    d.setDate(d.getDate() - i);
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    dates.push(`${year}-${month}-${day}`);
  }
  return dates;
};

const chartData = computed(() => {
  const rawSeries = statsSeries.value;
  const seriesMap = new Map(rawSeries.map(item => [item.day, item]));

  // Detect granularity based on data, fallback to filter setting if no data
  const hasData = rawSeries.length > 0;
  const isDataHourly = hasData ? rawSeries[0].day.length > 10 : filters.days <= 1;

  const labels = isDataHourly ? getLast24Hours() : getLastDays(filters.days);
  const filledSeries = labels.map(date => {
    return (
      seriesMap.get(date) || {
        day: date,
        total_requests: 0,
        input_tokens: 0,
        output_tokens: 0,
        reasoning_tokens: 0,
        cache_create_tokens: 0,
        cache_read_tokens: 0,
        total_cost: 0,
      }
    );
  });

  const palette = chartPalette.value;
  return {
    labels: labels.map(d => formatSeriesLabel(d)),
    datasets: [
      {
        label: t('components.logs.tokenLabels.cost'),
        data: filledSeries.map(item => Number((item.total_cost ?? 0).toFixed(4))),
        borderColor: palette.cost.line,
        backgroundColor: palette.cost.fill,
        tension: 0.4,
        fill: false,
        yAxisID: 'yCost',
        pointRadius: 0,
        pointHoverRadius: 4,
      },
      {
        label: t('components.logs.tokenLabels.input'),
        data: filledSeries.map(item => item.input_tokens ?? 0),
        borderColor: palette.input.line,
        backgroundColor: palette.input.fill,
        tension: 0.35,
        fill: true,
        pointRadius: 2,
        pointHoverRadius: 4,
      },
      {
        label: t('components.logs.tokenLabels.output'),
        data: filledSeries.map(item => item.output_tokens ?? 0),
        borderColor: palette.output.line,
        backgroundColor: palette.output.fill,
        tension: 0.4,
        fill: true,
        pointRadius: 0,
        pointHoverRadius: 4,
      },
      {
        label: t('components.logs.tokenLabels.reasoning'),
        data: filledSeries.map(item => item.reasoning_tokens ?? 0),
        borderColor: palette.reasoning.line,
        backgroundColor: palette.reasoning.fill,
        tension: 0.4,
        fill: true,
        pointRadius: 0,
        pointHoverRadius: 4,
      },
      {
        label: t('components.logs.tokenLabels.cacheWrite'),
        data: filledSeries.map(item => item.cache_create_tokens ?? 0),
        borderColor: palette.cacheWrite.line,
        backgroundColor: palette.cacheWrite.fill,
        tension: 0.4,
        fill: false,
        pointRadius: 0,
        pointHoverRadius: 4,
      },
      {
        label: t('components.logs.tokenLabels.cacheRead'),
        data: filledSeries.map(item => item.cache_read_tokens ?? 0),
        borderColor: palette.cacheRead.line,
        backgroundColor: palette.cacheRead.fill,
        tension: 0.4,
        fill: false,
        pointRadius: 0,
        pointHoverRadius: 4,
      },
    ],
  };
});

const chartOptions = computed<ChartOptions<'line'>>(() => {
  const axis = chartAxisColors.value;
  const legendColor = axis.legend;
  const axisColor = axis.axis;
  const axisStrongColor = axis.axisStrong;
  const gridColor = axis.grid;

  return {
    responsive: true,
    maintainAspectRatio: false,
    interaction: {
      mode: 'index',
      intersect: false,
    },
    plugins: {
      legend: {
        labels: {
          color: legendColor,
          font: {
            size: 12,
            weight: 500,
          },
        },
      },
    },
    scales: {
      x: {
        grid: { display: false },
        ticks: { color: axisColor },
      },
      y: {
        beginAtZero: true,
        ticks: { color: axisColor },
        grid: { color: gridColor },
      },
      yCost: {
        position: 'right',
        beginAtZero: true,
        grid: { drawOnChartArea: false },
        ticks: {
          color: axisStrongColor,
          callback: (value: string | number) => {
            const numeric = typeof value === 'number' ? value : Number(value);
            if (Number.isNaN(numeric)) return '$0';
            if (numeric >= 1) return `$${numeric.toFixed(2)}`;
            return `$${numeric.toFixed(4)}`;
          },
        },
      },
    },
  };
});
const formatSeriesLabel = (value?: string) => {
  if (!value) return '';

  if (/^\d{4}-\d{2}-\d{2}$/.test(value)) {
    const [, , month, day] = value.match(/^(\d{4})-(\d{2})-(\d{2})$/) || [];
    return `${month}-${day}`;
  }

  const parsed = parseLogDate(value);
  if (parsed) {
    return `${padHour(parsed.getHours())}:00`;
  }
  const match = value.match(/(\d{2}):(\d{2})/);
  if (match) {
    return `${match[1]}:${match[2]}`;
  }
  return value;
};

const REFRESH_INTERVAL = 30;
const countdown = ref(REFRESH_INTERVAL);
let timer: number | undefined;

const resetTimer = () => {
  countdown.value = REFRESH_INTERVAL;
};

const startCountdown = () => {
  if (!isBrowser) return;
  stopCountdown();
  timer = window.setInterval(() => {
    if (countdown.value <= 1) {
      countdown.value = REFRESH_INTERVAL;
      void loadDashboard();
    } else {
      countdown.value -= 1;
    }
  }, 1000);
};

const stopCountdown = () => {
  if (timer) {
    clearInterval(timer);
    timer = undefined;
  }
};

const loadLogs = async () => {
  loading.value = true;
  try {
    const data = await fetchRequestLogs({
      platform: filters.platform,
      provider: filters.provider,
      limit: 200,
    });
    logs.value = data ?? [];
    page.value = Math.min(page.value, totalPages.value);
  } catch (error) {
    console.error('failed to load request logs', error);
  } finally {
    loading.value = false;
  }
};

const loadStats = async () => {
  try {
    // Pass configured days
    const data = await fetchLogStats(filters.platform, filters.provider, filters.days);
    stats.value = data ?? null;
  } catch (error) {
    console.error('failed to load log stats', error);
  }
};

const loadDashboard = async () => {
  await Promise.all([loadLogs(), loadStats()]);
};

const pagedLogs = computed(() => {
  const start = (page.value - 1) * PAGE_SIZE;
  return logs.value.slice(start, start + PAGE_SIZE);
});

const totalPages = computed(() => Math.max(1, Math.ceil(logs.value.length / PAGE_SIZE)));

const applyFilters = async () => {
  page.value = 1;
  await loadDashboard();
  resetTimer();
};

const manualRefresh = () => {
  resetTimer();
  void loadDashboard();
};

const nextPage = () => {
  if (page.value < totalPages.value) {
    page.value += 1;
  }
};

const prevPage = () => {
  if (page.value > 1) {
    page.value -= 1;
  }
};

const backToHome = () => {
  router.push('/');
};

const handlePlatformChange = () => {
  filters.provider = '';
  applyFilters();
};

const padHour = (num: number) => num.toString().padStart(2, '0');

const formatTime = (value?: string) => {
  const date = parseUtcDate(value);
  if (!date) return value || '—';
  return `${date.getFullYear()}-${padHour(date.getMonth() + 1)}-${padHour(date.getDate())} ${padHour(date.getHours())}:${padHour(date.getMinutes())}:${padHour(date.getSeconds())}`;
};

const formatStream = (value?: boolean | number) => {
  const isOn = value === true || value === 1;
  return isOn ? t('components.logs.streamOn') : t('components.logs.streamOff');
};

const formatDuration = (value?: number) => {
  if (!value || Number.isNaN(value)) return '—';
  return `${value.toFixed(2)}s`;
};

const httpCodeClass = (code: number) => {
  if (code >= 500) return 'http-server-error';
  if (code >= 400) return 'http-client-error';
  if (code >= 300) return 'http-redirect';
  if (code >= 200) return 'http-success';
  return 'http-info';
};

const durationColor = (value?: number) => {
  if (!value || Number.isNaN(value)) return 'neutral';
  if (value < 2) return 'fast';
  if (value < 5) return 'medium';
  return 'slow';
};

const formatNumber = (value?: number) => {
  if (value === undefined || value === null) return '—';
  return value.toLocaleString();
};

const formatCurrency = (value?: number) => {
  if (value === undefined || value === null || Number.isNaN(value)) {
    return '$0.0000';
  }
  if (value >= 1) {
    return `$${value.toFixed(2)}`;
  }
  if (value >= 0.01) {
    return `$${value.toFixed(3)}`;
  }
  return `$${value.toFixed(4)}`;
};

const startOfTodayLocal = () => {
  const now = new Date();
  now.setHours(0, 0, 0, 0);
  return now;
};

const statsCards = computed(() => {
  const data = stats.value;
  const summaryDate = summaryDateLabel.value;
  /* const summaryDate = summaryDateLabel.value; */
  const totalTokens = (data?.input_tokens ?? 0) + (data?.output_tokens ?? 0);

  let costHint = '';
  if (filters.days <= 1) {
    costHint = summaryDate ? t('components.logs.summary.todayScope', { date: summaryDate }) : '';
  } else if (filters.days === 7) {
    costHint = t('components.logs.filters.last7Days');
  } else if (filters.days === 30) {
    costHint = t('components.logs.filters.last30Days');
  } else {
    costHint = `${filters.days} Days`;
  }

  return [
    {
      key: 'requests',
      label: t('components.logs.summary.total'),
      hint: t('components.logs.summary.requests'),
      value: data ? formatNumber(data.total_requests) : '—',
    },
    {
      key: 'tokens',
      label: t('components.logs.summary.tokens'),
      hint: t('components.logs.summary.tokenHint'),
      value: data ? formatNumber(totalTokens) : '—',
    },
    {
      key: 'cacheReads',
      label: t('components.logs.summary.cache'),
      hint: t('components.logs.summary.cacheHint'),
      value: data ? formatNumber(data.cache_read_tokens) : '—',
    },
    {
      key: 'cost',
      label: t('components.logs.tokenLabels.cost'),
      hint: costHint,
      value: formatCurrency(data?.cost_total ?? 0),
    },
  ];
});

const summaryDateLabel = computed(() => {
  const firstBucket = statsSeries.value.find(item => item.day);
  const parsed = parseLogDate(firstBucket?.day ?? '');
  const date = parsed ?? startOfTodayLocal();
  return `${date.getFullYear()}-${padHour(date.getMonth() + 1)}-${padHour(date.getDate())}`;
});

const loadProviderOptions = async () => {
  try {
    const list = await fetchLogProviders(filters.platform);
    providerOptions.value = list ?? [];
    if (filters.provider && !providerOptions.value.includes(filters.provider)) {
      filters.provider = '';
    }
  } catch (error) {
    console.error('failed to load provider options', error);
  }
};

watch(
  () => filters.platform,
  async () => {
    await loadProviderOptions();
  }
);

onMounted(async () => {
  await Promise.all([loadDashboard(), loadProviderOptions()]);
  startCountdown();
});

onUnmounted(() => {
  stopCountdown();
});
</script>

<style scoped>
.logs-hero {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 24px;
  flex-wrap: wrap;
}

.logs-hero-text h1 {
  font-size: clamp(2rem, 4vw, 2.75rem);
  margin: 0;
  color: var(--mac-text);
}

.logs-hero-lead {
  margin: 12px 0 0;
  max-width: 540px;
  color: var(--mac-text-secondary);
  line-height: 1.6;
}

.logs-hero-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 12px;
  color: var(--mac-text-secondary);
}

.logs-summary {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.summary-card {
  border: 1px solid var(--surface-card-border);
  border-radius: 20px;
  padding: 1rem 1.25rem;
  background: var(--surface-card-bg);
  box-shadow: var(--surface-card-shadow);
  backdrop-filter: blur(24px);
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.summary-card__label {
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--surface-card-label);
}

.summary-card__value {
  font-size: 1.85rem;
  font-weight: 600;
  color: var(--surface-card-value);
}

.summary-card__hint {
  font-size: 0.85rem;
  color: var(--surface-card-hint);
}

@media (max-width: 768px) {
  .logs-summary {
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  }

  .logs-hero-meta {
    width: 100%;
    align-items: flex-start;
  }
}

.model-cell {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 2px;
}

.model-name {
  font-weight: 600;
  font-size: 0.9em;
  color: var(--mac-text);
  letter-spacing: -0.01em;
}

.model-meta {
  font-size: 0.75em;
  color: var(--mac-text-secondary);
  display: flex;
  align-items: center;
  gap: 4px;
}

.model-meta .sep {
  opacity: 0.4;
  font-weight: 400;
}

.capitalize {
  text-transform: capitalize;
}
</style>
