use lunatic::{channel, Process};

fn main() {
    let (sender, receiver) = channel::unbounded();
    let vec: Vec<i32> = (0..3).collect();

    for i in vec.iter() {
        let process = Process::spawn_with((*i, vec.clone(), sender.clone()), child);
        process.detach();
    }

    for _ in vec.iter() {
        let (i, sum) = receiver.receive().unwrap();
        println!("Sum until {}: {}", i, sum);
    }
}

// Child process calculates the sum of numbers of context.1 until context.0 index.
fn child(context: (i32, Vec<i32>, channel::Sender<(i32, i32)>)) {
    let i = context.0;
    let vec = context.1;
    let channel = context.2;
    let sum_until_i: i32 = vec[..=i as usize].iter().sum();
    channel.send((i, sum_until_i)).unwrap();
}
