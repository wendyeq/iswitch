.PHONY: dev build install test check clean help lint format format-rust format-docs format-docs-check format-check version bump-patch bump-minor bump-major changelog release

# 项目路径变量
TAURI_ROOT := iswitch-tauri
RUST_ROOT := $(TAURI_ROOT)/src-tauri
PRETTIER_BIN := $(TAURI_ROOT)/node_modules/.bin/prettier

# 默认目标：显示帮助
all: help

help: ## 显示此帮助信息
	@echo "iSwitch 项目常用命令:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# --- 开发流程 ---

install: ## 安装项目依赖 (Node.js)
	@echo "📦 安装前端依赖..."
	cd $(TAURI_ROOT) && npm install

dev: ## 启动 Tauri 开发环境 (热重载)
	@echo "🚀 启动开发服务器..."
	cd $(TAURI_ROOT) && npm run tauri dev

build: ## 构建生产版本安装包 (Mac/Win/Linux)
	@echo "🔨 构建生产版本..."
	cd $(TAURI_ROOT) && npm run tauri build

# --- Rust 后端开发 ---

test: ## 运行 Rust 单元测试
	@echo "🧪 运行后端测试..."
	cd $(RUST_ROOT) && cargo test

test-ui: ## 运行前端单元测试
	@echo "🧪 运行前端测试..."
	cd $(TAURI_ROOT) && npm run test:run

check: ## 运行 Rust 编译检查 (快速)
	@echo "✅ 检查 Rust 代码..."
	cd $(RUST_ROOT) && cargo check

lint: ## 运行 Cargo Clippy (代码风格检查)
	@echo "🔍 运行 Clippy..."
	cd $(RUST_ROOT) && cargo clippy -- -D warnings

format: ## 格式化前端、Rust 代码与文档
	@echo "🎨 格式化前端与配置..."
	cd $(TAURI_ROOT) && npm run format
	@echo "🦀 格式化 Rust 代码..."
	cd $(RUST_ROOT) && cargo fmt
	@$(MAKE) format-docs

format-rust: ## 仅格式化 Rust 代码
	@echo "🦀 格式化 Rust 代码..."
	cd $(RUST_ROOT) && cargo fmt

format-check: ## 检查前端、Rust 代码与文档格式
	@echo "🔍 检查前端格式..."
	cd $(TAURI_ROOT) && npm run format:check
	@echo "🔍 检查 Rust 格式..."
	cd $(RUST_ROOT) && cargo fmt --check
	@$(MAKE) format-docs-check

format-docs: ## 使用 Prettier 格式化仓库 Markdown 文档
	@echo "📄 格式化仓库 Markdown 文档..."
	@DOC_FILES=$$(git ls-files -- 'docs/*.md' 'docs/**/*.md' 'README*.md' 'CLAUDE.md' | grep -v '^$(TAURI_ROOT)/' | sort -u || true); \
	if [ -n "$$DOC_FILES" ]; then \
		$(PRETTIER_BIN) --config $(TAURI_ROOT)/.prettierrc --ignore-path $(TAURI_ROOT)/.prettierignore --write $$DOC_FILES; \
	else \
		echo "没有额外的 Markdown 文档需要格式化"; \
	fi

format-docs-check: ## 检查仓库 Markdown 文档格式
	@echo "📄 检查仓库 Markdown 文档格式..."
	@DOC_FILES=$$(git ls-files -- 'docs/*.md' 'docs/**/*.md' 'README*.md' 'CLAUDE.md' | grep -v '^$(TAURI_ROOT)/' | sort -u || true); \
	if [ -n "$$DOC_FILES" ]; then \
		$(PRETTIER_BIN) --config $(TAURI_ROOT)/.prettierrc --ignore-path $(TAURI_ROOT)/.prettierignore --check $$DOC_FILES; \
	else \
		echo "没有额外的 Markdown 文档需要检查"; \
	fi

# --- 代码覆盖率 ---

