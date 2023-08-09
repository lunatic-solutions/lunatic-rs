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
    use lunatic::protocol::End;
    use lunatic::protocol::Protocol;
    use lunatic::protocol::Recv;
    use lunatic::protocol::Send;
    use lunatic::serializer::MessagePack;

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
    use lunatic::protocol::Branch;
    use lunatic::protocol::End;
    use lunatic::protocol::Offer;
    use lunatic::protocol::Pop;
    use lunatic::protocol::Protocol;
    use lunatic::protocol::Rec;
    use lunatic::protocol::Recv;
    use lunatic::protocol::Send;
    type P = Offer<Recv<u64, Send<u64, Pop>>, End>;

    let protocol = Process::spawn_link((), |(), loop_protocol: Protocol<P>| loop {
        match loop_protocol.offer() {
            Branch::Left(protocol) => {
                let (protocol, v) = protocol.receive();
                let protocol = protocol.send(v * 2);
                loop_protocol = protocol.pop();
            }
            Branch::Right(_end) => break,
        };
    });

    // let mut loop_protocol = protocol.repeat();
    // for i in 0..5 {
    //     let protocol = loop_protocol.select_left();
    //     let protocol = protocol.send(i);
    //     let (end, value) = protocol.receive();
    //     assert_eq!(i * 2, value);
    //     loop_protocol = end.pop().repeat();
    // }

    // let _end = loop_protocol.select_right();
}
