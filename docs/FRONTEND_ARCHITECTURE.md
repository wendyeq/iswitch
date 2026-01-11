# iSwitch 前端架构文档

> **INPUT**: [source: iswitch-tauri/src/.folder.md ([POS]: Vue 3 前端源码目录)]
> **OUTPUT**: 前端架构详细文档，供开发者理解组件关系和数据流
> **PROTOCOL**: FractalFlow v1.0
> **POS**: docs/ - 项目开发者文档目录

---

## 1. 前端概览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Vue 3 Frontend Architecture                        │
│                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                              App.vue (根组件)                            │ │
│  │  ┌──────────────────────────────────────────────────────────────────┐   │ │
│  │  │                          Vue Router                               │   │ │
│  │  └────────────────────────────┬─────────────────────────────────────┘   │ │
│  └───────────────────────────────┼─────────────────────────────────────────┘ │
│                                  │                                           │
│  ┌───────────────────────────────▼─────────────────────────────────────────┐ │
│  │                           Page Components                                │ │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │ │
│  │  │  Main   │ │ General │ │  Logs   │ │   MCP   │ │  Skill  │           │ │
│  │  │ (主页)  │ │ (设置)  │ │ (日志)  │ │ (MCP)   │ │ (技能)  │           │ │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘           │ │
│  └───────┴───────────┴───────────┴───────────┴───────────┴─────────────────┘ │
│                                  │                                           │
│  ┌───────────────────────────────▼─────────────────────────────────────────┐ │
│  │                          Common Components                               │ │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────────────┐   │ │
│  │  │ BaseButton│ │ BaseModal │ │ BaseInput │ │ ModelMappingEditor    │   │ │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────────────────┘   │ │
│  │  ┌───────────────────────┐ ┌───────────────────────────────────────┐   │ │
│  │  │ ModelWhitelistEditor  │ │ Setting/* (ListRow, ThemeSetting...)  │   │ │
│  │  └───────────────────────┘ └───────────────────────────────────────┘   │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                  │                                           │
│  ┌───────────────────────────────▼─────────────────────────────────────────┐ │
│  │                           Services Layer                                 │ │
│  │  ┌─────────────────────────────────────────────────────────────────────┐│ │
│  │  │                         tauri.ts (核心)                              ││ │
│  │  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  ││ │
│  │  │   │Provider  │ │Settings  │ │Logs      │ │MCP       │ │Skill     │  ││ │
│  │  │   │API       │ │API       │ │API       │ │API       │ │API       │  ││ │
│  │  │   └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  ││ │
│  │  └─────────────────────────────────────────────────────────────────────┘│ │
│  │                                    │                                     │ │
│  │              invoke('@tauri-apps/api/core')                              │ │
│  └────────────────────────────────────┼─────────────────────────────────────┘ │
│                                       │                                       │
│  ═════════════════════════════════════════════════════════════════════════   │
│                              Tauri IPC Bridge                                 │
│  ═════════════════════════════════════════════════════════════════════════   │
│                                       │                                       │
│                                       ▼                                       │
│                           Rust Backend Commands                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. 技术栈

| 技术                           | 版本            | 用途                      |
| :----------------------------- | :-------------- | :------------------------ |
| **Vue 3**                      | ^3.5.23         | UI 框架 (Composition API) |
| **TypeScript**                 | ^5.9.3          | 类型安全                  |
| **Vue Router**                 | ^4.6.3          | 路由管理 (Hash 模式)      |
| **Vue I18n**                   | ^11.1.12        | 国际化 (中/英)            |
| **TailwindCSS**                | ^4.1.17         | 工具类样式                |
| **Vite**                       | ^7.2.1          | 构建工具                  |
| **@headlessui/vue**            | ^1.7.23         | 无障碍 UI 组件            |
| **chart.js** + **vue-chartjs** | ^4.5.1 / ^5.3.3 | 图表展示                  |
| **Tauri API**                  | ^2              | 前后端通信                |

---

## 3. 目录结构

```
iswitch-tauri/src/
├── .folder.md                    # FractalFlow 目录描述
├── App.vue                       # 根组件 (主题初始化、RouterView)
├── main.ts                       # 入口文件 (Vue 实例、插件注册)
├── style.css                     # 全局样式 (TailwindCSS + 自定义)
├── vite-env.d.ts                 # Vite 类型声明
│
├── components/                   # Vue 组件
│   ├── Main/                     # 主页面组件
│   │   └── Index.vue             # ★ 核心: Provider 管理、热力图、代理控制
│   │
│   ├── General/                  # 设置页面组件
│   │   └── Index.vue             # 应用设置、配置导入
│   │
│   ├── Logs/                     # 日志页面组件
│   │   └── Index.vue             # 请求日志表格、图表统计
│   │
│   ├── Mcp/                      # MCP 管理页面组件
│   │   └── Index.vue             # MCP Server CRUD
│   │
│   ├── Skill/                    # Skill 管理页面组件
│   │   └── Index.vue             # Skill 安装/卸载
│   │
│   ├── Setting/                  # 设置相关子组件
│   │   ├── LanguageSwitcher.vue  # 语言切换器
│   │   ├── ListRow.vue           # 列表行布局
│   │   ├── ShortcutInput.vue     # 快捷键输入
│   │   └── ThemeSetting.vue      # 主题设置
│   │
│   └── common/                   # 通用基础组件
│       ├── BaseButton.vue        # 基础按钮
│       ├── BaseInput.vue         # 基础输入框
│       ├── BaseModal.vue         # 模态框
│       ├── BaseTextarea.vue      # 多行文本框
│       ├── ModelMappingEditor.vue    # 模型映射编辑器
│       └── ModelWhitelistEditor.vue  # 模型白名单编辑器
│
├── services/                     # API 服务层
│   ├── .folder.md                # 目录描述
│   ├── tauri.ts                  # ★ 核心: 所有 Tauri invoke 封装
│   ├── appSettings.ts            # 应用设置 (重导出)
│   ├── claudeSettings.ts         # Claude/Codex 代理设置 (重导出)
│   ├── configImport.ts           # 配置导入服务 (重导出)
│   ├── logs.ts                   # 日志服务 (重导出)
│   ├── mcp.ts                    # MCP 服务 (重导出)
│   ├── skill.ts                  # Skill 服务 (重导出)
│   └── version.ts                # 版本服务 (重导出)
│
├── utils/                        # 工具函数
│   ├── .folder.md                # 目录描述
│   ├── ThemeManager.ts           # 主题管理 (dark/light/system)
│   ├── hotkeyUtils.ts            # 快捷键解析工具
│   ├── i18n.ts                   # 国际化配置
│   └── toast.ts                  # Toast 通知
│
├── router/                       # 路由配置
│   └── index.ts                  # Vue Router 实例 (Hash 模式)
│
├── types/                        # TypeScript 类型
│   └── index.d.ts                # 全局类型声明
│
├── data/                         # 静态数据
│   ├── cards.ts                  # Provider 卡片配置
│   └── usageHeatmap.ts           # 热力图数据处理
│
├── icons/                        # 图标资源
│   └── lobeIconMap.ts            # LobeHub 图标 SVG 映射
│
└── locales/                      # 国际化语言包
    ├── index.ts                  # 语言加载器
    ├── en.json                   # 英文
    └── zh.json                   # 中文
```

---

## 4. 路由配置

```typescript
// router/index.ts
const routes = [
  { path: '/', component: MainPage }, // 主页 - Provider 管理
  { path: '/logs', component: LogsPage }, // 日志页 - 请求日志
  { path: '/settings', component: GeneralPage }, // 设置页 - 应用配置
  { path: '/mcp', component: McpPage }, // MCP 页 - MCP 服务管理
  { path: '/skill', component: SkillPage }, // Skill 页 - 技能管理
];
```

**路由模式**: `createWebHashHistory()` (Tauri 推荐)

---

## 5. 组件详解

### 5.1 Main/Index.vue (主页面)

**代码行数**: ~1244 行 (需要拆分)

**核心功能**:

| 区域              | 功能                       | 相关服务                              |
| :---------------- | :------------------------- | :------------------------------------ |
| **全局导航**      | 主题切换、设置入口         | `ThemeManager.ts`                     |
| **热力图**        | 使用统计可视化             | `fetchHeatmapStats()`                 |
| **Tab 切换**      | Claude / Codex 平台切换    | —                                     |
| **Provider 列表** | 卡片式 Provider 管理       | `fetchProviders()`, `saveProviders()` |
| **代理开关**      | 启用/禁用代理              | `enableProxy()`, `disableProxy()`     |
| **Provider 统计** | 成功率、请求数、Token 统计 | `fetchProviderDailyStats()`           |
| **Modal 弹窗**    | 创建/编辑 Provider         | —                                     |

**数据流**:

```
┌─────────────┐    ┌─────────────────┐    ┌────────────────┐
│  onMounted  │───►│ loadUsageHeatmap│───►│ fetchHeatmapStats │
│             │    │ loadProviders   │    │ invoke(load_providers) │
│             │    │ refreshProxy    │    │ invoke(get_*_proxy_status) │
│             │    │ loadStats       │    │ fetchProviderDailyStats │
└─────────────┘    └─────────────────┘    └────────────────┘
        │
        ▼
┌──────────────────────────────────────────────────────────┐
│                      UI 渲染                              │
│  ┌──────────────┐  ┌───────────────┐  ┌───────────────┐ │
│  │  热力图       │  │  Provider     │  │  统计数据     │ │
│  │  (usageHeatmap)│  │  卡片列表     │  │  (stats)      │ │
│  └──────────────┘  └───────────────┘  └───────────────┘ │
└──────────────────────────────────────────────────────────┘
```

**状态管理** (使用 `reactive` / `ref`):

```typescript
// Provider 卡片数据 (按平台分组)
const cards = reactive<Record<ProviderTab, AutomationCard[]>>({
  claude: [...],
  codex: [...],
})

// 代理状态
const proxyStates = reactive<Record<ProviderTab, boolean>>({
  claude: false,
  codex: false,
})

// Provider 统计数据
const providerStatsMap = reactive<Record<ProviderTab, Record<string, ProviderDailyStat>>>({
  claude: {},
  codex: {},
})

// UI 状态
const showHeatmap = ref(true)
const showHomeTitle = ref(true)
const selectedIndex = ref(0)
```

---

### 5.2 Logs/Index.vue (日志页面)

**核心功能**:

| 功能         | 说明                             | 相关服务               |
| :----------- | :------------------------------- | :--------------------- |
| **统计卡片** | 请求数、Token 数、缓存命中、费用 | `fetchLogStats()`      |
| **折线图表** | 30 天趋势图 (多数据集)           | Chart.js + vue-chartjs |
| **筛选器**   | 平台、Provider 筛选              | —                      |
| **日志表格** | 分页请求日志列表                 | `fetchRequestLogs()`   |
| **自动刷新** | 30 秒倒计时刷新                  | `setInterval`          |

**图表数据结构**:

```typescript
const chartData = computed(() => ({
  labels: getLast30Days().map(formatSeriesLabel),
  datasets: [
    { label: '费用',     data: [...], yAxisID: 'yCost' },
    { label: '输入 Token', data: [...] },
    { label: '输出 Token', data: [...] },
    { label: '推理 Token', data: [...] },
    { label: '缓存写入',   data: [...] },
    { label: '缓存读取',   data: [...] },
  ]
}))
```

---

### 5.3 General/Index.vue (设置页面)

**核心功能**:

| 区域             | 功能                                 | 服务                                             |
| :--------------- | :----------------------------------- | :----------------------------------------------- |
| **应用设置**     | 热力图、首页标题、自动启动、代理端口 | `fetchAppSettings()`, `saveAppSettings()`        |
| **故障转移设置** | 阈值、恢复超时                       | 同上                                             |
| **配置导入**     | cc-switch / code-switch 导入         | `importFromCcSwitch()`, `importFromCodeSwitch()` |
| **外观设置**     | 语言、主题                           | `LanguageSwitcher`, `ThemeSetting`               |
| **关于**         | 版本号显示                           | `getVersion()`                                   |

---

### 5.4 Mcp/Index.vue (MCP 管理页面)

**核心功能**:

| 功能           | 说明                         | 服务                |
| :------------- | :--------------------------- | :------------------ |
| **MCP 列表**   | 显示所有 MCP Server          | `fetchMcpServers()` |
| **平台开关**   | Claude Code / Codex 独立启用 | `saveMcpServers()`  |
| **创建/编辑**  | 支持 stdio / http 两种类型   | —                   |
| **参数编辑**   | args (多行)、env (键值对)    | —                   |
| **占位符检测** | 检测未填写的 `{变量}`        | —                   |

---

### 5.5 通用组件 (common/)

| 组件                     | 功能       | 特性                                  |
| :----------------------- | :--------- | :------------------------------------ |
| **BaseButton**           | 按钮       | variant: primary/outline/danger/ghost |
| **BaseModal**            | 模态框     | 使用 @headlessui/vue                  |
| **BaseInput**            | 输入框     | 支持 v-model                          |
| **BaseTextarea**         | 多行输入   | 支持 v-model                          |
| **ModelMappingEditor**   | 模型映射   | 支持通配符 `*`                        |
| **ModelWhitelistEditor** | 模型白名单 | 多选支持                              |

---

## 6. Services 层架构

### 6.1 核心服务 (tauri.ts)

所有 Tauri `invoke` 调用都封装在 `services/tauri.ts` 中:

```typescript
// 示例: Provider 相关
export const fetchProviders = async (kind: ProviderKind): Promise<Provider[]> =>
  invoke<Provider[]>('load_providers', { kind });

export const saveProviders = async (kind: ProviderKind, providers: Provider[]): Promise<void> =>
  invoke('save_providers', { kind, providers });
```

**API 分类**:

| 类别         | 函数                                                    | 后端 Command                                              |
| :----------- | :------------------------------------------------------ | :-------------------------------------------------------- |
| **版本**     | `fetchCurrentVersion()`                                 | `get_version`                                             |
| **代理**     | `fetchProxyStatus()`, `enableProxy()`, `disableProxy()` | `get_*_proxy_status`, `enable_*_proxy`, `disable_*_proxy` |
| **设置**     | `fetchAppSettings()`, `saveAppSettings()`               | `get_app_settings`, `save_app_settings`                   |
| **日志**     | `fetchRequestLogs()`, `fetchLogStats()`                 | `list_request_logs`, `get_log_stats`                      |
| **MCP**      | `fetchMcpServers()`, `saveMcpServers()`                 | `list_mcp_servers`, `save_mcp_servers`                    |
| **Skill**    | `fetchSkills()`, `installSkill()`                       | `list_skills`, `install_skill`                            |
| **Provider** | `fetchProviders()`, `saveProviders()`                   | `load_providers`, `save_providers`                        |

### 6.2 服务文件职责

| 文件                | 职责                                         |
| :------------------ | :------------------------------------------- |
| `tauri.ts`          | 所有 Tauri invoke 调用 + 类型定义            |
| `appSettings.ts`    | 重导出 `fetchAppSettings`, `saveAppSettings` |
| `claudeSettings.ts` | 重导出代理相关函数                           |
| `logs.ts`           | 重导出日志相关函数                           |
| `mcp.ts`            | 重导出 MCP 相关函数                          |
| `skill.ts`          | 重导出 Skill 相关函数                        |
| `configImport.ts`   | 重导出配置导入相关函数                       |
| `version.ts`        | 重导出版本相关函数                           |

---

## 7. Utils 工具层

### 7.1 ThemeManager.ts

```typescript
export type ThemeMode = 'light' | 'dark' | 'systemdefault';

export function initTheme(); // 初始化主题 + 监听系统变化
export function applyTheme(mode); // 应用主题 (添加 CSS class)
export function setTheme(mode); // 保存并应用主题
export function getCurrentTheme(); // 获取当前主题
```

**存储**: `localStorage.getItem('theme')`

### 7.2 i18n.ts

```typescript
export const i18n = createI18n({
  legacy: false, // Composition API 模式
  locale: 'zh', // 默认中文
  fallbackLocale: 'en',
});

export async function setupI18n(locale: Locale);
```

### 7.3 toast.ts

```typescript
export function showToast(message: string, type: 'success' | 'error' = 'success');
```

**样式**: macOS 风格的通知提示

### 7.4 hotkeyUtils.ts

```typescript
export function parseHotkeyString(hotkey: string);
export function parseShortcutToHotkey(shortcut: string);
export function formatHotkeyString(key: number, modifier: number);
export function formatHotkeyStringmac(keycode: number, modifiers: number);
```

---

## 8. 数据层 (data/)

### 8.1 cards.ts

定义 Provider 卡片默认配置:

```typescript
export type AutomationCard = {
  id: number
  name: string
  apiUrl: string
  apiKey: string
  officialSite: string
  icon: string
  tint: string           // 背景色
  accent: string         // 强调色
  enabled: boolean
  supportedModels?: Record<string, boolean>
  modelMapping?: Record<string, string>
  level?: number         // 优先级
}

export const automationCardGroups: Record<'claude' | 'codex', AutomationCard[]> = {
  claude: [...],
  codex: [...],
}
```

### 8.2 usageHeatmap.ts

热力图数据处理:

```typescript
export type UsageHeatmapDay = {
  label: string           // 显示标签 "01-15 14"
  dateKey: string         // ISO 日期
  requests: number
  inputTokens: number
  outputTokens: number
  reasoningTokens: number
  cost: number
  intensity: number       // 0-4 强度等级
}

export type UsageHeatmapWeek = UsageHeatmapDay[]

// 核心函数
export const buildUsageHeatmapMatrix = (stats: HeatmapStat[], days: number)
export const generateFallbackUsageHeatmap = (days: number)
```

---

## 9. 国际化 (locales/)

### 9.1 语言包结构

```json
// zh.json
{
  "components": {
    "main": {
      "hero": { "title": "iSwitch", "eyebrow": "AI 代理管理" },
      "controls": { "theme": "切换主题", "settings": "设置" },
      "heatmap": { ... },
      "providers": { ... },
      "form": { ... }
    },
    "logs": { ... },
    "mcp": { ... },
    "general": { ... }
  }
}
```

### 9.2 使用方式

```vue
<script setup>
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
</script>

<template>
  <h1>{{ t('components.main.hero.title') }}</h1>
</template>
```

---

## 10. 样式系统

### 10.1 TailwindCSS 配置

- 使用 TailwindCSS v4
- 通过 `@tailwindcss/postcss` 集成

### 10.2 自定义样式 (style.css)

```css
/* 全局变量 */
:root {
  --mac-text: #0f172a;
  --mac-text-secondary: #94a3b8;
  --mac-bg: #ffffff;
  /* ... */
}

