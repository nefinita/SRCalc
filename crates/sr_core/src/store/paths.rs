//! paths — 数据目录定位
//!
//! 查找顺序：`SR_DATA_DIR` env → exe 旁/上级 `data/` → macOS bundle `Resources/data`
//!         → CWD 及向上逐级查找 `data/`（覆盖 `app/`、`app/src-tauri/`、仓库根等 dev 布局）。
//! 仅接受含 `characters/` 子目录的 `data/`，避免误命中 node_modules 等。
//! 用户可写数据目录（队伍保存）：macOS 用 Application Support，否则用当前目录。

use std::path::{Path, PathBuf};

fn find_data_dir(start: &Path, max_up: usize) -> Option<PathBuf> {
    let mut cur = Some(start.to_path_buf());
    for _ in 0..=max_up {
        let Some(dir) = cur.as_deref() else { break };
        let candidate = dir.join("data");
        if candidate.join("characters").is_dir() {
            return Some(candidate);
        }
        cur = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

pub fn data_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SR_DATA_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        // exe 旁或向上 2 级（dev: target/debug → target → 仓库根）
        if let Some(dir) = exe.parent().and_then(|p| find_data_dir(p, 2)) {
            return Some(dir);
        }
        // macOS bundle: exe 位于 Contents/MacOS，数据在 Contents/Resources/data
        if let Some(resources) = exe
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.join("Resources"))
        {
            let candidate = resources.join("data");
            if candidate.join("characters").is_dir() {
                return Some(candidate);
            }
        }
    }
    // CWD 及向上逐级（覆盖 app/、app/src-tauri/ 等 dev 启动目录）
    if let Ok(cwd) = std::env::current_dir()
        && let Some(dir) = find_data_dir(&cwd, 6)
    {
        return Some(dir);
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
