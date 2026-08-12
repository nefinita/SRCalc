mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::load_config_cmd,
            commands::calculate_damage_cmd,
            commands::calculate_rotation_cmd,
            commands::run_optimize_cmd,
            commands::save_character_cmd,
            commands::delete_character_cmd,
            commands::save_enemy_cmd,
            commands::save_team_cmd,
            commands::load_team_cmd,
            commands::list_teams_cmd,
            commands::delete_team_cmd,
            commands::get_module_versions,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用程序时发生错误");
}
