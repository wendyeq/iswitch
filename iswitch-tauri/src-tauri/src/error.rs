//! [INPUT]:
//!   source: ../../openspec/changes/migrate-to-tauri-stack/design.md ([POS]: 错误处理规范)
//!
//! [OUTPUT]:
//!   - AppError: 统一应用错误类型
//!   - AppResult<T>: 结果类型别名
//!
//! [POS]: 统一错误处理模块，定义所有可能的错误类型并实现 Tauri 序列化
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use thiserror::Error;

/// 统一应用错误类型
///
/// 所有错误都应转换为此类型，便于统一处理和前端展示
#[derive(Error, Debug)]
pub enum AppError {
    // === IO 相关 ===
    /// 配置文件读取失败
    #[error("配置文件读取失败: {path}")]
    ConfigRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// 配置文件写入失败
    #[error("配置文件写入失败: {path}")]
    ConfigWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// 目录创建失败
    #[error("目录创建失败: {path}")]
    DirCreate {
        path: String,
        #[source]
        source: std::io::Error,
    },

    // === IO (General) ===
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    // === Provider 相关 ===
    /// 没有可用的 Provider
    #[error("没有可用的 Provider")]
    NoAvailableProvider,

    /// Provider 请求失败
    #[error("Provider 请求失败: {provider}")]
    ProviderRequest {
        provider: String,
        #[source]
        source: reqwest::Error,
    },

    /// 所有 Provider 均失败
    #[error("所有 Provider 均失败，尝试过: {attempts:?}")]
    AllProvidersFailed { attempts: Vec<String> },

    /// Provider 不支持请求的模型
    #[error("Provider '{provider}' 不支持模型 '{model}'")]
    ModelNotSupported { provider: String, model: String },

    // === 配置解析相关 ===
    /// JSON 解析失败
    #[error("JSON 解析失败: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// 配置解析失败 (通用)
    #[error("配置解析失败: {0}")]
    ConfigParse(serde_json::Error),

    /// TOML 解析失败
    #[error("TOML 解析失败: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// TOML 序列化失败
    #[error("TOML 序列化失败: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// 序列化失败
    #[error("序列化失败: {0}")]
    Serialize(String),

    // === 数据库相关 ===
    /// 数据库错误
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    // === 网络相关 ===
    /// HTTP 请求错误
    #[error("HTTP 请求错误: {0}")]
    HttpRequest(#[from] reqwest::Error),

    // === 代理相关 ===
    /// 代理服务器启动失败
    #[error("代理服务器启动失败: {0}")]
    ProxyStart(String),

    /// 代理服务器已在运行
    #[error("代理服务器已在运行")]
    ProxyAlreadyRunning,

    // === 通用错误 ===
    /// 无效参数
    #[error("无效参数: {0}")]
    InvalidArgument(String),

    /// 无效输入
    #[error("无效输入: {0}")]
    InvalidInput(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),

    /// 窗口操作错误
    #[error("窗口操作错误: {0}")]
    WindowError(String),
}

/// 应用结果类型别名
pub type AppResult<T> = Result<T, AppError>;

/// 实现 Serialize 供 Tauri Commands 返回
///
/// Tauri 要求 Command 返回的错误类型必须实现 Serialize
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 将错误转换为字符串形式
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::NoAvailableProvider;
        assert_eq!(err.to_string(), "没有可用的 Provider");

        let err = AppError::AllProvidersFailed {
            attempts: vec!["OpenAI".to_string(), "Anthropic".to_string()],
        };
        assert!(err.to_string().contains("OpenAI"));
        assert!(err.to_string().contains("Anthropic"));
    }

    #[test]
    fn test_error_serialize() {
        let err = AppError::NoAvailableProvider;
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("没有可用的 Provider"));
    }
}
