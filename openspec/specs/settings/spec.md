# settings Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-stack. Update Purpose after archive.
## Requirements
### Requirement: Claude Code 配置自动更新
系统 MUST 自动更新 Claude Code 的本地配置文件，使其请求经过本地代理。

#### Scenario: 启用 Claude 代理
- **GIVEN** 应用已安装且代理服务正在运行
- **WHEN** 用户点击 "启用代理" 按钮
- **THEN** 创建或更新 `~/.claude/settings.json`
- **AND** 设置 `ANTHROPIC_BASE_URL` 为 `http://127.0.0.1:18099`
- **AND** 设置 `ANTHROPIC_AUTH_TOKEN` 为占位符值
- **AND** 原配置文件被备份

#### Scenario: 禁用 Claude 代理
- **GIVEN** Claude 代理已启用
- **WHEN** 用户点击 "禁用代理" 按钮
- **THEN** 删除代理相关配置
- **AND** 从备份恢复原配置文件

#### Scenario: 检测 Claude 代理状态
- **GIVEN** 用户打开应用
- **WHEN** 应用读取 Claude 配置文件
- **THEN** 正确显示代理是否已启用
- **AND** 显示当前配置的 Base URL

---

### Requirement: Codex 配置自动更新
系统 MUST 自动更新 Codex 的本地配置文件，使其请求经过本地代理。

#### Scenario: 启用 Codex 代理
- **GIVEN** 应用已安装且代理服务正在运行
- **WHEN** 用户点击 "启用代理" 按钮
- **THEN** 创建或更新 `~/.codex/config.toml`
- **AND** 添加 `code-switch` model_provider 配置
- **AND** 创建 `~/.codex/auth.json` 认证文件
- **AND** 原配置文件被备份

#### Scenario: 禁用 Codex 代理
- **GIVEN** Codex 代理已启用
- **WHEN** 用户点击 "禁用代理" 按钮
- **THEN** 删除代理相关配置
- **AND** 从备份恢复原配置文件和认证文件

#### Scenario: 检测 Codex 代理状态
- **GIVEN** 用户打开应用
- **WHEN** 应用读取 Codex 配置文件
- **THEN** 正确显示代理是否已启用
- **AND** 显示当前配置的 model_provider

---

### Requirement: 应用设置管理

系统 SHALL 提供应用级别的设置管理功能，并对部分默认值进行调整以简化体验。

#### Scenario: 首页标题显示 - 默认关闭 (NEW)
- **GIVEN** 用户全新安装应用或从未保存过设置
- **WHEN** 应用启动
- **THEN** 首页大标题 **不显示** (show_home_title = false)
- **AND** 设置页面中的"显示首页标题"开关处于关闭状态

### Requirement: Code-Switch 配置导入
系统 SHALL 提供从 Go 版本 code-switch 应用导入供应商和 MCP 配置的能力。

系统 SHALL 扫描 `~/.code-switch/` 目录下的以下文件：

#### claude-code.json 格式
```json
{
  "providers": [
    {
      "id": 1763435071814,
      "name": "provider-name",
      "apiUrl": "https://api.example.com",
      "apiKey": "sk-...",
      "officialSite": "https://example.com",
      "icon": "icon-name",
      "tint": "rgba(...)",
      "accent": "#0a84ff",
      "enabled": true,
      "supportedModels": {"claude-sonnet-4-20250514": true},
      "modelMapping": {"claude-sonnet-4-20250514": "claude-sonnet-latest"}
    }
  ]
}
```

#### codex.json 格式
与 `claude-code.json` 格式相同，包含 Codex provider 列表。

#### mcp.json 格式
```json
{
  "server-name": {
    "type": "stdio",
    "command": "npx",
    "args": ["-y", "package@latest"],
    "env": {"KEY": "value"},
    "url": "https://...",
    "website": "https://...",
    "tips": "Description",
    "enable_platform": ["claude-code", "codex"]
  }
}
```

---

#### Scenario: 检测 code-switch 导入状态
- **GIVEN** 用户导航到通用设置页面
- **WHEN** 页面加载完成
- **THEN** 系统检查 code-switch 配置文件是否存在
- **AND** 显示待导入的 provider 数量和 MCP 服务器数量

#### Scenario: 从 code-switch 导入 providers
- **GIVEN** `~/.code-switch/claude-code.json` 或 `codex.json` 存在
- **WHEN** 用户点击 code-switch 的"导入"按钮
- **THEN** 系统读取 `claude-code.json` 和 `codex.json` 中的 providers
- **AND** 导入 iswitch 中不存在的 providers（基于 `id` 去重）
- **AND** 保留所有 provider 属性（包括 supportedModels 和 modelMapping）

