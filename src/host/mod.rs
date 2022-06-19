//! Low level lunatic VM syscalls.

pub mod api;

use crate::{
    module::{params_to_vec, Param, WasmModule},
    LunaticError, ProcessConfig, Tag,
};

// Performs the low level dance that will turn a high level rust function into a lunatic process.
//
// Returns the process resource ID as an u64 in case of success.
//
// If `config` is None, the configuration used by the currently running process will be inherited.
// If link is `Some`, the newly crated process will be linked to the caller.
//
// The function `entry` will be used as entry point into the process. It will be called with the
// argument `arg`.
pub(crate) fn spawn(
    config: Option<&ProcessConfig>,
    link: Option<Tag>,
    entry: fn(i32),
    arg: i32,
) -> Result<u64, LunaticError> {
    let entry = entry as usize as i32;
    let params = params_to_vec(&[Param::I32(entry), Param::I32(arg)]);
    let mut id = 0;
    let func = "_lunatic_spawn_by_index";
    let link = match link {
        Some(tag) => tag.id(),
        None => 0,
    };
    let config_id = config.map_or_else(|| ProcessConfig::inherit().id(), |config| config.id());
    let result = unsafe {
        api::process::spawn(
            link,
            config_id,
            WasmModule::inherit().id(),
            func.as_ptr() as u32,
            func.len() as u32,
            params.as_ptr() as u32,
            params.len() as u32,
            &mut id as *mut u64 as u32,
        )
    };
    if result == 0 {
        Ok(id)
    } else {
        Err(LunaticError::from(id))
    }
}

#[export_name = "_lunatic_spawn_by_index"]
extern "C" fn _lunatic_spawn_by_index(function: i32, arg: i32) {
    let function: fn(i32) = unsafe { std::mem::transmute(function) };
    function(arg);
}
