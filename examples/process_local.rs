use lunatic::{process_local, spawn_link, Mailbox};
use std::cell::RefCell;

#[lunatic::main]
fn main(_: Mailbox<()>) {
    process_local!(static FOO: RefCell<u32> = RefCell::new(1));

    FOO.with(|f| {
        assert_eq!(*f.borrow(), 1);
        *f.borrow_mut() = 2;
    });

    // each process starts out with the initial value of 1
    let child = spawn_link!(@task || {
        FOO.with(|f| {
            assert_eq!(*f.borrow(), 1);
            *f.borrow_mut() = 3;
        });
    });

    // wait for the process to complete
    let _ = child.result();

    // we retain our original value of 2 despite the child process
    FOO.with(|f| {
        assert_eq!(*f.borrow(), 2);
    });
}