#### Scenario: 从 code-switch 导入 MCP 服务器
- **GIVEN** `~/.code-switch/mcp.json` 存在
- **WHEN** 用户点击 code-switch 的"导入"按钮
- **THEN** 系统读取 `mcp.json` 中的 MCP 服务器配置
- **AND** 导入 iswitch 中不存在的服务器（基于名称去重）
- **AND** 将 `enable_platform` 字段映射为 `platforms`

#### Scenario: 独立导入操作
- **GIVEN** 用户先从 cc-switch 导入配置
- **WHEN** 用户随后从 code-switch 导入配置
- **THEN** 两次导入独立进行
- **AND** 每个来源的配置保持独立
- **AND** 不会相互覆盖

#### Scenario: 字段映射转换
- **GIVEN** code-switch 使用 camelCase 字段命名
- **WHEN** 导入 provider 配置
- **THEN** 系统自动转换字段名：
  - `apiUrl` → `api_url`
  - `apiKey` → `api_key`
  - `officialSite` → `official_site`
  - `supportedModels` → `supported_models`
  - `modelMapping` → `model_mapping`

---

### Requirement: Code-Switch 导入 UI
系统 SHALL 在通用设置页面显示独立的 code-switch 导入区块。

导入区块 SHALL 位于现有 cc-switch 导入区块下方，采用相同的 UI 样式，包含：
- 标签: "导入 code-switch 数据" (或等效的本地化文本)
- 子标签显示: "{providers} providers · {servers} MCP servers 待导入"
- 主操作按钮: "立即导入" / "已同步" / "导入中..."
- 次要操作按钮: "上传 JSON" / "清除选择"

---

#### Scenario: 显示 - code-switch 配置存在
- **GIVEN** `~/.code-switch/claude-code.json` 或 `codex.json` 或 `mcp.json` 存在
- **WHEN** 用户打开通用设置页面
- **THEN** code-switch 导入行显示
- **AND** 显示待导入的数量

#### Scenario: 显示 - code-switch 配置不存在
- **GIVEN** `~/.code-switch/` 目录不存在或目录为空
- **WHEN** 用户打开通用设置页面
- **THEN** code-switch 导入行隐藏

#### Scenario: 显示 - 导入进行中
- **GIVEN** 用户点击了导入按钮
- **WHEN** 导入正在进行
- **THEN** 主操作按钮显示"导入中..."
- **AND** 按钮处于禁用状态

#### Scenario: 显示 - 导入完成
- **GIVEN** 导入成功完成
- **WHEN** 没有更多待导入的配置
- **THEN** 主操作按钮显示"已同步"
- **AND** 待导入数量显示为 0

---

### Requirement: 错误处理
系统 SHALL 优雅地处理导入过程中的错误。

---

#### Scenario: 配置文件不存在
- **GIVEN** `~/.code-switch/` 目录下某配置文件不存在
- **WHEN** 执行导入检测
- **THEN** 系统跳过该文件，继续检测其他文件
- **AND** 返回实际存在文件的导入状态

#### Scenario: 配置文件格式错误
- **GIVEN** 某配置文件包含无效的 JSON
- **WHEN** 尝试解析该文件
- **THEN** 系统记录错误日志
- **AND** 跳过该文件，继续处理其他文件
- **AND** 向用户显示部分导入成功的消息

#### Scenario: Provider 解析失败
- **GIVEN** 某个 provider 条目缺少必要字段
- **WHEN** 解析该 provider
- **THEN** 系统跳过该 provider
- **AND** 继续导入其他有效的 providers

#### Scenario: MCP 服务器解析失败
- **GIVEN** 某个 MCP 服务器配置缺少必要字段
- **WHEN** 解析该服务器
- **THEN** 系统跳过该服务器
- **AND** 继续导入其他有效的服务器

### Requirement: Mini HUD 设置持久化

系统 SHALL 持久化 Mini HUD 的用户偏好设置，**但启动时默认不显示 HUD 窗口**。

#### Scenario: HUD 启动状态 - 默认关闭 (MODIFIED)
- **GIVEN** 用户上次关闭应用时 HUD 处于开启状态
- **WHEN** 应用下次启动
- **THEN** HUD 窗口 **不会** 自动显示
- **AND** 用户需通过托盘菜单手动打开 HUD

#### Scenario: HUD 坐标持久化 (UNCHANGED)
- **GIVEN** 用户拖拽调整了 HUD 窗口位置
- **WHEN** 应用重启
- **THEN** HUD 窗口在之前保存的位置显示（当用户手动打开时）
- **AND** 位置信息 (x, y) 被从配置文件读取

#### Scenario: 置顶设置默认值 (UNCHANGED)
- **GIVEN** 用户没有手动修改过配置文件
- **WHEN** HUD 窗口启动
- **THEN** `always_on_top` 字段默认为 `false`
- **AND** 窗口不置顶

---

