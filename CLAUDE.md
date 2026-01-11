<!-- OPENSPEC:START -->

# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:

- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:

- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

---

# FractalFlow Instructions

Always open `@/skills/fractal/AGENTS.md` when the request:

- Mentions documentation structure, headers, or consistency
- Involves project initialization or code/doc sync
- Requires enforcing FractalFlow protocol

Use `@/skills/fractal/AGENTS.md` to learn:

- Guardian Protocol rules
- File header generation
- Folder documentation standards
- Context audit procedures

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

## Code Style & Formatting

- Follow Prettier config at `iswitch-tauri/.prettierrc` (120 width, single quotes, trailing commas, LF endings) for Vue/TS/JS/CSS/JSON/Markdown.
- Rust files must pass `cargo fmt` using the repo `rustfmt.toml` (100 width, 4 spaces, reordered imports).
- Favor `make format` / `make format-check` before sharing work; these commands run frontend + backend + doc formatters.
- Pre-commit hooks (Husky + lint-staged) will auto-format staged files; re-stage changes if a hook updates files.
- Never format generated artifacts listed in `.prettierignore` (node_modules, dist, CHANGELOG, package-lock, etc.).
- In emergencies, note when you use `git commit --no-verify` and run `make format` as follow-up.
