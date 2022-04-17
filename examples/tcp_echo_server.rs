use lunatic::{net, Mailbox, Process};
use std::io::{BufRead, BufReader, Write};

fn main() {
    let listener = net::TcpListener::bind("127.0.0.1:0").unwrap();
    println!("Listening on addr: {}", listener.local_addr().unwrap());
    while let Ok((tcp_stream, _peer)) = listener.accept() {
        // Pass the TCP stream as a context to the new process. We can't use a closures that
        // capture parent variables because no memory is shared between processes.
        Process::spawn(tcp_stream, handle);
    }
}

fn handle(mut tcp_stream: net::TcpStream, _: Mailbox<()>) {
    let mut buf_reader = BufReader::new(tcp_stream.clone());
    loop {
        let mut buffer = String::new();
        let read = buf_reader.read_line(&mut buffer).unwrap();
        if buffer.contains("exit") || read == 0 {
            return;
        }
        tcp_stream.write(buffer.as_bytes()).unwrap();
    }
}
