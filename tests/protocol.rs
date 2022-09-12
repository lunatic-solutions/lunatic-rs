use lunatic::Process;
use lunatic_test::test;

#[test]
#[should_panic]
fn drop_unfinished() {
    use lunatic::protocol::End;
    use lunatic::protocol::Protocol;
    use lunatic::protocol::Send;
    let protocol = Process::spawn_link((), |_, _: Protocol<Send<(), End>>| {
        // Protocol dropped without sending a message back.
    });
    let _ = protocol.receive();
}

#[cfg(feature = "msgpack_serializer")]
#[test]
fn msg_pack_serializer() {
    use lunatic::protocol::Recv;
    use lunatic::serializer::MessagePack;
    use lunatic::protocol::End;
    use lunatic::protocol::Protocol;
    use lunatic::protocol::Send;

    let protocol = Process::spawn_link(
        (),
        |_, proto: Protocol<Recv<Vec<f64>, Send<f64, End>>, MessagePack>| {
            let (proto, input) = proto.receive();
            let _ = proto.send(input.iter().sum());
        },
    );

    let input = vec![0.33, 0.44, 0.11];
    let protocol = protocol.send(input);
    let (_, result) = protocol.receive();
    assert_eq!(0.88, result);
}

#[test]
fn recursive_protocols() {
    use lunatic::protocol::Rec;
    use lunatic::protocol::Recv;
    use lunatic::protocol::Send;
    use lunatic::protocol::End;
    use lunatic::protocol::Protocol;
    type P = Recv<u64, Send<u64, End>>;

    let protocol = Process::spawn_link(
        (),
        |(), proto: Protocol<Rec<P>>| {

            loop {
                let loop_protocol = proto.repeat();
                let (protocol, v) = loop_protocol.receive();
                let _end = protocol.send(v * 2);
            }
        }
    );

    for i in 0..5 {
        let loop_protocol = protocol.repeat();
        let p = loop_protocol.send(i);
        let (_end, value) = p.receive();
        assert_eq!(i * 2, value);
    }

    let _end = protocol.end();
}