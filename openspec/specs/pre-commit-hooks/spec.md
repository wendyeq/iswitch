# pre-commit-hooks Specification

## Purpose
TBD - created by archiving change add-code-formatting-and-pre-commit-hooks. Update Purpose after archive.
## Requirements
### Requirement: Pre-Commit Hook Installation

The project MUST automatically install git pre-commit hooks when developers set up their development environment.

**Priority**: P1 (Must Have)

#### Scenario: Developer installs project dependencies

**Given** a developer has cloned the iSwitch repository
**When** they run `npm install` in the `iswitch-tauri/` directory
**Then** Husky MUST automatically install git hooks in `.git/hooks/`
**And** the `pre-commit` hook MUST be created and executable
**And** the installation process SHOULD complete without errors

#### Scenario: Pre-commit hook is executable

**Given** the pre-commit hook has been installed
**When** the developer runs `ls -la .git/hooks/pre-commit`
**Then** the file MUST exist
**And** MUST have executable permissions (`-rwxr-xr-x`)
**And** MUST be a symlink or script that invokes lint-staged

### Requirement: Automatic Formatting on Commit

The pre-commit hook MUST automatically format staged files before allowing the commit to proceed.

**Priority**: P1 (Must Have)

#### Scenario: Developer commits formatted code

**Given** a developer has staged a Vue component file that is already properly formatted
**And** they have staged a Rust file that is already properly formatted
**When** they run `git commit -m "feat: add new feature"`
**Then** the pre-commit hook MUST check the formatting
**And** MUST find no formatting issues
**And** the commit MUST proceed successfully
**And** the commit SHOULD complete within 3 seconds

#### Scenario: Developer commits unformatted code

**Given** a developer has staged a TypeScript file with inconsistent formatting (e.g., mixed quotes, no semicolons)
**When** they run `git commit -m "feat: add new feature"`
**Then** the pre-commit hook MUST detect formatting violations
**And** Prettier MUST automatically format the file in-place
**And** the formatted file MUST be re-staged automatically
**And** the commit MUST proceed successfully with the formatted code
**And** the developer SHOULD see a message indicating files were formatted

#### Scenario: Developer commits unformatted Rust code

**Given** a developer has staged a Rust file with formatting issues (e.g., inconsistent indentation, line width)
**When** they run `git commit -m "feat: add service"`
**Then** the pre-commit hook MUST detect formatting violations
**And** rustfmt MUST automatically format the file
**And** the formatted file MUST be re-staged
**And** the commit MUST proceed successfully

#### Scenario: Mixed file types in single commit

**Given** a developer has staged multiple files: a Vue component, a TypeScript service, a Rust module, and a Markdown doc
**And** some files have formatting issues
**When** they run `git commit`
**Then** the pre-commit hook MUST run appropriate formatters for each file type
**And** all files MUST be formatted according to their respective formatters
**And** the commit MUST proceed only after all files are properly formatted
**And** the formatters MUST run in parallel for better performance

### Requirement: Formatting Validation

The pre-commit hook MUST validate that staged files meet formatting standards and block commits if validation fails.

**Priority**: P1 (Must Have)

#### Scenario: Commit is blocked due to syntax errors

**Given** a developer has staged a TypeScript file with syntax errors (e.g., missing closing brace)
**When** they run `git commit`
**Then** Prettier MUST fail to format the file due to syntax error
**And** the pre-commit hook MUST exit with a non-zero status
**And** the commit MUST be blocked
**And** an error message MUST be displayed indicating:
  - The file with syntax errors
  - The line number of the error
  - Suggestion to run `npm run format` manually

#### Scenario: Formatter fails unexpectedly

**Given** a developer has staged a file
**When** the pre-commit hook runs
**And** the formatter crashes or encounters an unexpected error
**Then** the commit MUST be blocked
**And** an error message MUST be displayed with:
  - The formatter that failed (Prettier or rustfmt)
  - The exit code or error message
  - Suggestion to try `git commit --no-verify` in emergency

#### Scenario: Error message is clear and actionable

