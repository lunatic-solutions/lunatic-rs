use std::time::Duration;

use lunatic::sleep;

fn main() {
    sleep(Duration::from_millis(1000));
    let nodes = lunatic::distributed::nodes();
    for node in nodes {
        println!("Guest: spawn on node {node}");
        lunatic::distributed::spawn(node, hello, node as i32).ok();
    }
    sleep(Duration::from_millis(1000));
}

fn hello(x: i32) {
    println!("Hi from {x}")
}
