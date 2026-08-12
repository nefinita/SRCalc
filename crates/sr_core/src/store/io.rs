//! io — 磁盘 TOML 数据读写（角色/光锥/遗器套装/敌方）
//!
//! 目录约定：`data/characters/<id>.toml`、`data/light_cones/<id>.toml`、
//! `data/relic_sets/<id>.toml`、`data/enemies/<id>.toml`。

use sr_api::ConfigData;
use std::path::{Path, PathBuf};

use super::paths::data_dir;

pub fn disk_data_dir() -> Option<PathBuf> {
    data_dir()
}

fn read_dir_toml(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "toml")
            && let Ok(content) = std::fs::read_to_string(&path)
        {
            out.push(content);
        }
    }
    out
}

fn parse_all<T: serde::de::DeserializeOwned>(dir: &Path) -> Vec<T> {
    read_dir_toml(dir)
        .iter()
        .filter_map(|content| toml::from_str::<T>(content).ok())
        .collect()
}

/// 从磁盘加载全部数据；任一目录不存在时返回 None（调用方回退内置数据）
pub fn load_from_disk() -> Option<ConfigData> {
    let root = data_dir()?;
    let characters = parse_all(&root.join("characters"));
    let light_cones = parse_all(&root.join("light_cones"));
    let relic_sets = parse_all(&root.join("relic_sets"));
    let enemies = parse_all(&root.join("enemies"));
    Some(ConfigData {
        characters,
        light_cones,
        relic_sets,
        enemies,
    })
}

pub fn save_toml(kind: &str, id: &str, value: &impl serde::Serialize) -> Result<(), String> {
    let Some(root) = data_dir() else {
        return Err("未找到数据目录".to_string());
    };
    let dir = root.join(kind);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let content = toml::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(format!("{id}.toml")), content).map_err(|e| e.to_string())
}

pub fn delete_toml(kind: &str, id: &str) -> Result<(), String> {
    let Some(root) = data_dir() else {
        return Err("未找到数据目录".to_string());
    };
    let path = root.join(kind).join(format!("{id}.toml"));
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

pub fn save_dir_for(kind: &str) -> Option<PathBuf> {
    data_dir().map(|root| root.join(kind))
}

// ---------- 队伍持久化（用户可写目录） ----------

pub fn teams_dir() -> Option<PathBuf> {
    super::paths::user_data_dir().map(|d| d.join("teams"))
}

pub fn save_team(name: &str, team: &sr_api::Team) -> Result<(), String> {
    let Some(dir) = teams_dir() else {
        return Err("未找到用户数据目录".to_string());
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let content = toml::to_string_pretty(team).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(format!("{name}.toml")), content).map_err(|e| e.to_string())
}

pub fn load_team(name: &str) -> Result<Option<sr_api::Team>, String> {
    let Some(dir) = teams_dir() else {
        return Ok(None);
    };
    let path = dir.join(format!("{name}.toml"));
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    toml::from_str(&content).map(Some).map_err(|e| e.to_string())
}

pub fn list_teams() -> Vec<String> {
    let Some(dir) = teams_dir() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "toml"))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|f| f.strip_suffix(".toml").map(|s| s.to_string()))
        .collect();
    names.sort();
    names
}

pub fn delete_team(name: &str) -> Result<(), String> {
    let Some(dir) = teams_dir() else {
        return Err("未找到用户数据目录".to_string());
    };
    let path = dir.join(format!("{name}.toml"));
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}
