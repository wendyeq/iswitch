# iSwitch

iSwitch 是一个基于 **Tauri (Rust)** 和 **Vue 3** 构建的现代化桌面应用，旨在为开发者提供强大的 AI 提供商（如 OpenAI, Anthropic 等）管理与本地代理服务。它是 Claude Code 和 Codex 等 AI 编程工具的最佳伴侣。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/backend-Rust-orange.svg)
![Vue](https://img.shields.io/badge/frontend-Vue3-green.svg)

## 📦 下载安装

支持 macOS (Apple Silicon) 和 Windows 系统。

[![Download for macOS](https://img.shields.io/badge/Download-macOS-orange?style=for-the-badge&logo=apple)](https://git.gdyhsp.com/wendyeq/iswitch/-/releases)
[![Download for Windows](https://img.shields.io/badge/Download-Windows-blue?style=for-the-badge&logo=windows)](https://git.gdyhsp.com/wendyeq/iswitch/-/releases)

> **注意**: 由于暂时未配置自动构建，请前往 [Releases 页面](https://git.gdyhsp.com/wendyeq/iswitch/-/releases) 下载最新手动发布的安装包，或者通过下方命令自行构建。

#### ⚠️ macOS 打开提示 "文件已损坏"？

如果遇到 "文件已损坏，打不开" 的提示，这是 macOS 对未签名应用的安全拦截。请在终端执行以下命令解除隔离：

```bash
sudo xattr -rd com.apple.quarantine /Applications/iSwitch.app
```
*(请将 `/Applications/iSwitch.app` 替换为你的实际安装路径)*

### 本地构建 (推荐)

如果你熟悉开发环境，也可以直接克隆代码在本地运行或构建：

```bash
# 1. 安装依赖
make install

# 2. 启动开发模式
make dev

# 3. 构建安装包 (产物位于 iswitch-tauri/src-tauri/target/release/bundle/)
make build
```

如果你是在 **Windows** 系统下开发，可以直接运行提供的批处理脚本：

```cmd
scripts\build-windows.bat
```



## 🌟 核心功能

*   **本地智能代理 (Local Smart Proxy)**
    *   启动本地代理服务（默认端口 `:18099`），拦截并转发 AI 请求。
    *   **智能路由**：根据配置自动分发请求到最优的提供商。
    *   **自动降级 (Auto-Switch)**：当主提供商（如 OpenAI）响应超时或失败时，自动无缝切换到备用提供商（如 Anthropic/Claude），确保服务不中断。
    *   **流式响应**：完整支持 SSE 流式传输，体验丝滑。

*   **多模型管理 (Provider Management)**
    *   **Claude 支持**：集中管理 Anthropic、DeepSeek、MiniMax、智谱 (ZhipuAI) 等多家 AI 提供商。
    *   **Codex 支持**： OpenAI、特别支持 Azure OpenAI 作为后端，提升企业级稳定性和合规性。
    *   自定义每个模型的优先级和并发策略。

*   **可视化监控 (HUD & Logs)**
    *   **Mini HUD**：精巧的桌面悬浮窗，实时显示当前的生成速度 (TPS)、Token 消耗和模型状态。
    *   **详细日志**：记录每一次 API 调用的完整链路，包括输入输出 Token、耗时、成本估算等。
    *   **成本统计**：直观的图表展示每日/每月的 AI 消耗成本。



## 📅 项目演变

本项目源于对高效 AI 编程体验的探索，经历了三个主要版本的迭代：

1.  **第一版 (cc-switch)**
2.  **第二版 (code-switch)**
3.  **第三版 (iSwitch)**: **当前版本**。基于 `code-switch` 核心逻辑进行了**完全重构**，采用 Tauri + Vue 3 打造了全新的可视化桌面客户端，大幅提升了易用性与交互体验。

## 🛠️ 技术栈

*   **Frontend**: Vue 3, TypeScript, TailwindCSS, Vite
*   **Backend**: Rust (Tokio, Axum, SQLite), Tauri v2
*   **Architecture**: 详见 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

## 🚀 快速开始

### 前置要求

*   [Rust](https://www.rust-lang.org/tools/install) (1.70+)
*   [Node.js](https://nodejs.org/) (20+) & pnpm/npm

### 开发环境搭建

1.  **安装依赖**
    ```bash
    make install
    ```

2.  **启动开发服务器**
    ```bash
    make dev
    ```
    这将同时启动前端服务器和 Tauri 应用程序窗口。

### 常用命令

| 命令 | 说明 |
| :--- | :--- |
| `make install` | 安装前端与后端依赖 |
| `make dev` | 启动本地开发环境 |
| `make build` | 构建生产环境应用包 |
| `make format` | 格式化代码 (Rust + TS) |
| `make test` | 运行后端 Rust 测试 |
| `make test-ui` | 运行前端 Vue 测试 |

## 📚 文档资源

*   **系统架构**: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
*   **代码规范**: [`docs/CODE_FORMATTING.md`](docs/CODE_FORMATTING.md)
*   **质量保证**: [`docs/QUALITY_ASSURANCE_SOP.md`](docs/QUALITY_ASSURANCE_SOP.md)


## 📜 许可证

MIT License