coverage: ## 生成测试覆盖率报告 (Html, 需安装 cargo-llvm-cov)
	@echo "📊 生成覆盖率报告..."
	cd $(RUST_ROOT) && rustup run stable cargo llvm-cov --html --output-dir ../../coverage

coverage-open: ## 生成并打开覆盖率报告
	@echo "📊 生成并打开覆盖率报告..."
	cd $(RUST_ROOT) && rustup run stable cargo llvm-cov --html --open --output-dir ../../coverage

coverage-ui: ## 生成前端测试覆盖率报告
	@echo "📊 生成前端覆盖率报告..."
	cd $(TAURI_ROOT) && npm run test:coverage

coverage-ui-open: ## 生成并打开前端测试覆盖率报告 (Web)
	@echo "📊 生成并打开前端覆盖率报告..."
	cd $(TAURI_ROOT) && npm run test:coverage
	@open reports/coverage/frontend/index.html

clean: ## 清理构建产物
	@echo "🧹 清理构建产物..."
	cd $(RUST_ROOT) && cargo clean
	rm -rf $(TAURI_ROOT)/dist

sweep: ## 清理旧构建缓存 (默认 3 天, 可用 make sweep DAYS=7)
	@echo "🧹 清理旧构建缓存 (保留最近 $(or $(DAYS),3) 天)..."
	@if command -v cargo-sweep > /dev/null 2>&1; then \
		cd $(RUST_ROOT) && cargo sweep --time $(or $(DAYS),3); \
	else \
		echo "⚠️  cargo-sweep 未安装，请运行: cargo install cargo-sweep"; \
	fi



# --- OpenSpec / Workflow ---

spec-check: ## 检查 OpenSpec 规范实现状态 (TODO)
	@echo "🚧 OpenSpec 检查脚本待实现"

update-deps: ## 更新所有依赖
	cd $(TAURI_ROOT) && npm update
	cd $(RUST_ROOT) && cargo update

# --- 版本管理 ---

# 操作系统检测
ifeq ($(OS),Windows_NT)
    BUMP_SCRIPT := powershell -ExecutionPolicy Bypass -File scripts/bump-version.ps1
else
    BUMP_SCRIPT := ./scripts/bump-version.sh
endif

version: ## 显示当前版本
ifeq ($(OS),Windows_NT)
	@powershell -Command "(Get-Content $(TAURI_ROOT)/src-tauri/tauri.conf.json | ConvertFrom-Json).version"
else
	@jq -r '.version' $(TAURI_ROOT)/src-tauri/tauri.conf.json
endif

bump-patch: ## 递增补丁版本 (0.1.0 → 0.1.1)
	@$(BUMP_SCRIPT) patch

bump-minor: ## 递增次版本 (0.1.0 → 0.2.0)
	@$(BUMP_SCRIPT) minor

bump-major: ## 递增主版本 (0.1.0 → 1.0.0)
	@$(BUMP_SCRIPT) major

# --- CHANGELOG ---

changelog: ## 生成/更新 CHANGELOG (需安装 git-cliff)
	@if command -v git-cliff > /dev/null 2>&1; then \
		echo "📝 生成 CHANGELOG..."; \
		git-cliff -o CHANGELOG.md; \
	else \
		echo "⚠️  git-cliff 未安装，请运行: cargo install git-cliff"; \
	fi

release: ## 发布新版本 (bump + changelog + commit)
	@if [ -z "$(TYPE)" ]; then \
		echo "用法: make release TYPE=patch|minor|major"; \
		exit 1; \
	fi
	@$(BUMP_SCRIPT) $(TYPE)
	@if command -v git-cliff > /dev/null 2>&1; then \
		git-cliff -o CHANGELOG.md; \
	fi
	@VERSION=$$(jq -r '.version' $(TAURI_ROOT)/src-tauri/tauri.conf.json); \
	git add -A && git commit -m "chore: release v$$VERSION"
