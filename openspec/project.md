# Project Context

## Purpose
[Describe your project's purpose and goals]

## Tech Stack
- **Core Framework**: Tauri v2 (Rust + Webview)
- **Frontend**: Vue 3 + TypeScript + TailwindCSS
- **Backend**: Rust (Tokio, Axum, SQLite)

## Native Application Constraints (CRITICAL)
1. **Window Vibrancy**:
   - The application uses native macOS/Windows vibrancy (via `window_vibrancy` crate).
   - **Constraint**: The root `<html>` and `<body>` MUST have `background: transparent`.
   - **Constraint**: Surface colors MUST use **RGBA** with transparency (e.g., `rgba(30,30,30,0.6)`), NEVER opaque Hex codes (like `#000000`) for backgrounds that overlay the vibrancy.
2. **No CSS Blur**:
   - **Forbidden**: Do NOT use `backdrop-filter: blur(...)` on large containers or the root, as it conflicts with native window blur and degrades performance.
3. **System Integration**:
   - Prefer native window controls and behavior over web-like simulations.
   - Respect `prefers-color-scheme` for theme detection.

## Project Conventions

### Code Style
[Describe your code style preferences, formatting rules, and naming conventions]

### Architecture Patterns
[Document your architectural decisions and patterns]

### Testing Strategy
See [Quality Assurance SOP](../docs/QUALITY_ASSURANCE_SOP.md) for detailed guidelines.

**Core Principles:**
- **Backend (Rust)**: High coverage (≥80%) for core logic (`proxy`, `services`).
  - Tool: `cargo test`, `cargo-tarpaulin`/`grcov`.
- **Frontend (Vue)**: Critical interaction tests for main components; 100% coverage for Utils/Services.
  - Tool: `Vitest`, `Vue Testing Library`.
- **Code Review**: Mandatory for core logic changes; assisted by `/code-review` workflow.

### Git Workflow
[Describe your branching strategy and commit conventions]

## Domain Context
[Add domain-specific knowledge that AI assistants need to understand]

## Important Constraints
[List any technical, business, or regulatory constraints]

## External Dependencies
[Document key external services, APIs, or systems]
