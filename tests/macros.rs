use lunatic::protocol::End;
use lunatic::{spawn, spawn_link, test, ProcessConfig};

#[test]
fn spawn() {
    // Background process
    spawn!(|| {});
    // Mailbox process
    spawn!(|_mailbox: Mailbox<()>| {});
    // Capture local var
    let local_var = "Hello".to_owned();
    spawn!(|local_var| assert_eq!(local_var, "Hello"));
    // Give variable during invocation
    spawn!(|local_var = {"Hello".to_owned()}| assert_eq!(local_var, "Hello"));
    // Capture local var & mailbox process
    let local_var = "Hello".to_owned();
    spawn!(|local_var, _mailbox: Mailbox<()>| assert_eq!(local_var, "Hello"));
}

#[test]
fn spawn_config() {
    let config = ProcessConfig::new();
    // Background process
    spawn!(&config, || {});
    // Mailbox process
    spawn!(&config, |_mailbox: Mailbox<()>| {});
    // Capture local var
    let local_var = "Hello".to_owned();
    spawn!(&config, |local_var| assert_eq!(local_var, "Hello"));
    // Give variable during invocation
    spawn!(&config, | local_var = { "Hello".to_owned() } | assert_eq!(local_var, "Hello"));
    // Capture local var & mailbox process
    let local_var = "Hello".to_owned();
    spawn!(&config, |local_var, _mailbox: Mailbox<()>| assert_eq!(
        local_var,
        "Hello"
    ));
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
fn spawn_link_config() {
    let config = ProcessConfig::new();
    // Background process
    spawn_link!(&config, || {});
    // Mailbox process
    spawn_link!(&config, |_mailbox: Mailbox<()>| {});
    // Capture local var
    let local_var = "Hello".to_owned();
    spawn_link!(&config, |local_var| assert_eq!(local_var, "Hello"));
    // Capture local var & mailbox process
    let local_var = "Hello".to_owned();
    spawn_link!(&config, |local_var, _mailbox: Mailbox<()>| assert_eq!(
        local_var,
        "Hello"
    ));

    // Protocol, no capture
    spawn_link!(&config, |_proto: Protocol<End>| {});
    // Protocol, capture local_var
    let local_var = "Hello".to_owned();
    spawn_link!(&config, |local_var, _proto: Protocol<End>| assert_eq!(
        local_var,
        "Hello"
    ));
}

#[test]
fn multi_caputre() {
    let var1 = 2;
    let var2 = 3;
    spawn!(|var1, var2| assert_eq!(var1, var2));
    spawn_link!(|var1, var2| assert_eq!(var1, var2));
}

#[test]
fn task() {
    let task = spawn_link!(@task || 33);
    assert_eq!(task.result(), 33);

    let task = spawn_link!(@task |a = 2, b = 3| a + b);
    assert_eq!(task.result(), 5);

    let a = "hello".to_owned();
    let b = "world".to_owned();
    let task = spawn_link!(@task |a, b| format!("{} {}",a, b));
    assert_eq!(task.result(), "hello world");
}

#[test]
fn task_config() {
    let config = ProcessConfig::new();

    let task = spawn_link!(@task &config, || 33);
    assert_eq!(task.result(), 33);

    let task = spawn_link!(@task  &config, |a = 2, b = 3| a + b);
    assert_eq!(task.result(), 5);

    let a = "hello".to_owned();
    let b = "world".to_owned();
    let task = spawn_link!(@task &config, |a, b| format!("{} {}",a, b));
    assert_eq!(task.result(), "hello world");
}

#[test]
fn functions() {
    spawn!(|| func1());
    spawn!(|a = 3, b = 3|func2(a, b));
}

fn func1() {}
fn func2(a: i32, b: i32) {
    assert_eq!(a, b)
}
