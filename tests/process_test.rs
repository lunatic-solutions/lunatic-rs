use std::{num::Wrapping, ops::Add};

use lunatic::{
    process::{self, Process},
    Config, Environment, Mailbox,
};

#[lunatic::test]
fn spawn_link(m: Mailbox<()>) {
    let (_child, m) = process::spawn_link(m, |_: Mailbox<()>| panic!()).unwrap();
    // The child failure is captured as a message
    assert!(m.receive().is_signal());
}

#[lunatic::test]
fn memory_limit(m: Mailbox<u64>) {
    let mut config = Config::new(1_200_000, None); // ~1Mb and unlimited CPU instructions
    config.allow_namespace("lunatic::");
    config.allow_namespace("wasi_snapshot_preview1::");
    let mut env = Environment::new(config).unwrap();
    let module = env.add_this_module().unwrap();
    let (this, m) = process::this(m);
    // Allocating 100 bytes will work
    let (_, m) = module
        .spawn_link_with(m, (this.clone(), 100), allocate)
        .unwrap();
    assert_eq!(100, m.receive().normal_or_unwrap().unwrap());
    // Allocating ~1Mb (150k * 8 bytes) will fail as Rust reserves some extra space for the shadow stack.
    let (_, m) = module.spawn_link_with(m, (this, 150000), allocate).unwrap();
    assert!(m.receive().is_signal());
}

fn allocate((parent, input): (Process<u64>, usize), _: Mailbox<()>) {
    let allocate = vec![1; input];
    parent.send(allocate.iter().sum());
}

#[lunatic::test]
fn compute_limit(m: Mailbox<u64>) {
    let mut config = Config::new(2_000_000, Some(1)); // ~2Mb and ~ 100k CPU instructions
    config.allow_namespace("lunatic::");
    config.allow_namespace("wasi_snapshot_preview1::");
    let mut env = Environment::new(config).unwrap();
    let module = env.add_this_module().unwrap();
    let (this, m) = process::this(m);
    // Calculating fibonacci of 1 succeeds
    let (_, m) = module
        .spawn_link_with(m, (this.clone(), 12), fibonacci)
        .unwrap();
    assert_eq!(144, m.receive().normal_or_unwrap().unwrap());
    // Calculating fibonacci of 10_000 fails
    let (_, m) = module.spawn_link_with(m, (this, 10000), fibonacci).unwrap();
    assert!(m.receive().is_signal());
}

fn fibonacci((parent, input): (Process<u64>, u64), _: Mailbox<()>) {
    if input == 1 {
        parent.send(1);
        return;
    }

    let mut sum = Wrapping(0u64);
    let mut last = Wrapping(0u64);
    let mut curr = Wrapping(1u64);
    for _i in 1..input {
        sum = last.add(curr);
        last = curr;
        curr = sum;
    }
    parent.send(sum.0);
}
