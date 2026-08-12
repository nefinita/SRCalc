//! ffi — C ABI：句柄 + JSON 协议
//!
//! 方法名与 host/ 契约方法一一对应。所有入口用 catch_unwind 包裹，
//! 错误经 `sr_last_error` 取回。

use sr_api::{
    DamageRequest, Enemy, OptimizeRequest, RotationRequest, Character, LightCone, RelicSet,
};
use std::os::raw::c_char;
use std::sync::Mutex;

pub const ABI_VERSION: u32 = 1;

pub struct SrHandle;

static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn set_error(err: String) {
    if let Ok(mut guard) = LAST_ERROR.lock() {
        *guard = Some(err);
    }
}

fn take_error() -> Option<String> {
    LAST_ERROR.lock().ok().and_then(|mut g| g.take())
}

unsafe fn c_str_in(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let bytes = unsafe { std::ffi::CStr::from_ptr(ptr) }.to_bytes();
    Some(String::from_utf8_lossy(bytes).into_owned())
}

fn c_str_out(s: String) -> *mut c_char {
    match std::ffi::CString::new(s) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn from_json<T: serde::de::DeserializeOwned>(request: &str) -> Result<T, String> {
    serde_json::from_str(request).map_err(|e| e.to_string())
}

fn to_json<T: serde::Serialize>(v: &T) -> Result<String, String> {
    serde_json::to_string(v).map_err(|e| e.to_string())
}

fn dispatch(method: &str, request: &str) -> Result<String, String> {
    match method {
        "calculate_damage" => {
            let req: DamageRequest = from_json(request)?;
            let out = crate::host::calc::calculate_damage(req)?;
            to_json(&out)
        }
        "calculate_rotation" => {
            let req: RotationRequest = from_json(request)?;
            let out = crate::host::rotation::calculate_rotation(req)?;
            to_json(&out)
        }
        "run_optimize" => {
            let req: OptimizeRequest = from_json(request)?;
            let out = crate::host::optimize::run_optimize(req)?;
            to_json(&out)
        }
        "load_config" => {
            let out = crate::host::config::load_config();
            to_json(&out)
        }
        "save_character" => {
            let v: Character = from_json(request)?;
            crate::host::config::save_character(&v)?;
            Ok("null".to_string())
        }
        "save_light_cone" => {
            let v: LightCone = from_json(request)?;
            crate::host::config::save_light_cone(&v)?;
            Ok("null".to_string())
        }
        "save_relic_set" => {
            let v: RelicSet = from_json(request)?;
            crate::host::config::save_relic_set(&v)?;
            Ok("null".to_string())
        }
        "save_enemy" => {
            let v: Enemy = from_json(request)?;
            crate::host::config::save_enemy(&v)?;
            Ok("null".to_string())
        }
        "delete_character" => {
            let v: serde_json::Value = from_json(request)?;
            let id = v["id"].as_str().ok_or("缺少 id")?;
            crate::host::config::delete_character(id)?;
            Ok("null".to_string())
        }
        "delete_enemy" => {
            let v: serde_json::Value = from_json(request)?;
            let id = v["id"].as_str().ok_or("缺少 id")?;
            crate::host::config::delete_enemy(id)?;
            Ok("null".to_string())
        }
        "save_team" => {
            let v: serde_json::Value = from_json(request)?;
            let name = v["name"].as_str().ok_or("缺少 name")?;
            let team: sr_api::Team = serde_json::from_value(v["team"].clone()).map_err(|e| e.to_string())?;
            crate::host::config::save_team(name, &team)?;
            Ok("null".to_string())
        }
        "load_team" => {
            let v: serde_json::Value = from_json(request)?;
            let name = v["name"].as_str().ok_or("缺少 name")?;
            let team = crate::host::config::load_team(name)?;
            to_json(&team)
        }
        "list_teams" => {
            let out = crate::host::config::list_teams();
            to_json(&out)
        }
        "delete_team" => {
            let v: serde_json::Value = from_json(request)?;
            let name = v["name"].as_str().ok_or("缺少 name")?;
            crate::host::config::delete_team(name)?;
            Ok("null".to_string())
        }
        _ => Err(format!("未知方法: {method}")),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sr_abi_version() -> u32 {
    ABI_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn sr_core_version() -> *mut c_char {
    c_str_out(crate::CORE_VERSION.to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn sr_handle_create() -> *mut SrHandle {
    Box::into_raw(Box::new(SrHandle))
}

/// 释放句柄。
///
/// # Safety
///
/// `handle` 必须来自 `sr_handle_create` 且未被释放过。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sr_handle_free(handle: *mut SrHandle) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

/// 调用契约方法，返回 JSON 字符串（须用 `sr_free_string` 释放）或 NULL（错误经 `sr_last_error`）。
///
/// # Safety
///
/// `handle` 必须有效；`method`/`request` 必须是以 NUL 结尾的有效 UTF-8 字符串指针。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sr_call(
    _handle: *mut SrHandle,
    method: *const c_char,
    request: *const c_char,
) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        let method = unsafe { c_str_in(method) }.unwrap_or_default();
        let request = unsafe { c_str_in(request) }.unwrap_or_else(|| "{}".to_string());
        dispatch(&method, &request)
    });
    match result {
        Ok(Ok(json)) => c_str_out(json),
        Ok(Err(err)) => {
            set_error(err.clone());
            log::warn(&err);
            std::ptr::null_mut()
        }
        Err(_) => {
            set_error("panic at FFI boundary".to_string());
            std::ptr::null_mut()
        }
    }
}

/// 释放 `sr_call` / `sr_core_version` / `sr_last_error` 返回的字符串。
///
/// # Safety
///
/// `s` 必须来自上述函数且未被释放过。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sr_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { std::ffi::CString::from_raw(s) });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn sr_last_error() -> *mut c_char {
    match take_error() {
        Some(err) => c_str_out(err),
        None => c_str_out(String::new()),
    }
}

mod log {
    use colorized::{Color, Colors};

    pub fn warn(msg: &str) {
        eprintln!("{}", msg.color(Colors::RedFg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(method: &str, request: &str) -> Result<String, String> {
        dispatch(method, request)
    }

    #[test]
    fn ffi_load_config_json() {
        let json = call("load_config", "{}").expect("load_config");
        let parsed: sr_api::ConfigData = serde_json::from_str(&json).expect("parse");
        assert!(parsed.characters.iter().any(|c| c.id == "1101"));
    }

    #[test]
    fn ffi_calculate_json() {
        let cfg_json = call("load_config", "{}").expect("load_config");
        let cfg: sr_api::ConfigData = serde_json::from_str(&cfg_json).expect("parse");
        let enemy = serde_json::json!(cfg.enemies[0]);
        let req = serde_json::json!({
            "config": cfg,
            "team": { "members": [ { "char_id": "1101", "build": {} } ] },
            "focus": "1101",
            "enemy": enemy,
            "buff": {},
            "coefficient": { "def_const": 200.0, "broken_multiplier": 0.9, "break_multiplier": 1.0 }
        });
        let out = call("calculate_damage", &req.to_string()).expect("calculate_damage");
        let parsed: Vec<sr_api::SkillResult> = serde_json::from_str(&out).expect("parse");
        assert!(!parsed.is_empty());
    }

    #[test]
    fn ffi_unknown_method_error() {
        assert!(call("no_such_method", "{}").is_err());
    }
}
