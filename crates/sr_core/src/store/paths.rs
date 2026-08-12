//! paths — 数据目录定位
//!
//! 查找顺序：`SR_DATA_DIR` env → exe 旁 `data/` → macOS bundle `Resources/data`
//!         → CWD `data/` → CWD `../data`（dev 从 `app/` 启动）。
//! 用户可写数据目录（队伍保存）：macOS 用 Application Support，否则用当前目录。

use std::path::PathBuf;

pub fn data_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SR_DATA_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent()?;
        let candidate = exe_dir.join("data");
        if candidate.is_dir() {
            return Some(candidate);
        }
        // macOS bundle: exe 位于 Contents/MacOS，数据在 Contents/Resources/data
        if let Some(resources) = exe_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("Resources"))
        {
            let candidate = resources.join("data");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }
    // 开发模式兜底：当前目录或上一级（npx tauri dev 从 app/ 启动 → 仓库根 data/）
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join("data");
        if candidate.is_dir() {
            return Some(candidate);
        }
        let parent = cwd.join("..").join("data");
        if parent.is_dir() {
            return Some(parent);
        }
    }
    None
}

/// 用户可写数据目录（队伍等运行时数据）
pub fn user_data_dir() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        return Some(
            PathBuf::from(home).join("Library/Application Support/com.qinthirteen.srcalc"),
        );
    }
    std::env::current_dir().ok()
}
