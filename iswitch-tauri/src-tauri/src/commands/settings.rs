//! [INPUT]:
//!   source: ../services/app_settings.rs ([POS]: App Settings 服务)
//!   source: ../services/import_service.rs ([POS]: Import 服务)
//!   source: ../models/settings.rs ([POS]: Settings 数据模型)
//!
//! [OUTPUT]:
//!   - get_app_settings, save_app_settings commands
//!   - open_logs_window command
//!   - get_import_status, import_config commands
//!
//! [POS]: 设置和导入相关的 Tauri Commands
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::AppResult;
use crate::models::AppSettings;
use crate::services::{
    AppSettingsService, ConfigImportResult, ImportService, ImportStatus, ProviderService,
};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;
use tracing::{debug, error, info, warn};

/// 日志窗口固定 Label
const LOGS_WINDOW_LABEL: &str = "logs";

/// 获取应用设置
#[tauri::command]
pub async fn get_app_settings(app: AppHandle) -> AppResult<AppSettings> {
    debug!("获取应用设置");
    let service = AppSettingsService::new();
    let mut settings = service.get_settings().await?;

    // 同步开机自启动状态
    let manager = app.autolaunch();
    match manager.is_enabled() {
        Ok(enabled) => settings.auto_start = enabled,
        Err(e) => warn!(error = %e, "获取开机自启动状态失败"),
    }

    Ok(settings)
}

/// 保存应用设置
#[tauri::command]
pub async fn save_app_settings(app: AppHandle, settings: AppSettings) -> AppResult<AppSettings> {
    debug!(?settings, "保存应用设置");
    let service = AppSettingsService::new();

    // 同步开机自启动状态
    let manager = app.autolaunch();
    if settings.auto_start {
        if let Err(e) = manager.enable() {
            error!(error = %e, "启用开机自启动失败");
        } else {
            info!("已启用开机自启动");
        }
    } else if let Err(e) = manager.disable() {
        error!(error = %e, "禁用开机自启动失败");
    } else {
        info!("已禁用开机自启动");
    }

    service.save_settings(settings).await
}

/// 打开日志窗口
#[tauri::command]
pub async fn open_logs_window(app: AppHandle) -> AppResult<()> {
    info!("打开日志窗口");

    // 检查是否已存在日志窗口
    if let Some(existing_window) = app.get_webview_window(LOGS_WINDOW_LABEL) {
        info!("日志窗口已存在，聚焦");
        let _ = existing_window.show();
        let _ = existing_window.set_focus();
        return Ok(());
    }

    // 创建新的日志窗口
    let window =
        WebviewWindowBuilder::new(&app, LOGS_WINDOW_LABEL, WebviewUrl::App("/#/logs".into()))
            .title("Logs")
            .inner_size(1024.0, 800.0)
            .min_inner_size(600.0, 300.0)
            .center()
            .visible(true)
            .resizable(true);

    #[cfg(target_os = "macos")]
    let window = window
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true);

    match window.build() {
        Ok(win) => {
            info!(label = LOGS_WINDOW_LABEL, "日志窗口已创建");
            let _ = win.set_focus();
        }
        Err(e) => {
            error!(error = %e, "创建日志窗口失败");
        }
    }

    Ok(())
}

/// 获取导入状态
#[tauri::command]
pub async fn get_import_status(
    _app: AppHandle,
    provider_service: State<'_, Arc<ProviderService>>,
) -> AppResult<ImportStatus> {
    debug!("获取导入状态");

    let mcp_service = std::sync::Arc::new(crate::services::MCPService::new());

    let import_service = ImportService::new(provider_service.inner().clone(), mcp_service);
    import_service.get_status().await
}

/// 导入配置
#[tauri::command]
pub async fn import_config(
    _app: AppHandle,
    provider_service: State<'_, Arc<ProviderService>>,
) -> AppResult<ConfigImportResult> {
    info!("开始导入配置");

    let mcp_service = std::sync::Arc::new(crate::services::MCPService::new());

    let import_service = ImportService::new(provider_service.inner().clone(), mcp_service);
    import_service.import_all().await
}

