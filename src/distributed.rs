use crate::host::api::{
    self,
    distributed::{get_nodes, module_id, nodes_count},
};
use crate::{
    module::{params_to_vec, Param},
    LunaticError,
};

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

pub fn spawn(node_id: u64, entry: fn(i32), arg: i32) -> Result<u64, LunaticError> {
    let entry = entry as usize as i32;
    let params = params_to_vec(&[Param::I32(entry), Param::I32(arg)]);
    let mut id = 0;
    let func = "_lunatic_spawn_by_index";
    let result = unsafe {
        api::distributed::spawn(
            node_id,
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
