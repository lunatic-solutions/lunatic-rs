use lunatic::{channel, Process};

#[test]
fn channel_integer_test() {
    let (sender, receiver) = channel::unbounded();
    sender.send(127).unwrap();
    assert_eq!(127, receiver.receive().unwrap());
}

#[test]
fn channel_vector_test() {
    let (sender, receiver) = channel::unbounded();
    sender.send(vec![1, 2, 3, 4, 5]).unwrap();
    assert_eq!(15, receiver.receive().unwrap().iter().sum());
}

#[test]
fn channel_across_process_test() {
    let (sender, receiver) = channel::unbounded();

    Process::spawn_with(sender.clone(), |sender| {
        sender.send(1337).unwrap();
    })
    .detach();

    Process::spawn_with(sender, |sender| {
        sender.send(1337).unwrap();
    })
    .detach();

    assert_eq!(1337, receiver.receive().unwrap());
    assert_eq!(1337, receiver.receive().unwrap());
}
