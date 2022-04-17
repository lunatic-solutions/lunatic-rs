use lunatic::Task;
use lunatic_test::test;

#[test]
fn simple_task() {
    let task = Task::spawn_link((2, 3), |(a, b)| a + b);
    assert_eq!(task.result(), 5);
}

#[test]
#[should_panic]
fn failing_child_kills_task() {
    let task = Task::spawn_link((), |_| panic!(""));
    task.result()
}

#[test]
fn result_should_be_called() {
    // This will pring an error to stderr. Display it with:
    // cargo test result_should_be_called --test task -- --show-output
    let _ = Task::spawn_link((), |_| {});
}

#[test]
fn recursive_count() {
    let task = Task::spawn_link(1000, recursive_count_sub);
    assert_eq!(500500, task.result());
}

fn recursive_count_sub(n: i32) -> i32 {
    if n > 0 {
        n + Task::spawn_link(n - 1, recursive_count_sub).result()
    } else {
        0
    }
}
