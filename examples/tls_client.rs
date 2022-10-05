use lunatic::{net, Mailbox, Process};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
};

fn main() {
    // let key = std::fs::read("./examples/CA/localhost.key")
    //     .expect("Should have been able to read the file");
    // let cert = std::fs::read("./examples/CA/localhost.crt").expect("Should have read cert file");
    println!("Starting stream");
    let mut stream = net::TlsStream::connect("www.rust-lang.org").unwrap();
    let mut buf = [0; 10000];
    let req = [
        "GET / HTTP/1.1\r\n",
        "Host: www.rust-lang.org\r\n\r\n",
        // "User-Agent: curl/7.79.1",
        // "Accept: */*",
    ]
    .concat();
    println!("Going to write to TLS stream");
    stream
        .write(req.as_bytes())
        .expect("Should have written request");
    stream.read(&mut buf).expect("Should have read response");
    println!("Listening on addr: {:?}", String::from_utf8(buf.to_vec()));
}
