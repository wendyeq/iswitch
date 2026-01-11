# mcp Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-stack. Update Purpose after archive.
## Requirements
### Requirement: MCP Server 列表管理
系统 SHALL 提供 MCP Server 的集中管理功能。

#### Scenario: 列出 MCP Servers
- **GIVEN** 用户打开 MCP 管理页面
- **WHEN** 页面加载完成
- **THEN** 显示所有已配置的 MCP Server
- **AND** 每个 Server 显示名称、类型、命令、启用状态

#### Scenario: 添加 MCP Server
- **GIVEN** 用户在 MCP 管理页面
- **WHEN** 用户填写 Server 信息并保存
- **THEN** 新 Server 被添加到列表
- **AND** 配置文件被更新

#### Scenario: 编辑 MCP Server
- **GIVEN** 存在一个已配置的 MCP Server
- **WHEN** 用户修改其配置并保存
- **THEN** 配置变更被持久化
- **AND** 同步更新到 Claude/Codex 配置

#### Scenario: 删除 MCP Server
- **GIVEN** 存在一个已配置的 MCP Server
- **WHEN** 用户确认删除
- **THEN** Server 从列表中移除
- **AND** 从 Claude/Codex 配置中同步移除

---

### Requirement: 双平台同步
系统 SHALL 将 MCP Server 配置同步到 Claude Code 和 Codex 两个平台。

#### Scenario: 同步到 Claude
- **GIVEN** 用户保存 MCP Server 配置
- **AND** Server 启用了 Claude 平台
- **WHEN** 配置保存完成
- **THEN** 更新 Claude Desktop 的 MCP 配置文件
- **AND** 格式符合 Claude 规范 (JSON)

#### Scenario: 同步到 Codex
- **GIVEN** 用户保存 MCP Server 配置
- **AND** Server 启用了 Codex 平台
- **WHEN** 配置保存完成
- **THEN** 更新 Codex 的 MCP 配置文件
- **AND** 格式符合 Codex 规范 (TOML)

#### Scenario: 单独启用/禁用平台
- **GIVEN** 存在一个 MCP Server
- **WHEN** 用户仅启用 Claude 平台
- **THEN** Server 仅同步到 Claude 配置
- **AND** 不出现在 Codex 配置中

---

### Requirement: 从 Claude Desktop 导入
系统 SHALL 支持从 Claude Desktop 配置中导入已有的 MCP Server。

#### Scenario: 检测可导入的 Servers
- **GIVEN** Claude Desktop 已配置 MCP Servers
- **WHEN** 用户打开导入功能
- **THEN** 列出 Claude Desktop 中的所有 MCP Servers
- **AND** 标记哪些已在本应用中存在

#### Scenario: 执行导入
- **GIVEN** 用户选择要导入的 Servers
- **WHEN** 用户确认导入
- **THEN** 选中的 Servers 被添加到配置
- **AND** 不覆盖同名已存在的 Server

---

### Requirement: 占位符验证
系统 SHALL 检测 MCP Server 配置中的占位符并提示用户填写。

#### Scenario: 检测未填写的占位符
- **GIVEN** MCP Server 配置中包含 `{API_KEY}` 占位符
- **WHEN** 用户查看 Server 详情
- **THEN** 高亮显示未填写的占位符
- **AND** 提示用户需要替换为实际值

#### Scenario: 内置 Server 模板
- **GIVEN** 系统内置 chrome-devtools 等常用 Server
- **WHEN** 用户首次使用
- **THEN** 内置 Server 自动出现在列表中
- **AND** 标记为内置类型

