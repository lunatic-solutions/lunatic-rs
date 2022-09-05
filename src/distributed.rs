use crate::host::api::distributed::{
    copy_lookup_nodes_results, exec_lookup_nodes, get_nodes, module_id, nodes_count,
};
use crate::host::api::{self};
use crate::module::{params_to_vec, Param};
use crate::LunaticError;

pub fn node_id() -> u64 {
    unsafe { api::distributed::node_id() }
}

pub fn nodes() -> Vec<u64> {
    let cnt = unsafe { nodes_count() as usize };
    let mut nodes = vec![0; cnt];
    let copied_cnt = unsafe { get_nodes(nodes.as_mut_ptr(), cnt as u32) as usize };
    nodes.truncate(copied_cnt);
    nodes
}

pub fn lookup_nodes(query: &'static str) -> Vec<u64> {
    let mut query_id = 0;
    let mut nodes_len = 0;
    let result = unsafe {
        exec_lookup_nodes(
            query.as_ptr(),
            query.len() as u32,
            &mut query_id,
            &mut nodes_len,
        )
    };
    if result == 1 {
        return Vec::new();
    }
    let mut nodes = vec![0; nodes_len as usize];
    let copied_cnt =
        unsafe { copy_lookup_nodes_results(query_id, nodes.as_mut_ptr(), nodes_len as u32) };
    if copied_cnt < 0 {
        return Vec::new();
    }
    nodes.truncate(copied_cnt as usize);
    nodes
}

pub fn spawn(node_id: u64, config_id: i64, entry: fn(i32), arg: i32) -> Result<u64, LunaticError> {
    let entry = entry as usize as i32;
    let params = params_to_vec(&[Param::I32(entry), Param::I32(arg)]);
    let mut id = 0;
    let func = concat!("_lunatic_spawn_by_index_", env!("CARGO_PKG_VERSION"));
    let result = unsafe {
        api::distributed::spawn(
            node_id,
            config_id,
            module_id(),
            func.as_ptr(),
            func.len(),
            params.as_ptr(),
            params.len(),
            &mut id,
        )
    };
    if result == 0 {
        Ok(id)
    } else {
        Err(LunaticError::from(id))
    }
}
