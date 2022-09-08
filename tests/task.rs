use std::time::Duration;

use lunatic::{sleep, spawn_link, ProcessConfig};
use lunatic_test::test;

#[test]
fn simple_task() {
    let task = spawn_link!(@task |a = 2, b = {3}| a + b);
    assert_eq!(task.result(), 5);
}

#[allow(unreachable_code)]
#[test]
#[should_panic]
fn failing_child_kills_task() {
    let task = spawn_link!(@task || panic!(""));
    task.result()
}

#[test]
#[should_panic]
fn result_must_be_called() {
    let _ = spawn_link!(@task  || {});
}

#[test]
fn recursive_count() {
    let mut config = ProcessConfig::new().unwrap();
    config.set_can_spawn_processes(true);
    let task = spawn_link!(@task &config, |n = 1_000| recursive_count_sub(n));
    assert_eq!(500500, task.result());
}

fn recursive_count_sub(n: i32) -> i32 {
    if n > 0 {
        n + spawn_link!(@task |n = {n - 1}| recursive_count_sub(n)).result()
    } else {
        0
    }
}

#[test]
fn timeout_task() {
    let task = spawn_link!(@task || sleep(Duration::from_millis(25)));
    let result = task.result_timeout(Duration::from_millis(10));
    assert!(result.is_timed_out());
}
