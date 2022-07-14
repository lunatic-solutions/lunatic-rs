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
    node: Option<u64>,
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
        if let Some(node) = node {
            api::distributed::spawn(
                node,
                config_id,
                api::distributed::module_id(),
                func.as_ptr(),
                func.len(),
                params.as_ptr(),
                params.len(),
                &mut id,
            )
        } else {
            api::process::spawn(
                link,
                config_id,
                WasmModule::inherit().id(),
                func.as_ptr(),
                func.len(),
                params.as_ptr(),
                params.len(),
                &mut id,
            )
        }
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

pub fn process_id() -> u64 {
    unsafe { api::process::process_id() }
}

pub fn node_id() -> u64 {
    unsafe { api::distributed::node_id() }
}

pub fn send(node: u64, process_id: u64) {
    if node_id() == node {
        unsafe { api::message::send(process_id) }
    } else {
        unsafe { api::distributed::send(node, process_id) }
    }
}

pub fn send_receive_skip_search(node: u64, process_id: u64, timeout: u32) -> u32 {
    if node_id() == node {
        unsafe { api::message::send_receive_skip_search(process_id, timeout) }
    } else {
        unsafe { api::distributed::send_receive_skip_search(node, process_id, timeout) }
    }
}
