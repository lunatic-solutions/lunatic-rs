use crate::host::api::distributed::{get_nodes, nodes_count};

pub fn nodes() -> Vec<u64> {
    let cnt = unsafe { nodes_count() as usize };
    let mut nodes = vec![0; cnt];
    let copied_cnt = unsafe { get_nodes(nodes.as_mut_ptr(), cnt as u32) as usize };
    nodes.truncate(copied_cnt);
    nodes
}
