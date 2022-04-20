use lunatic::{protocol::End, spawn, spawn_link, test};

#[test]
fn spawn() {
    // Background process
    spawn!(|| {});
    // Mailbox process
    spawn!(|_mailbox: Mailbox<()>| {});
    // Capture local var
    let local_var = "Hello".to_owned();
    spawn!(|local_var| assert_eq!(local_var, "Hello"));
    // Capture local var & mailbox process
    let local_var = "Hello".to_owned();
    spawn!(|local_var, _mailbox: Mailbox<()>| assert_eq!(local_var, "Hello"));
}

#[test]
fn spawn_link() {
    // Background process
    spawn_link!(|| {});
    // Mailbox process
    spawn_link!(|_mailbox: Mailbox<()>| {});
    // Capture local var
    let local_var = "Hello".to_owned();
    spawn_link!(|local_var| assert_eq!(local_var, "Hello"));
    // Capture local var & mailbox process
    let local_var = "Hello".to_owned();
    spawn_link!(|local_var, _mailbox: Mailbox<()>| assert_eq!(local_var, "Hello"));

    // Protocol, no capture
    spawn_link!(|_proto: Protocol<End>| {});
    // Protocol, capture local_var
    let local_var = "Hello".to_owned();
    spawn_link!(|local_var, _proto: Protocol<End>| assert_eq!(local_var, "Hello"));
}

#[test]
fn multi_caputre() {
    let var1 = 2;
    let var2 = 3;
    spawn!(|var1, var2| assert_eq!(var1, var2));
    spawn_link!(|var1, var2| assert_eq!(var1, var2));
}

#[test]
fn functions() {
    spawn!(func1);
    let a = 3;
    let b = 3;
    spawn!(func2(a, b));
}

fn func1() {}
fn func2(a: i32, b: i32) {
    assert_eq!(a, b)
}
