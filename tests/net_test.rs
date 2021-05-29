use std::io::{self, BufRead, BufReader, Write};

use lunatic::{net, Process};

#[test]
fn tcp_test() {
    let server = Process::spawn_with((), |_| {
        let listener = net::TcpListener::bind("127.0.0.1:3337").unwrap();
        let tcp_stream = listener.accept().unwrap();
        let mut buf_reader = BufReader::new(tcp_stream);
        let mut buffer = String::new();
        buf_reader.read_line(&mut buffer).unwrap();
        let result = buffer.contains("test");
        assert!(result);
    });

    Process::sleep(1);

    let client = Process::spawn_with((), |_| {
        let mut tcp_stream = net::TcpStream::connect("127.0.0.1:3337").unwrap();
        tcp_stream.write("test".as_bytes()).unwrap();
        tcp_stream.flush().unwrap();
    });

    assert!(server.join().is_ok());
    assert!(client.join().is_ok());
}

#[test]
fn test_bind_to_unavailable_host() {
    let process = Process::spawn_with((), |_| {
        let listener = net::TcpListener::bind("1.1.1.1:5000");
        assert_eq!(
            io::ErrorKind::AddrNotAvailable,
            listener.unwrap_err().kind()
        );
    });

    assert!(process.join().is_ok());
}

#[test]
fn resolve_test() {
    let _google = net::resolve("google.com:80").unwrap();
}
