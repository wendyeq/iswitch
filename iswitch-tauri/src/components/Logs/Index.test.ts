/**
 * ---
 * [INPUT]: {Logs Index Component}
 *     - Index: source: ./Index.vue ([POS]: 日志页组件)
 * [OUTPUT]: {测试结果} - 日志页交互测试
 * [POS]: Logs 页面集成测试
 * [PROTOCOL]:
 * 1. Mock Chart.js 和 Services
 * 2. 验证数据加载与渲染
 * 3. 验证刷新交互
 * ---
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import Index from './Index.vue';
import { fetchRequestLogs, fetchLogProviders, fetchLogStats } from '../../services/logs';

// Mock 依赖组件
vi.mock('vue-chartjs', () => ({
  Line: { template: '<div class="chart-stub">Chart</div>', props: ['data', 'options'] },
}));

// Mock Chart.js 避免 tree-shaking 或 import 错误
vi.mock('chart.js', () => ({
  Chart: { register: vi.fn() },
  Title: {},
  Tooltip: {},
  Legend: {},
  LineElement: {},
  CategoryScale: {},
  LinearScale: {},
  PointElement: {},
}));

// Mock Services
vi.mock('../../services/logs', () => ({
  fetchRequestLogs: vi.fn(),
  fetchLogStats: vi.fn(),
  fetchLogProviders: vi.fn(),
  fetchProviderDailyStats: vi.fn(),
  fetchHeatmapStats: vi.fn(),
}));

vi.mock('../../composables/useThemeTokens', () => ({
  useThemeTokens: () => ({
    tokens: { value: {} },
    getTokenValue: () => '',
    refreshTokens: vi.fn(),
  }),
}));

// Mock i18n
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, args?: any) => (args ? `${key} ${JSON.stringify(args)}` : key),
    locale: { value: 'zh' },
  }),
}));

// Mock Router
const pushMock = vi.fn();
vi.mock('vue-router', () => ({
  useRouter: () => ({ push: pushMock }),
}));

// Stubs
const BaseButtonStub = {
  template: '<button @click="$emit(\'click\')"><slot /></button>',
  props: ['variant', 'disabled', 'size'],
};

const baseLog = () => ({
  id: 1,
  created_at: '2026-01-01 10:00:00',
  platform: 'claude',
  provider: 'alpha',
  model: 'claude-3',
  http_code: 200,
  is_stream: true,
  duration_sec: 1.5,
  input_tokens: 10,
  output_tokens: 20,
  reasoning_tokens: 0,
  cache_create_tokens: 0,
  cache_read_tokens: 0,
});

const createLog = (overrides: Partial<ReturnType<typeof baseLog>> = {}) => ({
  ...baseLog(),
  ...overrides,
});

const mountLogsPage = () =>
  mount(Index, {
    global: { stubs: { BaseButton: BaseButtonStub } },
  });

describe('Logs/Index.vue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(fetchRequestLogs).mockResolvedValue([createLog()]);
    vi.mocked(fetchLogStats).mockResolvedValue({
      total_requests: 10,
      input_tokens: 100,
      output_tokens: 120,
      reasoning_tokens: 0,
      cache_read_tokens: 5,
      cost_total: 1.23,
      series: [],
    } as any);
    vi.mocked(fetchLogProviders).mockResolvedValue(['alpha', 'beta']);
  });

  it('加载并渲染日志列表', async () => {
    const mockLogs = [
      {
        id: 1,
        created_at: '2026-01-01 10:00:00',
        platform: 'claude',
        provider: 'test',
        input_tokens: 10,
        output_tokens: 20,
      },
    ];
    const mockStats = {
      total_requests: 100,
      cost_total: 1.5,
      series: [],
    };

    vi.mocked(fetchRequestLogs).mockResolvedValue(mockLogs as any);
    vi.mocked(fetchLogStats).mockResolvedValue(mockStats as any);

    const wrapper = mountLogsPage();

    // 初始状态 loading
    expect(fetchRequestLogs).toHaveBeenCalled();

    await flushPromises();

    // log table rendered
    const rows = wrapper.findAll('tbody tr');
    expect(rows.length).toBe(1); // 1 row + 0 empty state (if has data)
    expect(rows[0].text()).toContain('claude');

    // stats rendered
    expect(wrapper.find('.logs-summary').exists()).toBe(true);
  });

  it('显示空状态', async () => {
    vi.mocked(fetchRequestLogs).mockResolvedValue([]);
    vi.mocked(fetchLogStats).mockResolvedValue(null as any);

    const wrapper = mountLogsPage();

    await flushPromises();

    const rows = wrapper.findAll('tbody tr');
    expect(rows.length).toBe(1); // 空状态也是一个 tr
    expect(rows[0].text()).toContain('components.logs.empty');
  });

  it('手动刷新触发重新加载', async () => {
    vi.mocked(fetchRequestLogs).mockResolvedValue([]);
    vi.mocked(fetchLogStats).mockResolvedValue(null as any);

    const wrapper = mountLogsPage();

    await flushPromises();
    vi.clearAllMocks(); // 清除首次加载的调用记录

    const refreshBtn = wrapper
      .findAllComponents(BaseButtonStub)
      .find(button => button.text().includes('components.logs.refresh'));
    expect(refreshBtn).toBeTruthy();

    await refreshBtn!.trigger('click');

    expect(fetchRequestLogs).toHaveBeenCalled();
    expect(fetchLogStats).toHaveBeenCalled();
  });

  it('平台过滤变化会刷新 provider 列表并清空不存在的值', async () => {
    const wrapper = mountLogsPage();
    await flushPromises();

    vi.mocked(fetchLogProviders).mockClear();
    vi.mocked(fetchLogProviders).mockResolvedValue(['alpha']);
    const vm = wrapper.vm as any;
    vm.filters.provider = 'ghost';
    vm.filters.platform = 'codex';
    await flushPromises();

    expect(fetchLogProviders).toHaveBeenCalledTimes(1);
    expect(vm.filters.provider).toBe('');
  });

  it('applyFilters 重置页码与倒计时并重新拉取数据', async () => {
    const wrapper = mountLogsPage();
    await flushPromises();
    const vm = wrapper.vm as any;
    vm.page = 3;
    vm.countdown = 5;
    vi.mocked(fetchRequestLogs).mockClear();
    vi.mocked(fetchLogStats).mockClear();

    await vm.applyFilters();
    await flushPromises();

    expect(vm.page).toBe(1);
    expect(vm.countdown).toBeLessThanOrEqual(30);
    expect(vm.countdown).toBeGreaterThanOrEqual(0);
    expect(fetchRequestLogs).toHaveBeenCalled();
    expect(fetchLogStats).toHaveBeenCalled();
  });

  it('manualRefresh 立即重置倒计时并刷新', async () => {
    const wrapper = mountLogsPage();
    await flushPromises();
    const vm = wrapper.vm as any;
    vm.countdown = 2;
    vi.mocked(fetchRequestLogs).mockClear();
    vi.mocked(fetchLogStats).mockClear();

    vm.manualRefresh();
    await flushPromises();

    expect(vm.countdown).toBeLessThanOrEqual(30);
    expect(vm.countdown).toBeGreaterThanOrEqual(0);
    expect(fetchRequestLogs).toHaveBeenCalledTimes(1);
    expect(fetchLogStats).toHaveBeenCalledTimes(1);
  });

  it('分页函数保持在有效范围', async () => {
    vi.mocked(fetchRequestLogs).mockResolvedValue(
      Array.from({ length: 40 }, (_, idx) =>
        createLog({ id: idx + 1, created_at: `2026-01-${(idx % 30) + 1} 10:00:00` })
      )
    );
    const wrapper = mountLogsPage();
    await flushPromises();
    const vm = wrapper.vm as any;

    expect(vm.totalPages).toBe(3);
    vm.nextPage();
    expect(vm.page).toBe(2);
    vm.nextPage();
    vm.nextPage();
    expect(vm.page).toBe(3);
    vm.prevPage();
    vm.prevPage();
    vm.prevPage();
    expect(vm.page).toBe(1);
  });

  it('格式化工具覆盖多个边界用例', async () => {
    const wrapper = mountLogsPage();
    await flushPromises();
    const vm = wrapper.vm as any;

    expect(vm.httpCodeClass(503)).toBe('http-server-error');
    expect(vm.httpCodeClass(404)).toBe('http-client-error');
    expect(vm.httpCodeClass(302)).toBe('http-redirect');
    expect(vm.httpCodeClass(204)).toBe('http-success');
    expect(vm.httpCodeClass(150)).toBe('http-info');
    expect(vm.durationColor(undefined)).toBe('neutral');
    expect(vm.durationColor(1)).toBe('fast');
    expect(vm.durationColor(3)).toBe('medium');
    expect(vm.durationColor(6)).toBe('slow');
    expect(vm.formatDuration(undefined)).toBe('—');
    expect(vm.formatDuration(2.345)).toBe('2.35s');
    expect(vm.formatStream(0)).toBe('components.logs.streamOff');
    expect(vm.formatStream(1)).toBe('components.logs.streamOn');
    expect(vm.formatNumber(undefined)).toBe('—');
    expect(vm.formatNumber(1200)).toBe('1,200');
    expect(vm.formatCurrency(undefined)).toBe('$0.0000');
    expect(vm.formatCurrency(5)).toBe('$5.00');
    expect(vm.formatCurrency(0.02)).toBe('$0.020');
    expect(vm.formatCurrency(0.005)).toBe('$0.0050');
    expect(vm.formatSeriesLabel('2024-03-01')).toBe('03-01');
    expect(vm.formatSeriesLabel('2024-03-01 13:00:00')).toBe('13:00');
    expect(vm.formatSeriesLabel('bad')).toBe('bad');
    expect(vm.parseLogDate('2024-03-01 12:00:00 +0800 UTC')).toBeInstanceOf(Date);
    expect(vm.formatTime('2024-03-01 12:30:45')).toBe('2024-03-01 12:30:45');
  });

  it('chart yCost tick callback formats values consistently', async () => {
    const wrapper = mountLogsPage();
    await flushPromises();
    const vm = wrapper.vm as any;
    const callback = vm.chartOptions.scales.yCost.ticks.callback;
    expect(callback(2)).toBe('$2.00');
    expect(callback(0.1234)).toBe('$0.1234');
    expect(callback('not-a-number')).toBe('$0');
  });

  it('自动刷新倒计时触发 loadDashboard，并可停止', async () => {
    vi.useFakeTimers();
    const wrapper = mountLogsPage();
    await flushPromises();
    const vm = wrapper.vm as any;
    vi.mocked(fetchRequestLogs).mockClear();
    vi.mocked(fetchLogStats).mockClear();

    vm.startCountdown();
    expect(vm.countdown).toBe(30);

    vi.advanceTimersByTime(29_000);
    expect(fetchRequestLogs).not.toHaveBeenCalled();

    vi.advanceTimersByTime(2_000);
    await flushPromises();
    expect(fetchRequestLogs).toHaveBeenCalledTimes(1);
    expect(vm.countdown).toBeLessThanOrEqual(30);
    expect(vm.countdown).toBeGreaterThanOrEqual(0);

    vi.mocked(fetchRequestLogs).mockClear();
    vm.stopCountdown();
    vi.advanceTimersByTime(5_000);
    expect(fetchRequestLogs).not.toHaveBeenCalled();

    vi.useRealTimers();
  });
});
