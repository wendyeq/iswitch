# skill Specification

## Purpose
TBD - created by archiving change migrate-to-tauri-stack. Update Purpose after archive.
## Requirements
### Requirement: Skill 仓库管理
系统 SHALL 支持配置多个 GitHub 仓库作为 Skill 来源。

#### Scenario: 默认仓库
- **GIVEN** 用户首次使用应用
- **WHEN** 加载 Skill 列表
- **THEN** 包含内置的默认仓库 (ComposioHQ/awesome-claude-skills, anthropics/skills)
- **AND** 默认仓库已启用

#### Scenario: 添加自定义仓库
- **GIVEN** 用户在 Skill 仓库管理页面
- **WHEN** 用户添加新的 GitHub 仓库 (owner/name)
- **THEN** 仓库被添加到配置
- **AND** 该仓库的 Skills 出现在列表中

#### Scenario: 移除仓库
- **GIVEN** 存在一个自定义添加的仓库
- **WHEN** 用户确认移除
- **THEN** 仓库从配置中删除
- **AND** 该仓库的 Skills 不再显示（已安装的保留）

---

### Requirement: Skill 列表展示
系统 SHALL 聚合多个仓库的 Skills 并展示给用户。

#### Scenario: 加载远程 Skills
- **GIVEN** 存在已配置的 Skill 仓库
- **WHEN** 用户打开 Skill 页面
- **THEN** 从各仓库获取 Skill 目录
- **AND** 解析 skill.yaml 获取名称和描述
- **AND** 标记已安装状态

#### Scenario: 显示本地已安装 Skills
- **GIVEN** 用户之前已安装过 Skills
- **WHEN** 用户打开 Skill 页面
- **THEN** 已安装的 Skills 显示在列表中
- **AND** 标记为 "已安装" 状态

---

### Requirement: Skill 安装与卸载
系统 SHALL 支持从 GitHub 仓库下载并安装 Skill。

#### Scenario: 安装 Skill
- **GIVEN** 用户选择一个未安装的 Skill
- **WHEN** 用户点击 "安装" 按钮
- **THEN** 下载该 Skill 目录的 ZIP 包
- **AND** 解压到本地 Skills 目录 (`~/.claude/skills/`)
- **AND** 更新安装状态

#### Scenario: 卸载 Skill
- **GIVEN** 存在一个已安装的 Skill
- **WHEN** 用户点击 "卸载" 按钮
- **THEN** 删除本地 Skill 目录
- **AND** 更新安装状态为 "未安装"

#### Scenario: 安装失败处理
- **GIVEN** 用户尝试安装 Skill
- **WHEN** 网络错误或仓库不可用
- **THEN** 显示友好的错误提示
- **AND** 不留下不完整的安装文件

---

### Requirement: Skill 元数据解析
系统 SHALL 解析 skill.yaml 文件获取 Skill 信息。

#### Scenario: 解析 skill.yaml
- **GIVEN** Skill 目录中存在 skill.yaml
- **WHEN** 系统读取该文件
- **THEN** 提取 name、description 字段
- **AND** 用于列表显示

#### Scenario: 缺少 skill.yaml
- **GIVEN** Skill 目录中不存在 skill.yaml
- **WHEN** 系统列出 Skills
- **THEN** 使用目录名作为 Skill 名称
- **AND** 描述显示为空或默认值

