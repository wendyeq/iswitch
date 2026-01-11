//! [INPUT]:
//!   source: ../services/app_settings.rs ([POS]: App Settings 服务)
//!   source: ../proxy/events.rs ([POS]: HUD 事件结构)
//!   source: ../../../../openspec/changes/simplify-mini-hud/specs/desktop/spec.md ([POS]: HUD 窗口规范)
//!
//! [OUTPUT]:
//!   - toggle_mini_hud: 切换 HUD 显示状态
//!   - close_hud: 关闭 HUD 窗口
//!   - set_hud_click_through: 设置 Click-Through 模式 (仅 macOS)
//!
//! [POS]: Mini HUD 窗口管理 Commands
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::AppResult;
use crate::models::HudSettings;
use crate::proxy::events::HudEvent;
use crate::services::{hud_service, AppSettingsService};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tracing::{debug, error, info, warn};

/// HUD 窗口固定 Label
const HUD_WINDOW_LABEL: &str = "mini-hud";

/// HUD 窗口默认尺寸
const HUD_WIDTH: f64 = 180.0;
const HUD_HEIGHT: f64 = 210.0;

/// HUD 窗口与屏幕边缘的默认间距
const HUD_MARGIN: f64 = 20.0;

/// 检查位置是否在任意活动显示器的可视区域内
fn is_position_valid(app: &AppHandle, x: f64, y: f64) -> bool {
    if let Ok(monitors) = app.available_monitors() {
        for monitor in monitors {
            let pos = monitor.position();
            let size = monitor.size();

            if is_rect_in_monitor_logical(
                pos.x,
                pos.y,
                size.width,
                size.height,
                monitor.scale_factor(),
                x,
                y,
            ) {
                return true;
            }
        }
    }
    false
}

/// 纯函数：检查给定的逻辑坐标 (x, y) 是否在显示器范围内
///
/// 考虑到 HUD 有一定尺寸，我们允许 HUD 只要有部分在屏幕内即可，
/// 或者严格限制完全在屏幕内？
/// 当前逻辑是宽松判定：只要 HUD 的左上角在 "屏幕逻辑区域扩大 HUD 宽高" 的范围内即可。
/// 即:
/// x >= logical_left - HUD_WIDTH
/// x <= logical_left + logical_width
/// y >= top - HUD_HEIGHT
/// y <= top + logical_height
fn is_rect_in_monitor_logical(
    monitor_x: i32,
    monitor_y: i32,
    monitor_width: u32,
    monitor_height: u32,
    scale_factor: f64,
    x: f64,
    y: f64,
) -> bool {
    // 转换为逻辑像素
    let logical_left = monitor_x as f64 / scale_factor;
    let logical_top = monitor_y as f64 / scale_factor;
    let logical_width = monitor_width as f64 / scale_factor;
    let logical_height = monitor_height as f64 / scale_factor;

    x >= logical_left - HUD_WIDTH
        && x <= logical_left + logical_width
        && y >= logical_top - HUD_HEIGHT
        && y <= logical_top + logical_height
}

/// 切换 Mini HUD 显示状态
///
/// 如果 HUD 窗口不存在则创建，存在则关闭
#[tauri::command]
pub async fn toggle_mini_hud(app: AppHandle) -> AppResult<bool> {
    info!("切换 Mini HUD 状态");

    // 检查是否已存在 HUD 窗口
    if let Some(existing_window) = app.get_webview_window(HUD_WINDOW_LABEL) {
        info!("HUD 窗口已存在，关闭");
        // 保存当前位置
        save_hud_position(&existing_window).await;
        let _ = existing_window.close();
        return Ok(false);
    }

    // 创建新的 HUD 窗口
    create_hud_window(&app).await?;
    Ok(true)
}

/// 关闭 HUD 窗口
#[tauri::command]
pub async fn close_hud(app: AppHandle) -> AppResult<()> {
    info!("关闭 Mini HUD");

    if let Some(window) = app.get_webview_window(HUD_WINDOW_LABEL) {
        // 保存当前位置
        save_hud_position(&window).await;
        let _ = window.close();
    }

    // 保存 HUD 禁用状态
    save_hud_enabled(false).await;

    // 更新托盘菜单项文字
    crate::update_hud_menu_item(&app, false);

    Ok(())
}

