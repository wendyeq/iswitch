# ---
# [INPUT]: {命令行参数} - patch | minor | major | set <version>
# [OUTPUT]: 同步更新 tauri.conf.json, package.json, Cargo.toml 中的版本号
# [POS]: Windows 版本同步脚本，实现版本号单一数据源 (tauri.conf.json)
# [PROTOCOL]: FractalFlow v1.0
# ---
# 版本同步脚本 (PowerShell) - 以 tauri.conf.json 为版本号单一数据源
# 用法:
#   .\scripts\bump-version.ps1 patch       # 0.1.0 → 0.1.1
#   .\scripts\bump-version.ps1 minor       # 0.1.0 → 0.2.0
#   .\scripts\bump-version.ps1 major       # 0.1.0 → 1.0.0
#   .\scripts\bump-version.ps1 set 2.0.0   # 设置特定版本

param(
    [Parameter(Position = 0)]
    [ValidateSet("patch", "minor", "major", "set", "help", "-h", "--help")]
    [string]$Action = "help",
    
    [Parameter(Position = 1)]
    [string]$Version
)

# ============================================================
# 路径配置
# ============================================================
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$TauriConf = Join-Path $ProjectRoot "iswitch-tauri\src-tauri\tauri.conf.json"
$PackageJson = Join-Path $ProjectRoot "iswitch-tauri\package.json"
$CargoToml = Join-Path $ProjectRoot "iswitch-tauri\src-tauri\Cargo.toml"

# ============================================================
# 辅助函数
# ============================================================

function Write-ColorOutput {
    param(
        [string]$Message,
        [ConsoleColor]$Color = [ConsoleColor]::White
    )
    $originalColor = $Host.UI.RawUI.ForegroundColor
    $Host.UI.RawUI.ForegroundColor = $Color
    Write-Output $Message
    $Host.UI.RawUI.ForegroundColor = $originalColor
}

function Show-Usage {
    Write-ColorOutput "用法:" -Color Blue
    Write-Output "  .\scripts\bump-version.ps1 patch         递增补丁版本 (0.1.0 → 0.1.1)"
    Write-Output "  .\scripts\bump-version.ps1 minor         递增次版本   (0.1.0 → 0.2.0)"
    Write-Output "  .\scripts\bump-version.ps1 major         递增主版本   (0.1.0 → 1.0.0)"
    Write-Output "  .\scripts\bump-version.ps1 set <version> 设置特定版本 (例如: set 2.0.0)"
    Write-Output ""
    Write-ColorOutput "示例:" -Color Blue
    Write-Output "  .\scripts\bump-version.ps1 patch"
    Write-Output "  .\scripts\bump-version.ps1 set 1.2.3"
}

function Test-SemVer {
    param([string]$Ver)
    return $Ver -match "^\d+\.\d+\.\d+$"
}

function Get-CurrentVersion {
    $config = Get-Content $TauriConf -Raw | ConvertFrom-Json
    return $config.version
}

function Get-BumpedVersion {
    param(
        [string]$CurrentVersion,
        [string]$Type
    )
    
    $parts = $CurrentVersion -split "\."
    $major = [int]$parts[0]
    $minor = [int]$parts[1]
    $patch = [int]$parts[2]
    
    switch ($Type) {
        "major" {
            $major++
            $minor = 0
            $patch = 0
        }
        "minor" {
            $minor++
            $patch = 0
        }
        "patch" {
            $patch++
        }
    }
    
    return "$major.$minor.$patch"
}

function Update-TauriConf {
    param([string]$NewVersion)
    
    $config = Get-Content $TauriConf -Raw | ConvertFrom-Json
    $config.version = $NewVersion
    $config | ConvertTo-Json -Depth 10 | Set-Content $TauriConf -Encoding UTF8
    
    Write-ColorOutput "  ✓ tauri.conf.json" -Color Green
}

function Update-PackageJson {
    param([string]$NewVersion)
    
    $package = Get-Content $PackageJson -Raw | ConvertFrom-Json
    $package.version = $NewVersion
    $package | ConvertTo-Json -Depth 10 | Set-Content $PackageJson -Encoding UTF8
    
    Write-ColorOutput "  ✓ package.json" -Color Green
}

function Update-CargoToml {
    param([string]$NewVersion)
    
    $content = Get-Content $CargoToml -Raw
    $content = $content -replace 'version = "\d+\.\d+\.\d+"', "version = `"$NewVersion`""
    Set-Content $CargoToml -Value $content -Encoding UTF8
    
    Write-ColorOutput "  ✓ Cargo.toml" -Color Green
}

# ============================================================
# 主逻辑
# ============================================================

# 显示帮助
if ($Action -in @("help", "-h", "--help")) {
    Show-Usage
    exit 0
}

# 获取当前版本
$CurrentVersion = Get-CurrentVersion

# 计算新版本
$NewVersion = ""
switch ($Action) {
    "patch" { $NewVersion = Get-BumpedVersion -CurrentVersion $CurrentVersion -Type "patch" }
    "minor" { $NewVersion = Get-BumpedVersion -CurrentVersion $CurrentVersion -Type "minor" }
    "major" { $NewVersion = Get-BumpedVersion -CurrentVersion $CurrentVersion -Type "major" }
    "set" {
        if (-not $Version) {
            Write-ColorOutput "错误: 'set' 命令需要提供版本号" -Color Red
            Show-Usage
            exit 1
        }
        if (-not (Test-SemVer -Ver $Version)) {
            Write-ColorOutput "错误: 版本号格式不正确" -Color Red
            Write-Output "版本号必须遵循语义化版本规范 (SemVer): MAJOR.MINOR.PATCH"
            Write-Output "示例: 1.0.0, 2.3.1, 0.1.0"
            exit 1
        }
        $NewVersion = $Version
    }
}

# 显示版本变更信息
Write-ColorOutput "版本更新: " -Color Blue
Write-Output "$CurrentVersion → $NewVersion"
Write-Output ""
Write-ColorOutput "正在同步文件..." -Color Blue

# 更新所有文件
Update-TauriConf -NewVersion $NewVersion
Update-PackageJson -NewVersion $NewVersion
Update-CargoToml -NewVersion $NewVersion

Write-Output ""
Write-ColorOutput "✓ 版本已更新为 $NewVersion" -Color Green
Write-Output ""
Write-ColorOutput "提示: " -Color Yellow
Write-Output "建议运行以下命令提交更改:"
Write-Output "  git add -A; git commit -m `"chore: bump version to $NewVersion`""
