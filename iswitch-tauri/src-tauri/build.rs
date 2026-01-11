//! [INPUT]:
//!   - Cargo.toml: source: ../Cargo.toml ([POS]: Rust 依赖配置)
//!   - Tauri 配置: source: ../tauri.conf.json ([POS]: 应用配置)
//!
//! [OUTPUT]:
//!   - Tauri 特定代码生成
//!   - 资源文件打包
//!
//! [POS]: 编译时构建脚本，由 cargo 在编译前自动执行
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽
//!   - 调用 tauri_build::build() 生成 Tauri 所需代码
//!   - 保持最小化，避免拖慢编译速度

fn main() {
    tauri_build::build()
}
