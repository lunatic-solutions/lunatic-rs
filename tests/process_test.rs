use lunatic::Process;

#[test]
fn spawn_process_test() {
    Process::spawn_with((), |_| {
        let _ = 2 + 2;
    })
    .join()
    .unwrap();
}

#[test]
fn spawn_process_with_vec_test() {
    Process::spawn_with(vec![1, 2, 3, 4, 5, 6], |v: Vec<i32>| {
        v.iter().sum::<i32>();
    })
    .join()
    .unwrap();
}

#[test]
fn spawn_process_that_fails_test() {
    let result = Process::spawn_with((), |_: ()| panic!("Abort")).join();

    assert!(result.is_err());
}

#[test]
fn detach_process_test() {
    Process::spawn_with((), |_: ()| {
        let _ = 2 + 2;
    })
    .detach();
}

#[test]
fn panic_detach_process_test() {
    Process::spawn_with((), |_| Process::spawn_with((), |_| panic!("hi")).detach()).detach();
}
