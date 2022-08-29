use lunatic::net::TcpStream;
use lunatic::serializer::{Bincode, Json, MessagePack};
use lunatic::{test, Mailbox, Process};

#[test]
fn bincode_resource_serialization() {
    let stream = TcpStream::connect("google.com:80").unwrap();
    Process::spawn(stream, |_, _: Mailbox<(), Bincode>| {});
}

#[test]
fn json_resource_serialization() {
    let stream = TcpStream::connect("google.com:80").unwrap();
    Process::spawn(stream, |_, _: Mailbox<(), Json>| {});
}

#[test]
fn msgpack_resource_serialization() {
    let stream = TcpStream::connect("google.com:80").unwrap();
    Process::spawn(stream, |_, _: Mailbox<(), MessagePack>| {});
}