/// 设置 HUD Click-Through 模式 (仅 macOS)
///
/// 当 enabled 为 true 时，鼠标点击将穿透 HUD 窗口
#[tauri::command]
pub async fn set_hud_click_through(app: AppHandle, enabled: bool) -> AppResult<()> {
    debug!(enabled, "设置 HUD Click-Through 模式");

    if let Some(window) = app.get_webview_window(HUD_WINDOW_LABEL) {
        #[cfg(target_os = "macos")]
        {
            let _ = window.set_ignore_cursor_events(enabled);
            info!(enabled, "macOS Click-Through 模式已设置");
        }

        #[cfg(not(target_os = "macos"))]
        {
            if enabled {
                warn!("Click-Through 模式仅支持 macOS");
            }
        }
    }

    Ok(())
}

/// 获取 HUD 状态（是否显示）
#[tauri::command]
pub fn get_hud_status(app: AppHandle) -> bool {
    app.get_webview_window(HUD_WINDOW_LABEL).is_some()
}

/// 获取最新 HUD 事件 (Polling 降级方案)
#[tauri::command]
pub fn get_latest_hud_event() -> Option<HudEvent> {
    hud_service::get_hud_emitter().get_last_event()
}

/// 创建 HUD 窗口
async fn create_hud_window(app: &AppHandle) -> AppResult<()> {
    info!("创建 Mini HUD 窗口");

    // 加载设置以获取保存的位置
    let settings_service = AppSettingsService::new();
    let app_settings = settings_service.get_settings().await.unwrap_or_default();
    let hud_settings = app_settings.hud.unwrap_or_default();

    // 智能位置计算：优先使用保存的位置，如无效则使用默认位置
    let (x, y) = match (hud_settings.x, hud_settings.y) {
        (Some(saved_x), Some(saved_y)) => {
            // 检查保存的位置是否在当前任意显示器的可视范围内
            if is_position_valid(app, saved_x, saved_y) {
                info!("使用保存的 HUD 位置: ({}, {})", saved_x, saved_y);
                (saved_x, saved_y)
            } else {
                info!("保存的 HUD 位置无效，使用默认位置");
                calculate_default_position(app)
            }
        }
        _ => {
            info!("无保存的 HUD 位置，使用默认位置");
            calculate_default_position(app)
        }
    };

    // 创建窗口 Builder
    let window_builder = WebviewWindowBuilder::new(
        app,
        HUD_WINDOW_LABEL,
        WebviewUrl::App("index.html#/hud".into()),
    )
    .title("Mini HUD")
    .inner_size(HUD_WIDTH, HUD_HEIGHT)
    .position(x, y)
    .decorations(false)
    .transparent(true)
    .shadow(false) // [Fix]: Disable native shadow to prevent 1px black border in dark mode
    .always_on_top(hud_settings.always_on_top)
    .skip_taskbar(true)
    .resizable(false)
    .visible(true);

    match window_builder.build() {
        Ok(window) => {
            info!(label = HUD_WINDOW_LABEL, "HUD 窗口已创建 at ({}, {})", x, y);

            // macOS: Apply 'Frost Glass' vibrancy
            // [Fix]: Disable native vibrancy because it forces a rectangular backdrop,
            // creating white corners that don't match the rounded border-radius.
            // We use CSS backdrop-filter in the frontend instead.
            #[cfg(target_os = "macos")]
            {
                // use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
                // let _ = apply_vibrancy(&window, NSVisualEffectMaterial::Sidebar, None, None);

                // [Fix]: 默认关闭 Click-Through，窗口默认可拖动
                // 用户可以通过右键菜单 Lock 启用穿透模式
                let _ = window.set_ignore_cursor_events(false);
                info!("HUD 窗口默认设置为可交互模式（Click-Through 已关闭）");
            }

            // Windows: Apply blur
            // [Fix]: Disable native blur for consistency with macOS fix.
            #[cfg(target_os = "windows")]
            {
                // use window_vibrancy::apply_blur;
                // let _ = apply_blur(&window, Some((255, 255, 255, 125))); // Light frost
            }

            // 保存 HUD 启用状态
            save_hud_enabled(true).await;

            // 立即保存一次正确的位置，覆盖可能的错误配置
            save_hud_position(&window).await;

            let _ = window;
        }
        Err(e) => {
            error!(error = %e, "创建 HUD 窗口失败");
            return Err(crate::error::AppError::WindowError(e.to_string()));
        }
    }

    Ok(())
}

