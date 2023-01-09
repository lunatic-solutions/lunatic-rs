use lunatic::{host, sleep};
use lunatic_test::test;

#[test]
#[should_panic]
fn registry_lock_sanity_check() {
    let name = "name";
    unsafe {
        let mut node_id: u64 = 0;
        let mut process_id: u64 = 0;
        // Will lock
        host::api::registry::get_or_put_later(
            name.as_ptr(),
            name.len(),
            &mut node_id,
            &mut process_id,
        );
        // Will trap to prevent deadlock
        host::api::registry::get(name.as_ptr(), name.len(), &mut node_id, &mut process_id);
    }
}

#[test]
fn registry_delayed_insert() {
    let name = "name";
    unsafe {
        let mut node_id: u64 = 0;
        let mut process_id: u64 = 0;
        // Will lock
        host::api::registry::get_or_put_later(
            name.as_ptr(),
            name.len(),
            &mut node_id,
            &mut process_id,
        );
        // Will release lock
        host::api::registry::put(name.as_ptr(), name.len(), 27, 27);
        host::api::registry::get(name.as_ptr(), name.len(), &mut node_id, &mut process_id);
        assert_eq!(node_id, 27);
        assert_eq!(process_id, 27);
    }
}

#[test]
fn registry_failed_process_release_lock() {
    lunatic::spawn!(|| {
        unsafe {
            let name = "name";
            let mut node_id: u64 = 0;
            let mut process_id: u64 = 0;
            // Will lock
            host::api::registry::get_or_put_later(
                name.as_ptr(),
                name.len(),
                &mut node_id,
                &mut process_id,
            );
        }
        panic!("Should release lock");
    });

    // Wait for the process to panic and release the lock
    sleep(std::time::Duration::from_millis(100));
    unsafe {
        let name = "name";
        let mut node_id: u64 = 0;
        let mut process_id: u64 = 0;
        host::api::registry::get(name.as_ptr(), name.len(), &mut node_id, &mut process_id);
    }
}
