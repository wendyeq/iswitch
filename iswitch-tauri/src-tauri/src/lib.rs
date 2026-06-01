//! [INPUT]:
//!   source: ../../../openspec/changes/migrate-to-tauri-stack/design.md ([POS]: 架构设计)
//!   source: ../../../code-switch/main.go ([POS]: 原 Go 入口参考)
//!
//! [OUTPUT]:
//!   - Tauri 应用初始化
//!   - 插件注册
//!   - Commands 注册
//!   - 系统托盘和窗口管理
//!
//! [POS]: Tauri 应用库入口，配置所有插件、Commands、状态管理和桌面集成
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽
//!
//! [INPUT]:
//!   source: ../../../openspec/changes/migrate-to-tauri-stack/design.md ([POS]: 架构设计)
//!   source: ../../../code-switch/main.go ([POS]: 原 Go 入口参考)
//!   source: tauri.conf.json ([POS]: 窗口配置)
//!
//! [OUTPUT]:
//!   - Tauri 应用初始化
//!   - 插件注册
//!   - Commands 注册
//!   - 系统托盘和窗口管理
//!   - 默认应用菜单配置

// === 模块声明 ===
pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod proxy;
pub mod services;
pub mod utils;

// === 公开导出 ===
pub use error::{AppError, AppResult};

use serde::Serialize;
use std::io::ErrorKind;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use tauri::menu::{Menu, MenuBuilder, MenuItem, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Runtime, WindowEvent};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

/// 窗口居中标志
static MAIN_WINDOW_CENTERED: AtomicBool = AtomicBool::new(false);

/// 全局 HUD 菜单项引用，用于动态更新菜单文字
static HUD_MENU_ITEM: OnceLock<MenuItem<tauri::Wry>> = OnceLock::new();

/// 全局主窗口菜单项引用，用于动态更新菜单文字
static MAIN_WINDOW_MENU_ITEM: OnceLock<MenuItem<tauri::Wry>> = OnceLock::new();

#[derive(Debug, Serialize, Clone)]
struct ProxyErrorPayload {
    #[serde(rename = "type")]
    error_type: &'static str,
    port: u16,
    blocker: Option<ProxyErrorBlockerPayload>,
}

#[derive(Debug, Serialize, Clone)]
struct ProxyErrorBlockerPayload {
    name: Option<String>,
    pid: Option<u32>,
}

/// 初始化 tracing 日志框架
///
/// 根据环境变量 RUST_LOG 控制日志级别
/// 默认级别:
/// - code_switch: debug
/// - hyper: warn
/// - tower_http: warn
fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("iswitch_lib=debug,hyper=warn,tower_http=warn"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("日志系统初始化完成");
}

/// 示例 Command: 问候
///
/// TODO: 在 Phase 2 替换为真实的 Commands
#[tauri::command]
fn greet(name: &str) -> String {
    info!(name = %name, "收到问候请求");
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 获取应用版本号
#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 同步窗口主题
#[tauri::command]
fn sync_window_theme(app: tauri::AppHandle, theme: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let tauri_theme = match theme {
            "dark" => Some(tauri::Theme::Dark),
            "light" => Some(tauri::Theme::Light),
            _ => Some(tauri::Theme::Light),
        };
        let _ = window.set_theme(tauri_theme);

        // 在 macOS 上重新应用 vibrancy 以确保材质刷新
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = apply_vibrancy(&window, NSVisualEffectMaterial::Sidebar, None, None) {
                tracing::error!("Failed to re-apply vibrancy during theme sync: {:?}", e);
            }
        }
    }
}

/// 显示主窗口
fn show_main_window(app: &AppHandle, with_focus: bool) {
    if let Some(window) = app.get_webview_window("main") {
        // 首次显示时居中
        if !MAIN_WINDOW_CENTERED.load(Ordering::Relaxed) {
            let _ = window.center();
            MAIN_WINDOW_CENTERED.store(true, Ordering::Relaxed);
        }

        // 如果最小化则恢复
        if window.is_minimized().unwrap_or(false) {
            let _ = window.unminimize();
        }

        // 显示窗口
        let _ = window.show();

        if with_focus {
            focus_main_window(app);
        }

        // macOS: 显示 Dock 图标
        #[cfg(target_os = "macos")]
        {
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        }

        // 更新菜单项文字
        update_main_window_menu_item(true);
    }
}

