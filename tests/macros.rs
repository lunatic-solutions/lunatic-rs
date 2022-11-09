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
    // Modify local var
    let local_var = "Hello".to_owned();
    spawn!(|local_var| {
        local_var.push_str(", world");
    });
}

#[test]
fn spawn_config() {
    let config = ProcessConfig::new().unwrap();
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
    // Modify local var
    let local_var = "Hello".to_owned();
    spawn!(&config, |local_var| {
        local_var.push_str(", world");
    });
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
    // Modify local var
    let local_var = "Hello".to_owned();
    spawn_link!(|local_var| {
        local_var.push_str(", world");
    });

    // Protocol, no capture
    spawn_link!(|_proto: Protocol<End>| {});
    // Protocol, capture local_var
    let local_var = "Hello".to_owned();
    spawn_link!(|local_var, _proto: Protocol<End>| assert_eq!(local_var, "Hello"));
}

#[test]
fn spawn_link_config() {
    let config = ProcessConfig::new().unwrap();
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
    // Modify local var
    let local_var = "Hello".to_owned();
    spawn_link!(&config, |local_var| {
        local_var.push_str(", world");
    });

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

    let a = "Hello".to_owned();
    let task = spawn_link!(@task |a| {
        a.push_str(", world");
        a
    });
    assert_eq!(task.result(), "Hello, world");

    let task = spawn_link!(@task || {
        let err = Err(());
        err?;
        Ok(())
    });
    assert_eq!(task.result(), Err(()));

    let task = spawn_link!(@task |a = 2, b = 3| {
        if a == 2 {
            return 0;
        }
        a + b
    });
    assert_eq!(task.result(), 0);
}

#[test]
fn task_config() {
    let config = ProcessConfig::new().unwrap();

    let task = spawn_link!(@task &config, || 33);
    assert_eq!(task.result(), 33);

    let task = spawn_link!(@task &config, |a = 2, b = 3| a + b);
    assert_eq!(task.result(), 5);

    let a = "hello".to_owned();
    let b = "world".to_owned();
    let task = spawn_link!(@task &config, |a, b| format!("{} {}",a, b));
    assert_eq!(task.result(), "hello world");

    let a = "Hello".to_owned();
    let task = spawn_link!(@task &config, |a| {
        a.push_str(", world");
        a
    });
    assert_eq!(task.result(), "Hello, world");
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
