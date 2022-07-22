use std::cell::{Cell, RefCell};

use lunatic::{process_local, spawn_link};
use lunatic_test::test;

#[test]
fn two_processes_have_independent_locals() {
    process_local!(static FOO: RefCell<u32> = RefCell::new(1));

    FOO.with_borrow_mut(|f| {
        *f = 2;
    });

    // each process starts out with the initial value of 1
    let child = spawn_link!(@task || {
        FOO.with_borrow_mut(|f| {
            assert_eq!(*f, 1);
            *f = 3;
        });
    });

    // wait for the process to complete
    let _ = child.result();

    // we retain our original value of 2 despite the child process
    FOO.with_borrow(|f| {
        assert_eq!(*f, 2);
    });
}

#[test]
fn cell_set_get() {
    process_local! {
        static X: Cell<i32> = panic!("!");
    }
    // Calling X.get() here would result in a panic.
    X.set(123); // But X.set() is fine, as it skips the initializer above.
    assert_eq!(X.get(), 123);
}

#[test]
fn cell_replace() {
    process_local! {
        static X: Cell<i32> = Cell::new(1);
    }

    assert_eq!(X.replace(2), 1);
    assert_eq!(X.replace(3), 2);
}

#[test]
fn refcell_borrow() {
    process_local! {
        static X: RefCell<Vec<i32>> = RefCell::new(Vec::new());
    }

    X.with_borrow(|v| assert!(v.is_empty()));
}

#[test]
fn refcell_take() {
    process_local! {
        static X: RefCell<Vec<i32>> = RefCell::new(Vec::new());
    }

    X.with_borrow_mut(|v| v.push(1));
    let a = X.take();
    assert_eq!(a, vec![1]);
    X.with_borrow(|v| assert!(v.is_empty()));
}