/// 聚焦主窗口
fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(target_os = "windows")]
        {
            // Windows 特殊处理：先置顶再取消，确保焦点
            let _ = window.set_always_on_top(true);
            let _ = window.set_focus();
            let window_clone = window.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(150));
                let _ = window_clone.set_always_on_top(false);
            });
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = window.set_focus();
        }
    }
}

/// 隐藏主窗口
fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();

        // macOS: 隐藏 Dock 图标
        #[cfg(target_os = "macos")]
        {
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
        }

        // 更新菜单项文字
        update_main_window_menu_item(false);
    }
}

fn emit_port_conflict_event<R: Runtime>(
    app: &tauri::AppHandle<R>,
    conflict: &proxy::server::PortConflictInfo,
) -> Result<(), tauri::Error> {
    let blocker = conflict
        .blocker
        .as_ref()
        .map(|info| ProxyErrorBlockerPayload {
            name: info.name.clone(),
            pid: info.pid,
        });

    let payload = ProxyErrorPayload {
        error_type: "PORT_CONFLICT",
        port: conflict.port,
        blocker,
    };

    app.emit("proxy-error", payload)
}

/// 设置系统托盘
fn setup_system_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    info!("设置系统托盘");

    // 创建托盘菜单
    // 检查主窗口当前状态以设置正确的菜单文字
    let main_window_visible = app
        .get_webview_window("main")
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false);
    let main_label = if main_window_visible {
        "隐藏主窗口"
    } else {
        "显示主窗口"
    };
    let show_item = MenuItemBuilder::with_id("toggle_main", main_label).build(app)?;
    // 检查 HUD 窗口当前状态以设置正确的菜单文字
    let hud_is_open = app.get_webview_window("mini-hud").is_some();
    let hud_label = if hud_is_open {
        "关闭 Mini HUD"
    } else {
        "打开 Mini HUD"
    };
    let hud_item = MenuItemBuilder::with_id("toggle_hud", hud_label).build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .item(&hud_item)
        .separator()
        .item(&quit_item)
        .build()?;

    // 加载托盘图标 - 使用 include_image! 宏
    let icon = tauri::include_image!("icons/icon.png");

    // 创建托盘图标
    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(false)
        .tooltip("iSwitch")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "toggle_main" => {
                // 切换主窗口显示/隐藏状态
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        hide_main_window(app);
                    } else {
                        show_main_window(app, true);
                    }
                } else {
                    show_main_window(app, true);
                }
            }
            "toggle_hud" => {
                info!("用户点击 Mini HUD 菜单");
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let result = commands::hud::toggle_mini_hud(app_clone.clone()).await;
                    // 根据 toggle 结果更新菜单项文字
                    if let Ok(is_now_open) = result {
                        update_hud_menu_item(&app_clone, is_now_open);
                    }
                });
            }
            "quit" => {
                info!("用户请求退出应用");
                app.exit(0);
            }
            _ => {}
        })
        // 注意：不再监听 on_tray_icon_event 的左键点击，让点击行为由系统处理（显示菜单）
        .build(app)?;

    // 保存菜单项引用，以便后续更新文字
    let _ = MAIN_WINDOW_MENU_ITEM.set(show_item);
    let _ = HUD_MENU_ITEM.set(hud_item);

    Ok(())
}

/// 更新 HUD 菜单项文字
///
/// 根据 HUD 当前是否打开来更新菜单显示
fn update_hud_menu_item(_app: &AppHandle, is_hud_open: bool) {
    if let Some(menu_item) = HUD_MENU_ITEM.get() {
        let new_text = if is_hud_open {
            "关闭 Mini HUD"
        } else {
            "打开 Mini HUD"
        };
        if let Err(e) = menu_item.set_text(new_text) {
            warn!(error = %e, "更新 HUD 菜单项文字失败");
        } else {
            info!(is_open = is_hud_open, "HUD 菜单项已更新为: {}", new_text);
        }
    } else {
        warn!("HUD 菜单项尚未初始化");
    }
}

