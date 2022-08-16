#[cfg(feature = "msgpack_serializer")]
mod msgpack {
    use lunatic::{serializer::MessagePack, test, Mailbox, Process, Tag};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct ProcBincode(Process<ProcMsgPack>);
    #[derive(Serialize, Deserialize)]
    struct ProcMsgPack(Process<i32, MessagePack>);

    #[test]
    fn msg_pack_serializer(mailbox: Mailbox<ProcMsgPack>) {
        let parent = mailbox.this();
        Process::spawn_link(
            ProcBincode(parent),
            |parent, _: Mailbox<(), MessagePack>| {
                // Propagate parent to sub-child
                Process::spawn_link(parent, |grandparent, mailbox: Mailbox<i32, MessagePack>| {
                    grandparent.0.send(ProcMsgPack(mailbox.this()));
                    let a = mailbox.receive();
                    let b = mailbox.receive();
                    assert_eq!(a + b, 5);
                    // Notify grandparent that we succeeded
                    grandparent.0.send(ProcMsgPack(mailbox.this()));
                });
            },
        );

        let grandchild = mailbox.receive();
        // Link grandchild
        grandchild.0.link();
        grandchild.0.send(8);
        grandchild.0.send(-3);
        // Wait on grandchild
        let _ = mailbox.receive();
    }

    #[test]
    fn message_equality(mailbox: Mailbox<Vec<i32>>) {
        let parent = mailbox.this();

        let child = Process::spawn_link(
            parent,
            |parent, child_mailbox: Mailbox<Vec<i32>, MessagePack>| {
                let input = child_mailbox.receive();
                parent.send(input);
            },
        );

        let input = vec![127; 500];

        child.send(input.clone());
        let output = mailbox.receive();

        assert_eq!(input, output);
    }

    #[test]
    fn tagged_message(mailbox: Mailbox<()>) {
        let parent = mailbox.this();

        let child = Process::spawn_link(
            parent,
            |parent, child_mailbox: Mailbox<u64, MessagePack>| {
                assert_eq!(
                    1,
                    child_mailbox.tag_receive(Some(&[Tag::special(64).unwrap()]))
                );
                assert_eq!(
                    2,
                    child_mailbox.tag_receive(Some(&[Tag::special(65).unwrap()]))
                );
                assert_eq!(
                    3,
                    child_mailbox.tag_receive(Some(&[Tag::special(66).unwrap()]))
                );
                // Indicate end of sub-process
                parent.send(());
            },
        );

        child.tag_send(Tag::special(66).unwrap(), 3);
        child.tag_send(Tag::special(65).unwrap(), 2);
        child.tag_send(Tag::special(64).unwrap(), 1);
        let _ = mailbox.receive();
    }
}
