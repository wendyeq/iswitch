# Code Formatting Guide

This guide explains how to keep the iSwitch codebase consistent using the new automated formatters and git hooks.

## Quick Start

1. Install dependencies: `cd iswitch-tauri && npm install`
2. Run `make format` from the repo root after large edits
3. Commit as usual – the pre-commit hook formats staged files automatically
4. If the hook reports issues, run `make format` (or the scoped command below) and re-stage files

## Tooling Overview

| Area                       | Tooling      | Config                                          | Key Rules                                                       |
| -------------------------- | ------------ | ----------------------------------------------- | --------------------------------------------------------------- |
| Vue / TS / JS / JSON / CSS | Prettier 3.x | `iswitch-tauri/.prettierrc`                     | Single quotes, 2 spaces, trailing commas, width 120, LF endings |
| Rust backend               | rustfmt      | `rustfmt.toml`                                  | Width 100, 4 spaces, import reordering, edition 2021            |
| Markdown docs              | Prettier     | `iswitch-tauri/.prettierrc` + `.prettierignore` | Consistent headings/lists, fenced code blocks with language     |

Generated assets (`dist/`, `node_modules/`, `package-lock.json`, `CHANGELOG.md`, etc.) are excluded via `.prettierignore`.

## Commands

| Scope               | Command                                           | Notes                                                              |
| ------------------- | ------------------------------------------------- | ------------------------------------------------------------------ |
| Frontend (write)    | `cd iswitch-tauri && npm run format`              | Formats Vue/TS/JS/CSS/JSON/Markdown inside `iswitch-tauri/`        |
| Frontend (check)    | `cd iswitch-tauri && npm run format:check`        | Reports formatting issues without modifying files                  |
| Rust backend        | `cd iswitch-tauri/src-tauri && cargo fmt`         | Uses `rustfmt.toml` at repo root                                   |
| Rust check only     | `cd iswitch-tauri/src-tauri && cargo fmt --check` | Fails on mismatches                                                |
| Entire repo         | `make format`                                     | Runs frontend formatter, `cargo fmt`, then formats repo-level docs |
| Entire repo (check) | `make format-check`                               | Read-only validation; exits non‑zero on issues                     |
| Docs only           | `make format-docs`                                | Formats Markdown outside `iswitch-tauri/`                          |

## Pre-Commit Hooks

- Husky config lives in `iswitch-tauri/.husky` and installs via the `prepare` script after `npm install`
- Hooks run `lint-staged` from the repo root so file paths match staged files
- Formatters run only on staged files and re-stage them after formatting

`lint-staged` mapping:

| Pattern                               | Formatter                                                                                         |
| ------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `*.{vue,ts,tsx,js,jsx,css,scss,json}` | `prettier --config iswitch-tauri/.prettierrc --ignore-path iswitch-tauri/.prettierignore --write` |
| `*.md`                                | Same Prettier command (covers docs + specs)                                                       |
| `src-tauri/**/*.rs`                   | `rustfmt --edition 2021`                                                                          |

Hook output highlights offending files. Successful runs finish in <3s for typical commits and <5s for 30-file commits (parallel execution).

## IDE Integration

- **VS Code**: install Prettier + Rust Analyzer, enable `editor.formatOnSave`, set default formatter to Prettier for TS/Vue and `rust-analyzer.rustfmt.enable` for Rust.
- **JetBrains**: enable “Run Prettier on Save” with the config file path `iswitch-tauri/.prettierrc`; Rust plugin uses `cargo fmt` automatically.
- **Other editors**: configure them to call `npm run format` or `cargo fmt` on save, or rely on the pre-commit hook.

## Troubleshooting

| Symptom                       | Fix                                                                                                                      |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `prettier: command not found` | Run `cd iswitch-tauri && npm install` to restore dev dependencies                                                        |
| Hook hangs on first commit    | Ensure you ran `npm install` so Husky installs hooks; rerun `git config core.hooksPath iswitch-tauri/.husky/_` if needed |
| File keeps being reformatted  | Unstage unrelated files or add them to `.prettierignore` when they are generated                                         |
| rustfmt rewrites entire crate | Stage only the files you intend to format; lint-staged will only pass staged paths to `rustfmt`                          |
| Docs not following rules      | Run `make format-docs` to apply Markdown formatting outside the frontend folder                                          |

## Emergency Bypass

`git commit --no-verify` skips the hook. Only use this for urgent hotfixes and follow up with a proper formatting commit.

## Verification Checklist

- `npm run format` / `npm run format:check`
- `cargo fmt` / `cargo fmt --check`
- `make format` / `make format-check`
- `git commit` (hook auto-runs formatters)

Document any failures in `reports/` or your PR description before requesting review.
