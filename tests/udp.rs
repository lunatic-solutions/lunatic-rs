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
