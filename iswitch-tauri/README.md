# iSwitch

**(Formerly Code Switch)**

iSwitch is a powerful developer tool designed to manage and switch between Claude Code and Codex providers seamlessly. It provides a local HTTP proxy with dynamic routing, automatic failover, and extensive configuration capabilities.

## Features

- **Provider Management**: Easily add, edit, and switch between multiple AI providers.
- **Local Proxy Service**: A high-performance local proxy that routes requests to the active provider.
- **Configurable Port**: Customize the proxy port (default: 18099) to suit your environment.
- **Automatic Failover**: Ensures continuity by automatically switching to backup providers if the primary one fails.
- **Smart Health Tracking**: Intelligent provider health monitoring with automatic degradation and recovery.
  - Tracks consecutive failures per provider
  - Automatically degrades unhealthy providers after reaching threshold (default: 3 failures)
  - Lazy recovery: degraded providers automatically retry after timeout (default: 5 minutes)
  - Fallback strategy: when all providers are degraded, forces retry on all providers
- **Cross-Platform**: Built with Rust and Tauri, supporting macOS, Windows, and Linux.
- **Modern UI**: A sleek, responsive interface built with Vue 3 and TailwindCSS, supporting system theme matching.
- **Unified Config**: All data stored in `~/.iswitch/` for clean isolation.

## Architecture

For detailed system architecture, data flow diagrams, and module documentation, see:
📐 **[Architecture Documentation](../docs/ARCHITECTURE.md)**

## Legacy Note

This project was previously known as **Code Switch**. We have renamed it to **iSwitch** to better distinguish it from the original Go version and to avoid conflicts. All core functionality remains consistent, with significant performance and stability improvements in this Rust + Tauri version.

## Development

### Quality Assurance

Please refer to the [Quality Assurance SOP](../docs/QUALITY_ASSURANCE_SOP.md) for testing standards and workflow.

### Prerequisites

- Node.js
- Rust (latest stable)
- Tauri CLI

### Setup

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Build

```bash
npm run tauri build
```