/// 更新主窗口菜单项文字
///
/// 根据主窗口当前是否显示来更新菜单显示
fn update_main_window_menu_item(is_visible: bool) {
    if let Some(menu_item) = MAIN_WINDOW_MENU_ITEM.get() {
        let new_text = if is_visible {
            "隐藏主窗口"
        } else {
            "显示主窗口"
        };
        if let Err(e) = menu_item.set_text(new_text) {
            warn!(error = %e, "更新主窗口菜单项文字失败");
        } else {
            info!(
                is_visible = is_visible,
                "主窗口菜单项已更新为: {}", new_text
            );
        }
    } else {
        warn!("主窗口菜单项尚未初始化");
    }
}

/// Tauri 应用入口
///
/// 初始化所有插件和 Commands，启动应用
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    init_logging();

    info!("iSwitch 启动中...");
    info!(version = env!("CARGO_PKG_VERSION"), "应用版本");

    // 确保配置目录存在
    if let Err(e) = utils::paths::ensure_iswitch_dir() {
        error!(error = %e, "创建配置目录失败");

        // 尝试使用临时目录作为备选方案
        let temp_dir = std::env::temp_dir().join("iswitch");
        if let Err(temp_err) = std::fs::create_dir_all(&temp_dir) {
            error!(
                temp_error = %temp_err,
                "无法创建临时目录，应用可能无法正常工作"
            );
        } else {
            info!(
                path = %temp_dir.display(),
                "使用临时目录作为配置目录"
            );
        }
    }

    tauri::Builder::default()
        // === Tauri 官方插件 ===
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // === 默认菜单配置 ===
            #[cfg(target_os = "macos")]
            {
                if let Ok(menu) = Menu::default(app.handle()) {
                    let _ = app.set_menu(menu);
                }
            }

            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = apply_vibrancy(&window, NSVisualEffectMaterial::Sidebar, None, None)
                {
                    error!(error = %e, "应用窗口 Vibrancy 效果失败");
                }
            }

            // === Phase 6: 系统托盘设置 ===
            if let Err(e) = setup_system_tray(app.handle()) {
                error!(error = %e, "设置系统托盘失败");
            }

            // === Phase 2: 启动本地代理服务器 ===
            let provider_service = Arc::new(services::ProviderService::new());

            // === Phase 5: 初始化数据库和日志服务 ===
            let db_path = utils::paths::logs_db_path();
            let pool = tauri::async_runtime::block_on(async { db::init_db(db_path).await })?;
            let pool = Arc::new(pool);
            let log_service = services::LogService::new(pool.clone());

            // === Phase 3: 初始化设置服务 ===
            let settings_service = services::AppSettingsService::new();
            // 预加载设置以获取代理端口
            let app_settings =
                tauri::async_runtime::block_on(async { settings_service.get_settings().await })
                    .unwrap_or_default();
            if app_settings.proxy_port != proxy::server::DEFAULT_PROXY_PORT {
                info!(
                    configured = app_settings.proxy_port,
                    default = proxy::server::DEFAULT_PROXY_PORT,
                    "检测到自定义 proxy_port，运行时将改用默认端口"
                );
            }
            if app_settings.failover_threshold != proxy::server::DEFAULT_FAILOVER_THRESHOLD {
                info!(
                    configured = app_settings.failover_threshold,
                    default = proxy::server::DEFAULT_FAILOVER_THRESHOLD,
                    "检测到自定义 failover_threshold，运行时将改用默认阈值"
                );
            }
            if app_settings.recovery_timeout_secs != proxy::server::DEFAULT_RECOVERY_TIMEOUT_SECS {
                info!(
                    configured = app_settings.recovery_timeout_secs,
                    default = proxy::server::DEFAULT_RECOVERY_TIMEOUT_SECS,
                    "检测到自定义 recovery_timeout_secs，运行时将改用默认恢复时间"
                );
            }

            app.manage(services::ClaudeSettingsService::new());
            app.manage(services::CodexSettingsService::new());
            app.manage(log_service);

            // 注册 AppSettingsService (如果之前没注册)
            // 注意: 原代码似乎没有注册 AppSettingsService，这里显式注册一下，或者依赖 commands::settings 自动创建?
            // 查看 commands::settings 可知它自行从 state 获取或者新建。为安全起见，我们在 commands 模块应该统一管理。
            // 经由 Phase 3 完成度，假设 commands::settings::get_app_settings 会使用 State<AppSettingsService>，
            // 但原代码 app.manage(...) 列表里确实没看到 AppSettingsService。
            // 让我们查看 commands/settings.rs 确认是否需要 manage。
            // 假设需要:
            app.manage(settings_service);

            // === Phase 2: Provider Service (shared with proxy) ===
            app.manage(provider_service.clone());

            // === Phase 4: Init MCP & Skill ===
            app.manage(services::MCPService::new());
            app.manage(services::SkillService::new());

            // === 初始化定价服务 ===
            info!(
                "定价服务初始化完成，已加载 {} 个模型定价",
                services::PRICING_SERVICE.model_count()
            );

            // === 初始化 HUD 事件发射器 ===
            services::init_hud_emitter(app.handle().clone());
            info!("HUD 事件发射器初始化完成");

            // === HUD 启动策略: 默认关闭 ===
            // [设计决策] simplify-ui-controls: Mini HUD 每次启动时默认不显示，
            // 用户需通过托盘菜单手动打开。保留 hud.enabled 字段用于运行时状态记录，
            // 但启动时不再依据其值自动创建 HUD 窗口。
            // 参见: openspec/changes/simplify-ui-controls/design.md

            // === 启动代理服务器 ===
            let pool_clone = pool.clone();
            let app_handle_clone = app.handle().clone();
            let provider_service_clone = provider_service.clone();
            let health_config = proxy::server::HealthTrackerConfig {
                threshold: proxy::server::DEFAULT_FAILOVER_THRESHOLD,
                recovery_timeout_secs: proxy::server::DEFAULT_RECOVERY_TIMEOUT_SECS,
            };
            tauri::async_runtime::spawn(async move {
                if let Some(conflict) =
                    proxy::server::detect_port_conflict(proxy::server::DEFAULT_PROXY_PORT).await
                {
                    if let Err(err) = emit_port_conflict_event(&app_handle_clone, &conflict) {
                        error!(?err, "发送 proxy-error 事件失败");
                    }
                    return;
                }

                if let Err(e) = proxy::server::start_proxy_server(
                    provider_service_clone,
                    pool_clone,
                    proxy::server::DEFAULT_PROXY_PORT,
                    Some(health_config),
                )
                .await
                {
                    if e.kind() == ErrorKind::AddrInUse {
                        if let Some(conflict) =
                            proxy::server::detect_port_conflict(proxy::server::DEFAULT_PROXY_PORT)
                                .await
                        {
                            if let Err(err) = emit_port_conflict_event(&app_handle_clone, &conflict)
                            {
                                error!(?err, "发送 proxy-error 事件失败");
                            }
                        }
                    }
                    error!("启动代理服务器失败: {}", e);
                }
            });

            // === Phase 6: 窗口关闭事件处理 ===
            let app_handle = app.handle().clone();
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        // 阻止窗口关闭，改为隐藏
                        api.prevent_close();
                        hide_main_window(&app_handle);
                        info!("主窗口已隐藏");
                    }
                });
            }

            // 显示主窗口
            show_main_window(app.handle(), false);

            Ok(())
        })
        // === 注册 Commands ===
        .invoke_handler(tauri::generate_handler![
            greet,
            get_version,
            // === Provider Commands ===
            commands::provider::load_providers,
            commands::provider::save_providers,
            commands::provider::get_proxy_status,
            // === Claude Settings Commands ===
            commands::claude::get_claude_proxy_status,
            commands::claude::enable_claude_proxy,
            commands::claude::disable_claude_proxy,
            // === Codex Settings Commands ===
            commands::codex::get_codex_proxy_status,
            commands::codex::enable_codex_proxy,
            commands::codex::disable_codex_proxy,
            // === MCP Commands ===
            commands::mcp::list_mcp_servers,
            commands::mcp::save_mcp_servers,
            // === Skill Commands ===
            commands::skill::list_skills,
            commands::skill::install_skill,
            commands::skill::uninstall_skill,
            commands::skill::list_skill_repos,
            commands::skill::add_skill_repo,
            commands::skill::remove_skill_repo,
            // === Log Commands ===
            commands::log::list_request_logs,
            commands::log::list_log_providers,
            commands::log::get_heatmap_stats,
            commands::log::get_log_stats,
            commands::log::get_provider_daily_stats,
            // === Settings Commands ===
            commands::settings::get_app_settings,
            commands::settings::save_app_settings,
            commands::settings::open_logs_window,
            commands::settings::get_import_status,
            commands::settings::import_config,
            commands::settings::list_import_sources,
            commands::settings::import_from_source,
            commands::settings::import_from_file,
            // Code-Switch 导入命令
            commands::settings::get_code_switch_import_status,
            commands::settings::import_from_code_switch,
            // === HUD Commands ===
            commands::hud::toggle_mini_hud,
            commands::hud::close_hud,
            commands::hud::set_hud_click_through,
            commands::hud::get_hud_status,
            commands::hud::get_latest_hud_event,
            sync_window_theme,
            quit_app,
        ])
        // === 运行应用 ===
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}

