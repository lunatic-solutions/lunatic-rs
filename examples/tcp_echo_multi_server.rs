//! This is a simple TcpServer example using spawn_link! macro
//!
//! 1) Without Supervisor or AbstractProcess
//! 2) Using Serde passed structs
//! 3) To implement simple linebuffer echo back
//! 4) With ability to listen on multiple local addresses
//!
use lunatic::{net, Mailbox};
use lunatic::{sleep, spawn_link};
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

#[derive(serde::Serialize, serde::Deserialize)]
struct TcpServer {
    local_address: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TcpClient {
    tcp_stream: lunatic::net::TcpStream,
    peer: std::net::SocketAddr,
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let listen1 = TcpServer {
        local_address: "127.0.0.1:6666".to_owned(),
    };
    let listen2 = TcpServer {
        local_address: "127.0.0.1:6667".to_owned(),
    };

    spawn_link!(|input = listen1 | listen(input));
    spawn_link!(|input = listen2 | listen(input));

    loop {
        // Mainloop every 10s
        sleep(Duration::from_millis(10000));
    }
}

// Creates a TcpServer Listener
fn listen(input: TcpServer) {
    let listener = net::TcpListener::bind(input.local_address.clone()).unwrap();
    println!("Listening on addr: {}", listener.local_addr().unwrap());

    while let Ok((tcp_stream, peer)) = listener.accept() {
        println!("Accepted peer {} on addr: {}", peer, &input.local_address);

        let tcp_client = TcpClient { tcp_stream, peer };

        spawn_link!(|input = tcp_client | respond(input));
    }
}

// Respond back to Line buffered input
fn respond(mut client: TcpClient) {
    let mut buf_reader = BufReader::new(client.tcp_stream.clone());
    loop {
        let mut buffer = String::new();
        let read = buf_reader.read_line(&mut buffer).unwrap();
        if buffer.contains("exit") || read == 0 {
            return;
        }
        client.tcp_stream.write_all(buffer.as_bytes()).unwrap();
    }
}