/// 列出所有可用的导入来源
///
/// 返回所有可检测到的配置来源（cc-switch、code-switch）及其状态
#[tauri::command]
pub async fn list_import_sources(
    _app: AppHandle,
    provider_service: State<'_, Arc<ProviderService>>,
) -> AppResult<Vec<crate::services::ImportSourceInfo>> {
    debug!("列出导入来源");

    let mcp_service = std::sync::Arc::new(crate::services::MCPService::new());

    let import_service = ImportService::new(provider_service.inner().clone(), mcp_service);
    import_service.list_import_sources().await
}

/// 从指定来源导入配置
///
/// 参数 source: "cc_switch" | "code_switch"
#[tauri::command]
pub async fn import_from_source(
    _app: AppHandle,
    source: crate::services::ImportSource,
    provider_service: State<'_, Arc<ProviderService>>,
) -> AppResult<ConfigImportResult> {
    info!(?source, "从指定来源导入配置");

    let mcp_service = std::sync::Arc::new(crate::services::MCPService::new());

    let import_service = ImportService::new(provider_service.inner().clone(), mcp_service);
    import_service.import_from_source(source).await
}

/// 从自定义文件导入配置
#[tauri::command]
pub async fn import_from_file(
    _app: AppHandle,
    path: String,
    provider_service: State<'_, Arc<ProviderService>>,
) -> AppResult<ConfigImportResult> {
    use crate::error::AppError;
    use std::path::Path;

    info!(path = %path, "从自定义文件导入配置");

    // 1. 验证路径格式
    let path_obj = Path::new(&path);

    // 2. 规范化路径，解析所有 `..` 和 `.`
    let canonical_path = path_obj
        .canonicalize()
        .map_err(|_| AppError::InvalidInput("无法解析文件路径".into()))?;

    // 3. 验证文件存在且是普通文件
    if !canonical_path.exists() || !canonical_path.is_file() {
        return Err(AppError::InvalidInput("文件不存在或不是有效文件".into()));
    }

    // 4. 验证文件扩展名（只允许.json）
    if canonical_path.extension().and_then(|s| s.to_str()) != Some("json") {
        return Err(AppError::InvalidInput(
            "只允许导入 JSON 格式的配置文件".into(),
        ));
    }

    // 5. 限制文件大小（防止DoS）
    let metadata = std::fs::metadata(&canonical_path)
        .map_err(|_| AppError::InvalidInput("无法读取文件元数据".into()))?;

    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    if metadata.len() > MAX_FILE_SIZE {
        return Err(AppError::InvalidInput(format!(
            "文件过大（最大允许 10MB，实际 {} 字节）",
            metadata.len()
        )));
    }

    info!(
        path = %canonical_path.display(),
        size = metadata.len(),
        "路径验证通过，开始导入配置"
    );

    let mcp_service = std::sync::Arc::new(crate::services::MCPService::new());

    let import_service = ImportService::new(provider_service.inner().clone(), mcp_service);

    // 将 PathBuf 转换为字符串
    let path_str = canonical_path
        .to_str()
        .ok_or_else(|| AppError::InvalidInput("文件路径包含无效字符".into()))?;

    import_service.import_from_file(path_str).await
}

/// 获取 Code-Switch 导入状态
///
/// 检查 ~/.code-switch/ 目录下的配置文件（claude-code.json, codex.json, mcp.json）
/// 返回待导入的 provider 和 MCP server 数量
#[tauri::command]
pub async fn get_code_switch_import_status(
    _app: AppHandle,
    provider_service: State<'_, Arc<ProviderService>>,
) -> AppResult<ImportStatus> {
    debug!("获取 Code-Switch 导入状态");

    let mcp_service = std::sync::Arc::new(crate::services::MCPService::new());

    let import_service = ImportService::new(provider_service.inner().clone(), mcp_service);
    import_service.get_code_switch_status().await
}

/// 从 Code-Switch 导入配置
///
/// 聚合导入 ~/.code-switch/ 目录下的所有配置文件：
/// - claude-code.json → Claude providers
/// - codex.json → Codex providers
/// - mcp.json → MCP servers
#[tauri::command]
pub async fn import_from_code_switch(
    _app: AppHandle,
    provider_service: State<'_, Arc<ProviderService>>,
) -> AppResult<ConfigImportResult> {
    info!("从 Code-Switch 导入配置");

    let mcp_service = std::sync::Arc::new(crate::services::MCPService::new());

    let import_service = ImportService::new(provider_service.inner().clone(), mcp_service);
    import_service.import_from_code_switch().await
}
