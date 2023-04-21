//! Low level lunatic VM syscalls.

pub mod api;

use serde::Deserialize;

use crate::module::{params_to_vec, Param, WasmModule};
use crate::{LunaticError, ProcessConfig, Tag};

/// Performs the low level dance that will turn a high level rust function into
/// a lunatic process.
///
/// Returns the process resource ID as an u64 in case of success.
///
/// If `config` is None, the configuration used by the currently running process
/// will be inherited. If link is `Some`, the newly crated process will be
/// linked to the caller.
///
/// The function `entry` will be used as entry point into the process. It will
/// be called with the argument `arg`.
pub(crate) fn spawn(
    name: Option<&str>,
    node: Option<u64>,
    config: Option<&ProcessConfig>,
    link: Option<Tag>,
    entry: fn(i32),
    arg: i32,
) -> Result<u64, LunaticError> {
    let entry = entry as usize as i32;
    let params = params_to_vec(&[Param::I32(entry), Param::I32(arg)]);
    let mut id = 0;
    let mut node_id = 0;
    let func = concat!("_lunatic_spawn_by_index_", env!("CARGO_PKG_VERSION"));
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
        } else if let Some(name) = name {
            api::process::get_or_spawn(
                name.as_ptr(),
                name.len(),
                link,
                config_id,
                WasmModule::inherit().id(),
                func.as_ptr(),
                func.len(),
                params.as_ptr(),
                params.len(),
                &mut node_id,
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
    } else if result == 2 {
        Err(LunaticError::NameAlreadyRegistered(node_id, id))
    } else {
        Err(LunaticError::Error(id))
    }
}

/// We attach the version to the exported function to avoid duplicate exports if
/// multiple dependencies use different versions of this crate. See:
/// https://github.com/lunatic-solutions/lunatic-rs/issues/71
#[export_name = concat!("_lunatic_spawn_by_index_", env!("CARGO_PKG_VERSION"))]
extern "C" fn _lunatic_spawn_by_index(function: i32, arg: i32) {
    let function: fn(i32) = unsafe { std::mem::transmute(function as usize) };
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
    };
}

pub fn send_receive_skip_search(node: u64, process_id: u64, wait_on_tag: i64, timeout: u64) -> u32 {
    if node_id() == node {
        unsafe { api::message::send_receive_skip_search(process_id, wait_on_tag, timeout) }
    } else {
        unsafe {
            api::distributed::send_receive_skip_search(node, process_id, wait_on_tag, timeout)
        }
    }
}

/// Utility for calling an allocating host function which is deserialized into
/// `T`.
///
/// # Example
///
/// ```no_run
/// struct Foo { a: String }
///
/// let foo = call_host_alloc::<Foo>(|len_ptr| unsafe {
///     lunatic::host::some_allocating_fn(len_ptr)
/// }).unwrap();
/// ```
#[doc(hidden)]
pub fn call_host_alloc<T>(f: impl Fn(*mut u32) -> u32) -> bincode::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut len = 0_u32;
    let len_ptr = &mut len as *mut u32;
    let ptr = f(len_ptr);
    let data_vec = unsafe { Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize) };
    bincode::deserialize(&data_vec)
}
