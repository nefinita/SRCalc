//! commands — Tauri 命令（薄 IPC 适配层，直接调用 sr_core::host）

use sr_api::{
    Character, ConfigData, DamageRequest, Enemy, OptimizeRequest, OptimizeResult, RotationRequest,
    RotationResult, SkillResult, Team,
};

#[tauri::command]
pub fn load_config_cmd() -> ConfigData {
    sr_core::host::config::load_config()
}

#[tauri::command]
pub fn calculate_damage_cmd(req: DamageRequest) -> Result<Vec<SkillResult>, String> {
    sr_core::host::calc::calculate_damage(req)
}

#[tauri::command]
pub fn calculate_rotation_cmd(req: RotationRequest) -> Result<RotationResult, String> {
    sr_core::host::rotation::calculate_rotation(req)
}

#[tauri::command]
pub fn run_optimize_cmd(req: OptimizeRequest) -> Result<OptimizeResult, String> {
    sr_core::host::optimize::run_optimize(req)
}

#[tauri::command]
pub fn save_character_cmd(character: Character) -> Result<(), String> {
    sr_core::host::config::save_character(&character)
}

#[tauri::command]
pub fn delete_character_cmd(id: String) -> Result<(), String> {
    sr_core::host::config::delete_character(&id)
}

#[tauri::command]
pub fn save_enemy_cmd(enemy: Enemy) -> Result<(), String> {
    sr_core::host::config::save_enemy(&enemy)
}

#[tauri::command]
pub fn save_team_cmd(name: String, team: Team) -> Result<(), String> {
    sr_core::host::config::save_team(&name, &team)
}

#[tauri::command]
pub fn load_team_cmd(name: String) -> Result<Option<Team>, String> {
    sr_core::host::config::load_team(&name)
}

#[tauri::command]
pub fn list_teams_cmd() -> Vec<String> {
    sr_core::host::config::list_teams()
}

#[tauri::command]
pub fn delete_team_cmd(name: String) -> Result<(), String> {
    sr_core::host::config::delete_team(&name)
}

#[derive(serde::Serialize)]
pub struct ModuleVersions {
    pub core: String,
    pub const_: String,
}

#[tauri::command]
pub fn get_module_versions() -> ModuleVersions {
    ModuleVersions {
        core: sr_core::CORE_VERSION.to_string(),
        const_: sr_const::CORE_VERSION.to_string(),
    }
}