**Given** a formatting check has failed during pre-commit
**When** the error message is displayed to the developer
**Then** the message MUST include:
  - Clear indication of what failed (e.g., "Prettier found 2 errors")
  - File paths and line numbers of issues
  - Suggested command to fix (e.g., "Run `npm run format` to auto-fix")
  - Information about bypassing the hook with `--no-verify`

### Requirement: File Type Filtering

The pre-commit hook system MUST only run appropriate formatters on files that match specific patterns.

**Priority**: P1 (Must Have)

#### Scenario: Only frontend files are processed by Prettier

**Given** a developer has staged: `Component.vue`, `service.ts`, `handler.rs`, `README.md`
**When** the pre-commit hook runs
**Then** Prettier MUST process: `Component.vue`, `service.ts`, `README.md`
**And** Prettier MUST NOT process: `handler.rs`
**And** rustfmt MUST process: `handler.rs`
**And** rustfmt MUST NOT process the other files

#### Scenario: Generated files are excluded

**Given** a developer has staged: `package-lock.json`, `dist/bundle.js`, `src/App.vue`
**When** the pre-commit hook runs
**Then** `package-lock.json` and `dist/bundle.js` MUST be skipped (excluded in .prettierignore)
**And** `src/App.vue` MUST be formatted

#### Scenario: Partially staged files are handled correctly

**Given** a developer has modified `service.ts` but only staged some of the changes
**When** the pre-commit hook runs
**Then** lint-staged MUST only process the staged portions
**And** the unstaged changes MUST remain unmodified
**And** the staging area MUST be updated with formatted changes

### Requirement: Performance Optimization

The pre-commit hook system MUST complete formatting checks quickly to avoid disrupting developer workflow.

**Priority**: P2 (Should Have)

#### Scenario: Small commit completes quickly

**Given** a developer has staged 3-5 small files (each < 100 lines)
**When** they run `git commit`
**Then** the pre-commit hook MUST complete within 2 seconds
**And** the developer SHOULD see minimal delay

#### Scenario: Large commit completes reasonably fast

**Given** a developer has staged 20-30 files totaling 2000 lines of code
**When** they run `git commit`
**Then** the pre-commit hook MUST complete within 5 seconds
**And** the formatters SHOULD run in parallel where possible

#### Scenario: Progress feedback is provided

**Given** the pre-commit hook is processing multiple files
**When** the hook runs
**Then** the developer SHOULD see progress indication (e.g., "lint-staged: running on 15 files")
**And** completion SHOULD be indicated (e.g., "lint-staged: done")

### Requirement: Emergency Bypass

The pre-commit hook system MUST allow developers to bypass hooks in emergency situations using a standard Git flag.

**Priority**: P2 (Should Have)

#### Scenario: Developer bypasses hook in emergency

**Given** a developer needs to make an urgent commit (e.g., hotfix to production)
**And** the pre-commit hook is failing or would cause delay
**When** they run `git commit --no-verify -m "emergency: hotfix"`
**Then** the pre-commit hook MUST NOT run
**And** the commit MUST proceed immediately
**And** a warning SHOULD be logged (if possible) noting the hook was bypassed

#### Scenario: Bypass is documented as exceptional

**Given** a developer is reading the project documentation
**When** they look up information about bypassing pre-commit hooks
**Then** the documentation MUST emphasize that `--no-verify` is for emergencies only
**And** MUST explain the risks of bypassing hooks
**And** SHOULD provide examples of appropriate use cases

### Requirement: Hook Maintenance

The pre-commit hook system MUST be version-controlled and automatically updated when developers pull changes.

**Priority**: P2 (Should Have)

#### Scenario: Hook script is version controlled

**Given** the pre-commit hook is installed
**When** a developer pulls changes that include updates to the hook script
**Then** the updated hook MUST be used on subsequent commits
**And** the developer MUST NOT need to manually reinstall hooks

#### Scenario: Husky configuration is updated

**Given** the project uses Husky for hook management
**When** a developer runs `npm install` after pulling changes
**Then** Husky MUST ensure hooks are properly installed
**And** MUST update existing hooks if the configuration has changed

