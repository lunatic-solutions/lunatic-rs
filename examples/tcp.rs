// Once WASI gets networking support you will be able to use Rust's `std::net` instead.
use lunatic::{net, Process};
use std::io::{BufRead, BufReader, Write};

fn main() {
    let listener = net::TcpListener::bind("127.0.0.1:1337").unwrap();
    while let Ok(tcp_stream) = listener.accept() {
        Process::spawn_with(tcp_stream, handle).detach();
    }
}

fn handle(mut tcp_stream: net::TcpStream) {
    let mut buf_reader = BufReader::new(tcp_stream.clone());
    loop {
        let mut buffer = String::new();
        buf_reader.read_line(&mut buffer).unwrap();
        if buffer.contains("exit") {
            return;
        }
        tcp_stream.write(buffer.as_bytes()).unwrap();
    }
}
