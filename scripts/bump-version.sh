#!/bin/bash
# ---
# [INPUT]: {命令行参数} - patch | minor | major | set <version>
# [OUTPUT]: 同步更新 tauri.conf.json, package.json, Cargo.toml 中的版本号
# [POS]: 版本同步脚本，实现版本号单一数据源 (tauri.conf.json)
# [PROTOCOL]: FractalFlow v1.0
# ---
# 版本同步脚本 - 以 tauri.conf.json 为版本号单一数据源
# 用法:
#   ./scripts/bump-version.sh patch       # 0.1.0 → 0.1.1
#   ./scripts/bump-version.sh minor       # 0.1.0 → 0.2.0
#   ./scripts/bump-version.sh major       # 0.1.0 → 1.0.0
#   ./scripts/bump-version.sh set 2.0.0   # 设置特定版本

set -e

# ============================================================
# 颜色定义
# ============================================================
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================
# 路径配置
# ============================================================
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_CONF="$PROJECT_ROOT/iswitch-tauri/src-tauri/tauri.conf.json"
PACKAGE_JSON="$PROJECT_ROOT/iswitch-tauri/package.json"
CARGO_TOML="$PROJECT_ROOT/iswitch-tauri/src-tauri/Cargo.toml"

# ============================================================
# 辅助函数
# ============================================================

print_usage() {
    echo -e "${BLUE}用法:${NC}"
    echo "  $0 patch         递增补丁版本 (0.1.0 → 0.1.1)"
    echo "  $0 minor         递增次版本   (0.1.0 → 0.2.0)"
    echo "  $0 major         递增主版本   (0.1.0 → 1.0.0)"
    echo "  $0 set <version> 设置特定版本 (例如: set 2.0.0)"
    echo ""
    echo -e "${BLUE}示例:${NC}"
    echo "  $0 patch"
    echo "  $0 set 1.2.3"
}

check_dependencies() {
    if ! command -v jq &> /dev/null; then
        echo -e "${RED}错误: jq 未安装${NC}"
        echo "请运行: brew install jq"
        exit 1
    fi
}

validate_semver() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo -e "${RED}错误: 版本号格式不正确${NC}"
        echo "版本号必须遵循语义化版本规范 (SemVer): MAJOR.MINOR.PATCH"
        echo "示例: 1.0.0, 2.3.1, 0.1.0"
        exit 1
    fi
}

get_current_version() {
    jq -r '.version' "$TAURI_CONF"
}

# 递增版本号
# $1: 当前版本
# $2: 类型 (patch | minor | major)
bump_version() {
    local version="$1"
    local type="$2"
    
    IFS='.' read -r major minor patch <<< "$version"
    
    case "$type" in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            echo -e "${RED}错误: 未知的版本类型 '$type'${NC}"
            exit 1
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# 更新 tauri.conf.json
update_tauri_conf() {
    local version="$1"
    local tmp_file=$(mktemp)
    
    jq --arg v "$version" '.version = $v' "$TAURI_CONF" > "$tmp_file"
    mv "$tmp_file" "$TAURI_CONF"
    
    echo -e "  ${GREEN}✓${NC} tauri.conf.json"
}

# 更新 package.json
update_package_json() {
    local version="$1"
    local tmp_file=$(mktemp)
    
    jq --arg v "$version" '.version = $v' "$PACKAGE_JSON" > "$tmp_file"
    mv "$tmp_file" "$PACKAGE_JSON"
    
    echo -e "  ${GREEN}✓${NC} package.json"
}

# 更新 Cargo.toml
update_cargo_toml() {
    local version="$1"
    
    # 使用 sed 替换 [package] 区块中的 version
    # 注意: macOS 的 sed 和 GNU sed 语法略有不同
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$version\"/" "$CARGO_TOML"
    else
        sed -i "s/^version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$version\"/" "$CARGO_TOML"
    fi
    
    echo -e "  ${GREEN}✓${NC} Cargo.toml"
}

# ============================================================
# 主逻辑
# ============================================================

main() {
    check_dependencies
    
    if [[ $# -lt 1 ]]; then
        print_usage
        exit 1
    fi
    
    local action="$1"
    local current_version
    local new_version
    
    current_version=$(get_current_version)
    
    case "$action" in
        patch|minor|major)
            new_version=$(bump_version "$current_version" "$action")
            ;;
        set)
            if [[ $# -lt 2 ]]; then
                echo -e "${RED}错误: 'set' 命令需要提供版本号${NC}"
                print_usage
                exit 1
            fi
            new_version="$2"
            validate_semver "$new_version"
            ;;
        -h|--help|help)
            print_usage
            exit 0
            ;;
        *)
            echo -e "${RED}错误: 未知的命令 '$action'${NC}"
            print_usage
            exit 1
            ;;
    esac
    
    echo -e "${BLUE}版本更新:${NC} $current_version → ${GREEN}$new_version${NC}"
    echo ""
    echo -e "${BLUE}正在同步文件...${NC}"
    
    update_tauri_conf "$new_version"
    update_package_json "$new_version"
    update_cargo_toml "$new_version"
    
    echo ""
    echo -e "${GREEN}✓ 版本已更新为 $new_version${NC}"
    echo ""
    echo -e "${YELLOW}提示:${NC} 建议运行以下命令提交更改:"
    echo "  git add -A && git commit -m \"chore: bump version to $new_version\""
}

main "$@"
