# iSwitch

Desktop companion for managing AI providers via Vue 3 + Tauri.

## Getting Started

```bash
make install   # install frontend deps
make dev       # run Tauri dev server
```

See `docs/ARCHITECTURE.md` for system details.

## Code Formatting & Hooks

- Run `make format` (or `make format-check`) before pushing
- Pre-commit hooks auto-run Prettier + rustfmt on staged files
- Full guide: `docs/CODE_FORMATTING.md`
- Rollout & communications assets: `docs/CODE_FORMATTING_ROLLOUT.md`

## Helpful Commands

| Command            | Purpose            |
| ------------------ | ------------------ |
| `make test`        | Run Rust tests     |
| `make test-ui`     | Run frontend tests |
| `make lint`        | Run Clippy         |
| `make coverage`    | Backend coverage   |
| `make coverage-ui` | Frontend coverage  |

For proposal/spec work, review `openspec/AGENTS.md` and follow the documented workflow.
