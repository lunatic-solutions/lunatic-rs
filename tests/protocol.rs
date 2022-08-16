use lunatic::{
    protocol::{End, Protocol, Send},
    Process,
};
use lunatic_test::test;

#[test]
#[should_panic]
fn drop_unfinished() {
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
