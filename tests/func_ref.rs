use std::time::Duration;

use lunatic::function::FuncRef;
use lunatic_test::test;

#[test]
fn send_func_ref() {
    type Message = (i32, FuncRef<fn(i32) -> i32>);
    let p = lunatic::spawn_link!(|mailbox: Mailbox<Message>| {
        while let Ok((expected_value, fn_ref)) = mailbox.receive_timeout(Duration::from_millis(1)) {
            let result = fn_ref(1);
            assert_eq!(expected_value, result);
        }
    });

    fn identity(x: i32) -> i32 {
        x
    }
    fn double(x: i32) -> i32 {
        x * 2
    }
    p.send((1, FuncRef::new(identity)));
    p.send((2, FuncRef::new(double)));
    p.send((3, FuncRef::new(|x: i32| x * 3)));

    // allow time for the process to be killed if assert failed
    lunatic::sleep(Duration::from_millis(10));
}
