use lunatic::{net, Mailbox, Process};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
};

fn main() {
    let key = std::fs::read("./examples/CA/localhost.key")
        .expect("Should have been able to read the file");
    let cert = std::fs::read("./examples/CA/localhost.crt").expect("Should have read cert file");
    let listener = net::TlsListener::bind("127.0.0.1:3000", cert, key).unwrap();
    println!("Listening on addr: {}", listener.local_addr().unwrap());
    while let Ok((tls_stream, _peer)) = listener.accept() {
        // let clone = tls_stream.clone();
        println!("GOING TO SPAWN");
        Process::spawn(tls_stream, handle);
    }
}

fn handle(mut tls_stream: net::TlsStream, _: Mailbox<()>) {
    println!("Start handler");
    let mut buf_reader = BufReader::new(tls_stream.clone());
    let mut buffer = [0u8; 100];
    let read = buf_reader.read(&mut buffer).expect("Should have read line");
    println!(
        "GOT REQUEST len {} | {:?}",
        read,
        String::from_utf8(buffer.to_vec())
    );
    tls_stream
        .write(
            [
                "HTTP/1.1 200 OK\n",
                "Date: Wed, 28 Sep 2022 09:45:07 GMT",
                "Content-Length: 12\n",
                "Content-Type: text/html\n",
                "\n\n",
                "<h1>Hello world!</h1>",
            ]
            .concat()
            .as_bytes(),
        )
        .unwrap();
    println!("WROTE TO STREAM");
}