/// 计算默认位置（屏幕右上角）
fn calculate_default_position(app: &AppHandle) -> (f64, f64) {
    // 尝试获取主显示器尺寸
    if let Some(monitor) = app.primary_monitor().ok().flatten() {
        let size = monitor.size();
        let scale_factor = monitor.scale_factor();

        // 注意：WebviewWindowBuilder position 使用逻辑像素
        // monitor.size() 返回物理像素，需要除以缩放因子
        let logical_width = size.width as f64 / scale_factor;

        let x = logical_width - HUD_WIDTH - HUD_MARGIN;
        let y = HUD_MARGIN;
        info!(
            "显示器尺寸: {}x{} (Physical), 缩放: {}, 逻辑宽度: {}, 计算位置: ({}, {})",
            size.width, size.height, scale_factor, logical_width, x, y
        );
        return (x, y);
    }

    // 如果无法获取显示器信息，使用合理的默认值
    warn!("无法获取主显示器信息，使用默认位置");
    (100.0, 100.0) // 修改默认位置为更显眼的地方 (100, 100)
}

/// 保存 HUD 窗口位置
async fn save_hud_position(window: &tauri::WebviewWindow) {
    let position = match window.outer_position() {
        Ok(pos) => pos,
        Err(e) => {
            warn!(error = %e, "获取 HUD 窗口位置失败");
            return;
        }
    };

    let scale_factor = window.scale_factor().unwrap_or(1.0);
    let logical_x = position.x as f64 / scale_factor;
    let logical_y = position.y as f64 / scale_factor;

    debug!(
        "保存 HUD 位置: Physical({}, {}), Scale({}), Logical({}, {})",
        position.x, position.y, scale_factor, logical_x, logical_y
    );

    let settings_service = AppSettingsService::new();
    if let Ok(mut settings) = settings_service.get_settings().await {
        let hud = settings.hud.get_or_insert_with(HudSettings::default);
        hud.x = Some(logical_x);
        hud.y = Some(logical_y);

        if let Err(e) = settings_service.save_settings(settings).await {
            warn!(error = %e, "保存 HUD 位置失败");
        } else {
            debug!("HUD 位置已保存到配置");
        }
    }
}

