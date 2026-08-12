//! sr_core — 星铁计算核心
//!
//! 分层：`engine`（纯计算）→ `store`（文件/内置数据）→ `host`（契约方法，供 Tauri/FFI 调用）。

pub mod engine;
pub mod ffi;
pub mod host;
pub mod store;

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