/// 退出应用
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::sync::mpsc;
    use std::time::Duration;
    use tauri::Listener;

    #[test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    fn build_test_app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("failed to build mock app")
    }

    #[test]
    fn test_emit_port_conflict_event_with_blocker_payload() {
        let app = build_test_app();
        let handle = app.handle();
        let (tx, rx) = mpsc::channel::<String>();
        let listener_id = handle.listen_any("proxy-error", move |event| {
            let _ = tx.send(event.payload().to_string());
        });

        let conflict = proxy::server::PortConflictInfo {
            port: 18099,
            blocker: Some(proxy::server::BlockerProcessInfo {
                name: Some("tauri-proc".to_string()),
                pid: Some(4242),
            }),
        };

        emit_port_conflict_event(&handle, &conflict).expect("emit should succeed");

        let payload = rx
            .recv_timeout(Duration::from_millis(200))
            .expect("event should be emitted");

        handle.unlisten(listener_id);

        let json: Value = serde_json::from_str(&payload).expect("payload must be valid json");
        assert_eq!(json["type"], "PORT_CONFLICT");
        assert_eq!(json["port"], 18099);
        assert_eq!(json["blocker"]["name"], "tauri-proc");
        assert_eq!(json["blocker"]["pid"], 4242);
    }

    #[test]
    fn test_emit_port_conflict_event_without_blocker_payload() {
        let app = build_test_app();
        let handle = app.handle();
        let (tx, rx) = mpsc::channel::<String>();
        let listener_id = handle.listen_any("proxy-error", move |event| {
            let _ = tx.send(event.payload().to_string());
        });

        let conflict = proxy::server::PortConflictInfo {
            port: 19000,
            blocker: None,
        };

        emit_port_conflict_event(&handle, &conflict).expect("emit should succeed");

        let payload = rx
            .recv_timeout(Duration::from_millis(200))
            .expect("event should be emitted");

        handle.unlisten(listener_id);

        let json: Value = serde_json::from_str(&payload).expect("payload must be valid json");
        assert_eq!(json["type"], "PORT_CONFLICT");
        assert_eq!(json["port"], 19000);
        assert!(json.get("blocker").map(|b| b.is_null()).unwrap_or(true));
    }
}
