# 版本管理工作流

> 本文档描述 iSwitch 项目的版本号规范、发布流程和相关工具使用方法。

## 版本号规范

iSwitch 遵循 [语义化版本 2.0.0](https://semver.org/lang/zh-CN/) 规范：

```
MAJOR.MINOR.PATCH
  │     │     └─── 补丁版本：向下兼容的问题修复
  │     └───────── 次版本：向下兼容的功能新增
  └─────────────── 主版本：有不兼容的 API 变更
```

**当前版本**: 参见 `iswitch-tauri/src-tauri/tauri.conf.json`

### 版本号位置

版本号需要在以下三个文件中保持同步（使用脚本自动维护）：

| 文件                                      | 用途                           |
| ----------------------------------------- | ------------------------------ |
| `iswitch-tauri/src-tauri/tauri.conf.json` | **单一数据源**，Tauri 打包使用 |
| `iswitch-tauri/package.json`              | 前端依赖版本                   |
| `iswitch-tauri/src-tauri/Cargo.toml`      | Rust crate 版本                |

## 版本更新命令

使用 Makefile 命令更新版本：

```bash
# 查看当前版本
make version

# 递增补丁版本：0.1.0 → 0.1.1
make bump-patch

# 递增次版本：0.1.0 → 0.2.0
make bump-minor

# 递增主版本：0.1.0 → 1.0.0
make bump-major
```

也可以直接使用脚本设置特定版本：

```bash
# macOS / Linux
./scripts/bump-version.sh set 2.0.0

# Windows PowerShell
.\scripts\bump-version.ps1 set 2.0.0
```

> **跨平台说明**: Makefile 会自动检测操作系统，在 Windows 上使用 PowerShell 脚本，在 macOS/Linux 上使用 Bash 脚本。

## 发布流程

### 1. 常规发布

```bash
# 一键发布 (版本更新 + CHANGELOG + Git 提交)
make release TYPE=patch   # 或 minor / major
```

### 2. 手动发布步骤

如果需要更精细的控制，可以手动执行：

```bash
# 1. 更新版本号
make bump-patch

# 2. 更新 CHANGELOG
make changelog

# 3. 提交变更
git add -A
git commit -m "chore: release v0.1.1"

# 4. 创建 Git 标签 (可选)
git tag v0.1.1

# 5. 推送到远程
git push && git push --tags
```

## CHANGELOG 自动生成

项目使用 [git-cliff](https://git-cliff.org/) 自动生成 CHANGELOG。

### 安装 git-cliff

```bash
# 使用 Cargo
cargo install git-cliff

# 或使用 Homebrew (macOS)
brew install git-cliff
```

### 生成 CHANGELOG

```bash
make changelog
```

### Commit Message 规范

为了生成高质量的 CHANGELOG，提交信息应遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

#### 常用类型

| 类型       | 说明         | CHANGELOG 分组   |
| ---------- | ------------ | ---------------- |
| `feat`     | 新功能       | ✨ Features      |
| `fix`      | Bug 修复     | 🐛 Bug Fixes     |
| `docs`     | 文档变更     | 📚 Documentation |
| `perf`     | 性能优化     | ⚡ Performance   |
| `refactor` | 代码重构     | ♻️ Refactor      |
| `style`    | 代码格式调整 | 🎨 Styling       |
| `test`     | 测试相关     | 🧪 Testing       |
| `chore`    | 构建/工具链  | ⚙️ Miscellaneous |

#### 示例

```bash
# 新功能
git commit -m "feat(proxy): add automatic failover support"

# Bug 修复
git commit -m "fix(ui): correct dark mode color contrast"

# 文档
git commit -m "docs: update versioning workflow guide"
```

## 前端版本显示

应用版本号可以在设置页面 → 关于 区块中查看，格式为 `iSwitch v{version}`。

版本号通过 Tauri API 获取：

```typescript
import { getVersion } from '@tauri-apps/api/app';

const version = await getVersion(); // 返回 "0.1.0"
```

## 依赖要求

| 工具        | 用途           | 安装命令                  |
| ----------- | -------------- | ------------------------- |
| `jq`        | JSON 处理      | `brew install jq`         |
| `git-cliff` | CHANGELOG 生成 | `cargo install git-cliff` |

## 参考资料

- [语义化版本 2.0.0](https://semver.org/lang/zh-CN/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [git-cliff 文档](https://git-cliff.org/docs)
