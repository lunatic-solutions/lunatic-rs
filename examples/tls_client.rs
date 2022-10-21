use lunatic::net;
use std::io::{Read, Write};

fn main() {
    // TODO: add cert for client identification
    let _cert = std::fs::read("./examples/CA/localhost.crt").expect("Should have read cert file");
    let mut stream = net::TlsStream::connect("www.rust-lang.org", 443).unwrap();
    let mut buf = [0; 2000];
    let req = ["GET / HTTP/1.1\r\n", "Host: www.rust-lang.org\r\n\r\n"].concat();

    // write message to stream
    stream
        .write(req.as_bytes())
        .expect("Should have written request");

    stream.read(&mut buf).expect("Should have read response");
    println!(
        "Got response from rust-lang.org {:?}",
        String::from_utf8(buf.to_vec())
    );
}
