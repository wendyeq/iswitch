# Code Formatting Rollout Plan

This document bundles the assets needed for the team rollout of automated formatting.

## 1. Team Announcement (Share in Slack/Email)

```
Subject: 📏 Automated Code Formatting & Pre-Commit Hooks Live

Hi team,

We just enabled automated formatters and pre-commit hooks across the repo.

Why it matters:
- Consistent Vue/TS, Rust, and Markdown style
- Faster reviews with fewer nits
- Hooks auto-fix staged files before each commit

What you need to do:
1. Pull the latest `main`
2. Run `cd iswitch-tauri && npm install`
3. Format your branch with `make format`
4. Commit as usual – the hook will run `prettier` + `rustfmt`

Docs & references:
- Code Formatting Guide: docs/CODE_FORMATTING.md
- Troubleshooting: same doc, "Troubleshooting" section

Ping #dev-help if you need a bypass or see hook issues.

Thanks!
```

## 2. Training Session Outline (30 minutes)

| Time      | Topic           | Details                                                             |
| --------- | --------------- | ------------------------------------------------------------------- |
| 0-5 min   | Context         | Show before/after screenshots, explain tooling stack                |
| 5-15 min  | Live demo       | `make format`, `git commit` with hook output, `lint-staged --debug` |
| 15-20 min | IDE setup       | Walk through VS Code + Rust Analyzer settings                       |
| 20-25 min | Troubleshooting | Demo failing hook, syntax error, `--no-verify` bypass               |
| 25-30 min | Q&A / hands-on  | Pair format tricky files, ensure everyone runs commands             |

Preparation checklist:

- Share `docs/CODE_FORMATTING.md` beforehand
- Create sample branch with intentionally messy files for the demo
- Capture hook timings to show performance expectations

## 3. First-Week Support Plan

- **Point of contact**: Dev productivity lead (or sprint captain)
- **Office hours**: 15-minute daily slot on Slack Huddle for the first 5 workdays
- **Issue tracking**: Log hook/formatting issues in `reports/formatting-rollout.md` (create if missing)
- **Knowledge base updates**: Append any new edge cases to `docs/CODE_FORMATTING.md`
- **Success criteria**:
  - 100% of devs successfully run `make format` on day 1
  - No more than 5 hook bypasses during the week (monitor via PR notes)

Document adoption metrics at the end of the week and decide if CI enforcement is needed.
