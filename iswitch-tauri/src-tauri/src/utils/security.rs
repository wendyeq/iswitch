//! [INPUT]:
//!   source: ../../../openspec/changes/migrate-to-tauri-stack/design.md ([POS]: 安全规范)
//!
//! [OUTPUT]:
//!   - mask_api_key(): API Key 脱敏函数
//!   - secure_write(): 安全文件写入 (600 权限)
//!
//! [POS]: 安全相关工具函数，处理敏感信息脱敏和安全文件操作
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽

use crate::error::{AppError, AppResult};
use std::path::Path;

/// API Key 脱敏函数
///
/// 将 API Key 转换为仅显示前 4 位的脱敏形式，用于日志输出
///
/// # 示例
/// ```
/// use iswitch_lib::utils::security::mask_api_key;
///
/// assert_eq!(mask_api_key("sk-1234567890abcdef"), "sk-1****");
/// assert_eq!(mask_api_key("abc"), "****");
/// ```
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 4 {
        return "****".to_string();
    }
    format!("{}****", &key[..4])
}

/// 安全文件写入
///
/// 将内容写入文件并设置 600 权限（仅所有者可读写）
/// 用于保存包含敏感信息的配置文件
///
/// # 参数
/// - `path`: 目标文件路径
/// - `content`: 文件内容
///
/// # 错误
/// 返回 `AppError::ConfigWrite` 如果写入失败
pub fn secure_write(path: &Path, content: &[u8]) -> AppResult<()> {
    // 写入文件
    std::fs::write(path, content).map_err(|e| AppError::ConfigWrite {
        path: path.display().to_string(),
        source: e,
    })?;

    // Unix 系统设置 600 权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = std::fs::metadata(path)
            .map_err(|e| AppError::ConfigWrite {
                path: path.display().to_string(),
                source: e,
            })?
            .permissions();

        perms.set_mode(0o600);

        std::fs::set_permissions(path, perms).map_err(|e| AppError::ConfigWrite {
            path: path.display().to_string(),
            source: e,
        })?;
    }

    Ok(())
}

/// 安全文本文件写入
///
/// `secure_write` 的便捷版本，接受字符串内容
pub fn secure_write_text(path: &Path, content: &str) -> AppResult<()> {
    secure_write(path, content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_mask_api_key_normal() {
        assert_eq!(mask_api_key("sk-1234567890abcdef"), "sk-1****");
        assert_eq!(mask_api_key("anthropic_key_12345"), "anth****");
    }

    #[test]
    fn test_mask_api_key_short() {
        assert_eq!(mask_api_key("abc"), "****");
        assert_eq!(mask_api_key("abcd"), "****");
        assert_eq!(mask_api_key(""), "****");
    }

    #[test]
    fn test_mask_api_key_edge() {
        assert_eq!(mask_api_key("abcde"), "abcd****");
    }

    #[test]
    fn test_secure_write() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("code_switch_test_secure_write.txt");

        // 写入
        secure_write(&test_file, b"test content").unwrap();

        // 验证内容
        let mut content = String::new();
        std::fs::File::open(&test_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "test content");

        // Unix: 验证权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::metadata(&test_file).unwrap().permissions();
            assert_eq!(perms.mode() & 0o777, 0o600);
        }

        // 清理
        std::fs::remove_file(&test_file).ok();
    }
}