/// 保存 HUD 启用状态
async fn save_hud_enabled(enabled: bool) {
    let settings_service = AppSettingsService::new();
    if let Ok(mut settings) = settings_service.get_settings().await {
        let hud = settings.hud.get_or_insert_with(HudSettings::default);
        hud.enabled = enabled;

        if let Err(e) = settings_service.save_settings(settings).await {
            warn!(error = %e, "保存 HUD 启用状态失败");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::events::HudStatus;

    #[test]
    fn test_constants() {
        assert_eq!(HUD_WINDOW_LABEL, "mini-hud");
        assert!(HUD_WIDTH > 0.0);
        assert_eq!(HUD_HEIGHT, 210.0); // Changed from 325.0 to 210.0 for 3-card layout
        assert!(HUD_MARGIN >= 0.0);
    }

    #[test]
    fn test_hud_height_for_three_cards() {
        // HUD_HEIGHT should be 210.0 for the simplified 3-card layout
        assert_eq!(HUD_HEIGHT, 210.0);
        // 3 cards * ~60px each + padding = ~200px total
        assert!(HUD_HEIGHT >= 180.0 && HUD_HEIGHT <= 250.0);
    }

    #[test]
    fn test_get_latest_hud_event_returns_last_emitted_event() {
        let emitter = hud_service::get_hud_emitter();
        emitter.emit_completed("claude", "claude-sonnet-4", 1024, 32.0);

        let latest = get_latest_hud_event().expect("event should be recorded");
        assert_eq!(latest.provider, "claude");
        assert_eq!(latest.model, "claude-sonnet-4");
        assert_eq!(latest.total_tokens, 1024);
        assert_eq!(latest.status, HudStatus::Completed);
    }

    #[test]
    fn test_is_rect_in_monitor_logical_standard_1x() {
        // Monitor: 0,0 1920x1080 @ 1.0x
        let mx = 0;
        let my = 0;
        let w = 1920;
        let h = 1080;
        let scale = 1.0;

        // Inside
        assert!(is_rect_in_monitor_logical(
            mx, my, w, h, scale, 100.0, 100.0
        ));
        assert!(is_rect_in_monitor_logical(mx, my, w, h, scale, 10.0, 10.0));

        // Edge (top-left) - should be valid even if negative up to -HUD_WIDTH
        // HUD_WIDTH = 180.0
        assert!(is_rect_in_monitor_logical(
            mx, my, w, h, scale, -100.0, -100.0
        ));
        assert!(is_rect_in_monitor_logical(mx, my, w, h, scale, -179.0, 0.0));
        assert!(!is_rect_in_monitor_logical(
            mx, my, w, h, scale, -200.0, 0.0
        ));

        // Edge (bottom-right)
        // Max X = 1920
        assert!(is_rect_in_monitor_logical(
            mx, my, w, h, scale, 1900.0, 1000.0
        ));
        // Allowed slightly out? No, 2000 is way past 1920. Logic says x <= logical_width.
        // x=2000 > 1920, so should be false.
        assert!(!is_rect_in_monitor_logical(
            mx, my, w, h, scale, 2000.0, 500.0
        ));
        // Right edge
        assert!(is_rect_in_monitor_logical(
            mx, my, w, h, scale, 1920.0, 500.0
        ));
        assert!(!is_rect_in_monitor_logical(
            mx, my, w, h, scale, 1921.0, 500.0
        ));
    }

    #[test]
    fn test_is_rect_in_monitor_logical_high_dpi_2x() {
        // Monitor: 0,0 3840x2160 @ 2.0x
        // Logical: 1920x1080
        let mx = 0;
        let my = 0;
        let w = 3840;
        let h = 2160;
        let scale = 2.0;

        // Center
        assert!(is_rect_in_monitor_logical(
            mx, my, w, h, scale, 960.0, 540.0
        ));

        // Right Edge: 1920
        assert!(is_rect_in_monitor_logical(
            mx, my, w, h, scale, 1920.0, 100.0
        ));
        assert!(!is_rect_in_monitor_logical(
            mx, my, w, h, scale, 1921.0, 100.0
        ));
    }

    #[test]
    fn test_is_rect_in_monitor_secondary_display() {
        // Monitor 2: placed at -1920, 0 (Left of primary) 1920x1080 @ 1.0x
        let mx = -1920;
        let my = 0;
        let w = 1920;
        let h = 1080;
        let scale = 1.0;

        // Logical range: [-1920, 0]
        assert!(is_rect_in_monitor_logical(
            mx, my, w, h, scale, -1000.0, 500.0
        ));
        assert!(is_rect_in_monitor_logical(
            mx, my, w, h, scale, -1920.0, 500.0
        ));

        // Slightly left of monitor 2 (allowed by HUD_WIDTH margin)
        // logical_left = -1920
        // x >= -1920 - 180 = -2100
        assert!(is_rect_in_monitor_logical(
            mx, my, w, h, scale, -2100.0, 500.0
        ));
        assert!(!is_rect_in_monitor_logical(
            mx, my, w, h, scale, -2101.0, 500.0
        ));
    }
}
