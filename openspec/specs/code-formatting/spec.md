# code-formatting Specification

## Purpose
TBD - created by archiving change add-code-formatting-and-pre-commit-hooks. Update Purpose after archive.
## Requirements
### Requirement: Frontend Code Formatting

The project MUST provide automated formatting for all frontend code files including Vue components, TypeScript, and JavaScript.

**Priority**: P1 (Must Have)

#### Scenario: Developer formats Vue component manually

**Given** a developer has modified a Vue component file with inconsistent formatting
**When** the developer runs `npm run format` in the `iswitch-tauri/` directory
**Then** the Vue file SHOULD be formatted according to `.prettierrc` configuration
**And** the file MUST use:
- Single quotes for strings
- 2-space indentation
- 120 character line width
- Trailing commas where applicable
- Semicolons for all statements

#### Scenario: Developer formats TypeScript service file

**Given** a developer has written a TypeScript service file with mixed formatting styles
**When** the developer runs `npm run format` or saves the file with format-on-save enabled
**Then** the TypeScript file MUST be reformatted to match project standards
**And** imports SHOULD be grouped and sorted
**And** object properties MUST use consistent syntax (shorthand where applicable)

#### Scenario: Developer checks formatting without modifying files

**Given** a developer wants to verify formatting before committing
**When** the developer runs `npm run format:check`
**Then** the system MUST report any files that don't match formatting standards
**And** MUST NOT modify any files
**And** MUST exit with error code if formatting issues exist

### Requirement: Backend Code Formatting

The project MUST provide automated formatting for all Rust backend code using rustfmt.

**Priority**: P1 (Must Have)

#### Scenario: Developer formats Rust module manually

**Given** a developer has modified a Rust service module with inconsistent formatting
**When** the developer runs `cargo fmt` in the `iswitch-tauri/src-tauri/` directory
**Then** all `.rs` files SHOULD be formatted according to `rustfmt.toml` configuration
**And** the code MUST use:
- 4-space indentation
- 100 character line width
- Unix-style line endings
- Edition 2021 syntax

#### Scenario: Rust formatter reorders imports

**Given** a Rust file has imports declared in random order
**When** the developer runs `cargo fmt`
**Then** imports MUST be reordered alphabetically and grouped
**And** nested `use` statements MUST be flattened where appropriate
**And** external vs internal imports MUST be separated

#### Scenario: Developer checks Rust formatting without modifying

**Given** a developer wants to verify Rust formatting
**When** the developer runs `cargo fmt --check`
**Then** the system MUST report any files that don't match formatting standards
**And** MUST NOT modify any files
**And** MUST exit with error code if formatting issues exist

### Requirement: Documentation Formatting

The project MUST provide automated formatting for all Markdown documentation files.

**Priority**: P2 (Should Have)

#### Scenario: Developer formats Markdown documentation

**Given** a developer has edited a Markdown file with inconsistent heading styles or list formatting
**When** the developer runs `npm run format` or `make format`
**Then** the Markdown file SHOULD be formatted according to Prettier Markdown rules
**And** headings MUST maintain consistent hierarchy (no skipped levels)
**And** lists MUST use consistent formatting (unordered vs ordered)
**And** code blocks MUST include language specification where applicable

#### Scenario: Auto-generated files are excluded from formatting

**Given** the project contains auto-generated files like `CHANGELOG.md` or `package-lock.json`
**When** formatting is run
**Then** these files MUST be excluded from formatting as defined in `.prettierignore`
**And** their content MUST remain unchanged

### Requirement: Configuration Files

The project MUST include configuration files that define consistent formatting rules across all file types.

**Priority**: P1 (Must Have)

#### Scenario: Prettier configuration exists for frontend

**Given** a new developer joins the project
**When** they open the `iswitch-tauri/.prettierrc` file
**Then** the file MUST contain complete Prettier configuration
**And** MUST specify all formatting rules (quotes, indentation, line width, etc.)
**And** MUST be valid JSON format

#### Scenario: rustfmt configuration exists for backend

**Given** a developer wants to verify Rust formatting rules
**When** they open the `rustfmt.toml` file in project root
**Then** the file MUST contain complete rustfmt configuration
**And** MUST specify max width, indentation, and other Rust-specific rules
**And** MUST be valid TOML format

#### Scenario: Ignore files exclude generated content

**Given** the project has `.prettierignore` and potentially `.rustfmt.toml` exclusions
**When** formatters run on the codebase
**Then** files matching patterns in ignore files MUST be skipped
**And** common exclusions MUST include: `node_modules/`, `dist/`, `build/`, `coverage/`, `*.min.js`

### Requirement: Cross-Project Formatting Command

The project MUST provide a command to format all project code (frontend + backend + docs) from the project root.

**Priority**: P2 (Should Have)

#### Scenario: Developer formats entire project

**Given** a developer wants to ensure all code follows formatting standards
**When** the developer runs `make format` from the project root
**Then** the system MUST format frontend code (Vue/TS/JS)
**And** MUST format backend code (Rust)
**And** MUST format documentation (Markdown)
**And** MUST report the number of files formatted

#### Scenario: Developer checks all project formatting

**Given** a developer wants to verify all code formatting before committing
**When** the developer runs `make format:check` from project root
**And** any formatting issues exist
**Then** the system MUST report all files with formatting violations
**And** MUST exit with error code to indicate failure
**And** MUST NOT modify any files

