use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }
}
