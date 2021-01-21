use std::env;

fn main() {
    println!("{}", std::mem::size_of::<std::net::SocketAddr>());
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
}
