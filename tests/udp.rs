use std::io::ErrorKind;
use std::net::IpAddr;

use lunatic::net;
use lunatic_test::test;

#[test]
fn udp_ping_connect_recv_send_main() {
    let sender = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    sender.connect(receiver_addr).expect("couldn't connect");
    sender
        .send("P1NG".as_bytes())
        .expect("couldn't send message");

    let mut buf = [0; 4];
    let len_in = receiver.recv(&mut buf).unwrap();

    assert_eq!(len_in, 4);
    assert_eq!(buf, "P1NG".as_bytes());
}

#[test]
fn udp_ping_recv_from_send_to_main() {
    let sender = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let sender_addr = sender.local_addr().unwrap();
    let receiver = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    sender
        .send_to("P2NG".as_bytes(), receiver_addr)
        .expect("couldn't send message");

    let mut buf = [0; 4];
    let (len_in, addr_in) = receiver.recv_from(&mut buf).unwrap();

    assert_eq!(len_in, 4);
    assert_eq!(addr_in, sender_addr);
    assert_eq!(buf, "P2NG".as_bytes());
}

#[test]
fn udp_ping_sender_clone() {
    let sender = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let sender2 = sender.try_clone().unwrap();

    let receiver = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    sender2.connect(receiver_addr).expect("couldn't connect");
    sender2
        .send("P1NG".as_bytes())
        .expect("couldn't send message");

    let mut buf = [0; 4];
    let len_in = receiver.recv(&mut buf).unwrap();

    assert_eq!(len_in, 4);
    assert_eq!(buf, "P1NG".as_bytes());
}

#[test]
fn udp_ping_receiver_clone() {
    let sender = net::UdpSocket::bind("127.0.0.1:0").unwrap();

    let receiver = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver2 = receiver.try_clone().unwrap();
    let receiver_addr = receiver2.local_addr().unwrap();

    sender.connect(receiver_addr).expect("couldn't connect");
    sender
        .send("P1NG".as_bytes())
        .expect("couldn't send message");

    let mut buf = [0; 4];
    let len_in = receiver2.recv(&mut buf).unwrap();

    assert_eq!(len_in, 4);
    assert_eq!(buf, "P1NG".as_bytes());
}

#[test]
fn udp_peer_addr() {
    let sender = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    assert_eq!(
        sender.peer_addr().unwrap_err().kind(),
        ErrorKind::NotConnected
    );

    sender.connect(receiver_addr).expect("couldn't connect");

    assert_eq!(
        sender.peer_addr().unwrap().ip(),
        IpAddr::from([127, 0, 0, 1])
    );
}

#[test]
fn udp_ttl_setter_getter() {
    let sender = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    sender.set_ttl(42).unwrap();
    let cur_ttl = sender.ttl().unwrap();

    assert_eq!(cur_ttl, 42);
}

#[test]
fn udp_broadcast_setter_getter_true() {
    let sender = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    sender.set_broadcast(true).unwrap();
    let cur_broadcast = sender.broadcast().unwrap();

    assert_eq!(cur_broadcast, true);
}

#[test]
fn udp_broadcast_setter_getter_false() {
    let sender = net::UdpSocket::bind("127.0.0.1:0").unwrap();
    sender.set_broadcast(true).unwrap();
    sender.set_broadcast(false).unwrap();
    let cur_broadcast = sender.broadcast().unwrap();

    assert_eq!(cur_broadcast, false);
}
