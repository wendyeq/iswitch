//! [INPUT]:
//!   - iswitch_lib: source: lib.rs ([POS]: Tauri 应用库入口)
//!
//! [OUTPUT]:
//!   - Tauri 进程初始化
//!   - 调用 iswitch_lib::run() 启动应用
//!
//! [POS]: Tauri 应用的实际入口点，负责调用库入口函数
//!
//! [PROTOCOL]: FractalFlow v1.0 - 分形自洽
//!   - 保持最小化，仅调用库函数
//!   - Windows release 配置（禁止额外控制台）

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    iswitch_lib::run()
}
