//! [INPUT]:
//!   source: ../../../../code-switch/services/appsettings.go ([POS]: 原 Go 设置数据模型)
//!
//! [OUTPUT]:
//!   - AppSettings 结构体
//!
//! [POS]: 应用设置数据模型定义
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use serde::{Deserialize, Serialize};

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 是否显示热力图
    #[serde(default = "default_true")]
    pub show_heatmap: bool,

    /// 是否开机自启动
    #[serde(default = "default_true")]
    pub auto_start: bool,

    /// 代理服务器端口 (默认 18099)
    ///
    /// # 已废弃
    /// 仅为兼容旧版配置保留，运行时会强制使用 DEFAULT_PROXY_PORT
    #[serde(default = "default_proxy_port")]
    pub proxy_port: u16,

    /// 故障转移阈值：连续失败多少次后降级供应商 (默认 5)
    ///
    /// # 已废弃
    /// 仅为兼容旧版配置保留，运行时会强制使用 DEFAULT_FAILOVER_THRESHOLD
    #[serde(default = "default_failover_threshold")]
    pub failover_threshold: u32,

    /// 恢复超时时间（秒）：降级后多久自动尝试恢复 (默认 300 = 5分钟)
    ///
    /// # 已废弃
    /// 仅为兼容旧版配置保留，运行时会强制使用 DEFAULT_RECOVERY_TIMEOUT_SECS
    #[serde(default = "default_recovery_timeout_secs")]
    pub recovery_timeout_secs: u64,

    /// Mini HUD 设置
    #[serde(default)]
    pub hud: Option<HudSettings>,
}

/// Mini HUD 设置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HudSettings {
    /// HUD 是否启用（下次启动时自动显示）
    #[serde(default)]
    pub enabled: bool,

    /// HUD 窗口 X 坐标
    #[serde(default)]
    pub x: Option<f64>,

    /// HUD 窗口 Y 坐标
    #[serde(default)]
    pub y: Option<f64>,

    /// HUD 窗口是否置顶 (默认 false)
    #[serde(default)]
    pub always_on_top: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            show_heatmap: true,
            auto_start: true,
            proxy_port: 18099,
            failover_threshold: 5,
            recovery_timeout_secs: 300,
            hud: None,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_proxy_port() -> u16 {
    18099
}

fn default_failover_threshold() -> u32 {
    5
}

fn default_recovery_timeout_secs() -> u64 {
    300
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert!(settings.show_heatmap);
        assert!(settings.auto_start);
        assert_eq!(settings.proxy_port, 18099);
        assert_eq!(settings.failover_threshold, 5);
        assert_eq!(settings.recovery_timeout_secs, 300);
        assert!(settings.hud.is_none());
    }

    #[test]
    fn test_app_settings_json() {
        let json = r#"{"show_heatmap": false, "auto_start": true, "proxy_port": 18101}"#;
        let settings: AppSettings = serde_json::from_str(json).unwrap();

        assert!(!settings.show_heatmap);
        assert!(settings.auto_start);
        assert_eq!(settings.proxy_port, 18101);
        // failover 配置使用默认值
        assert_eq!(settings.failover_threshold, 5);
        assert_eq!(settings.recovery_timeout_secs, 300);
    }

    #[test]
    fn test_app_settings_custom_failover() {
        let json = r#"{"failover_threshold": 5, "recovery_timeout_secs": 600}"#;
        let settings: AppSettings = serde_json::from_str(json).unwrap();

        assert_eq!(settings.failover_threshold, 5);
        assert_eq!(settings.recovery_timeout_secs, 600);
        // 其他字段使用默认值
        assert!(settings.show_heatmap);
        assert_eq!(settings.proxy_port, 18099);
    }

    #[test]
    fn test_app_settings_loads_custom_values_but_ignores_them_later() {
        let json = r#"{
            "proxy_port": 18100,
            "failover_threshold": 10,
            "recovery_timeout_secs": 600
        }"#;
        let settings: AppSettings = serde_json::from_str(json).unwrap();

        // 序列化阶段仍需完整保留用户配置，避免破坏旧文件
        assert_eq!(settings.proxy_port, 18100);
        assert_eq!(settings.failover_threshold, 10);
        assert_eq!(settings.recovery_timeout_secs, 600);

        // 运行时会改用硬编码常量（在 proxy::server 中验证）
    }

    #[test]
    fn test_hud_settings_default() {
        let hud = HudSettings::default();
        assert!(!hud.enabled);
        assert!(!hud.always_on_top);
    }

    #[test]
    fn test_hud_settings_json() {
        // Test with explicit values
        let json = r#"{"hud": {"enabled": true, "always_on_top": true}}"#;
        let settings: AppSettings = serde_json::from_str(json).unwrap();
        let hud = settings.hud.unwrap();
        assert!(hud.enabled);
        assert!(hud.always_on_top);

        // Test default values (missing fields rely on #[serde(default)])
        let json_default = r#"{"hud": {"enabled": true}}"#;
        let settings_default: AppSettings = serde_json::from_str(json_default).unwrap();
        let hud_default = settings_default.hud.unwrap();
        assert!(hud_default.enabled);
        assert!(!hud_default.always_on_top); // Default should be false
    }
}
