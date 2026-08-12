//! config — 数据加载与保存契约方法

use sr_api::{Character, ConfigData, LightCone, RelicSet, Enemy};

use crate::store::{embedded, io};

/// 加载数据：磁盘 TOML 优先，缺失时回退内置占位数据
pub fn load_config() -> ConfigData {
    io::load_from_disk().unwrap_or_else(embedded::default_config)
}

pub fn save_character(character: &Character) -> Result<(), String> {
    io::save_toml("characters", &character.id, character)
}

pub fn delete_character(id: &str) -> Result<(), String> {
    io::delete_toml("characters", id)
}

pub fn save_light_cone(cone: &LightCone) -> Result<(), String> {
    io::save_toml("light_cones", &cone.id, cone)
}

pub fn save_relic_set(set: &RelicSet) -> Result<(), String> {
    io::save_toml("relic_sets", &set.id, set)
}

pub fn save_enemy(enemy: &Enemy) -> Result<(), String> {
    io::save_toml("enemies", &enemy.id, enemy)
}

pub fn delete_enemy(id: &str) -> Result<(), String> {
    io::delete_toml("enemies", id)
}

// ---------- 队伍 ----------

pub fn save_team(name: &str, team: &sr_api::Team) -> Result<(), String> {
    io::save_team(name, team)
}

pub fn load_team(name: &str) -> Result<Option<sr_api::Team>, String> {
    io::load_team(name)
}

pub fn list_teams() -> Vec<String> {
    io::list_teams()
}

pub fn delete_team(name: &str) -> Result<(), String> {
    io::delete_team(name)
}