html.dark {
  --mac-text: #f8fafc;
  --mac-text-secondary: #cbd5e1;
  --mac-bg: #1e293b;
  /* ... */
}

/* 组件样式 */
.mac-switch { ... }
.mac-toast { ... }
.contrib-grid { ... }   /* 热力图网格 */
.automation-card { ... } /* Provider 卡片 */
```

---

## 11. 测试状态

> ✅ **当前状态**: 已建立完整的测试基础设施 (2026-01-07)

### 11.1 测试架构

| 任务               | 工具                   | 状态      | 覆盖率                                          |
| :----------------- | :--------------------- | :-------- | :---------------------------------------------- |
| **单元测试运行器** | Vitest                 | ✅ 已配置 | -                                               |
| **组件测试工具**   | Vue Test Utils         | ✅ 已配置 | `BaseButton`, `BaseInput`, `Main/Index` (Smoke) |
| **环境模拟**       | jsdom + Tauri API Mock | ✅ 已配置 | 100% Mock 适配                                  |
| **覆盖率报告**     | @vitest/coverage-v8    | ✅ 已配置 | Utils: 100%, Services: 80%                      |

### 11.2 已完成测试覆盖

1. **Utils 工具层** (100%): `ThemeManager`, `hotkeyUtils`, `toast`, `usageHeatmap`
2. **Services 服务层** (80%): `tauri.ts` (核心 IPC 调用及错误处理)
3. **Common 组件**: `BaseButton`, `BaseInput` 单元测试
4. **Main 组件**: `Main/Index.vue` 集成冒烟测试 (验证 DI 注入和渲染)

### 11.2 参考

参见 `openspec/changes/establish-quality-standards/tasks.md` 阶段 3

---

## 12. 待优化项

### 12.1 代码组织

1. **Main/Index.vue 拆分** (1244 行 → 多个组件)
   - 抽取 `HeatmapSection.vue`
   - 抽取 `ProviderCard.vue`
   - 抽取 `ProviderModal.vue`
   - 使用 Composable: `useProviderStats()`, `useProxyControl()`

2. **App.vue 清理**
   - 删除调试代码
   - 统一使用 ThemeManager.ts

### 12.2 类型安全

- 减少 `as any` 使用
- 为所有 Tauri 返回值定义类型

### 12.3 国际化

- 替换硬编码文本 ("上移"/"下移")

---

## 13. 更新记录

| 日期       | 版本 | 更新内容         |
| :--------- | :--- | :--------------- |
| 2026-01-07 | v1.0 | 初始前端架构文档 |
