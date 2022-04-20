#[macro_export]
macro_rules! spawn {
    // From closure

    // A background process (no mailbox & not capturing any variables).
    (|| $body:expr) => {
        lunatic::Process::spawn((), |_, _: lunatic::Mailbox<()>| $body)
    };
    // A background process (no mailbox) that can capture one or more variables.
    (|$($argument:ident),*| $body:expr) => {
        lunatic::Process::spawn(($($argument),*), |($($argument),*), _: lunatic::Mailbox<()>| $body)
    };
    // A process with a mailbox that is not capturing any variables.
    (|$mailbox:ident : Mailbox<$mailbox_ty:ty>| $body:expr) => {
        lunatic::Process::spawn((), |_, $mailbox: lunatic::Mailbox<$mailbox_ty>| $body)
    };
    // A process capturing variable `$argument`.
    (|$argument:ident, $mailbox:ident : Mailbox<$mailbox_ty:ty>| $body:expr) => {
        lunatic::Process::spawn(
            $argument,
            |$argument, $mailbox: lunatic::Mailbox<$mailbox_ty>| $body,
        )
    };

    // From functions

    // A background process (no mailbox & not capturing any variables).
    ($function:ident) => {
        lunatic::Process::spawn((), |_, _: lunatic::Mailbox<()>| $function() )
    };
    // A background process (no mailbox) that can capture one or more variables.
    ($function:ident($($argument:ident),*)) => {
        lunatic::Process::spawn(($($argument),*), |($($argument),*), _: lunatic::Mailbox<()>| $function($($argument),*))
    };
}

#[macro_export]
macro_rules! spawn_link {
    // From closure

    // A background process (no mailbox & not capturing any variables).
    (|| $body:expr) => {
        lunatic::Process::spawn_link((), |_, _: lunatic::Mailbox<()>| $body)
    };
    // A background process (no mailbox) that can capture one or more variables.
    (|$($argument:ident),*| $body:expr) => {
        lunatic::Process::spawn_link(($($argument),*), |($($argument),*), _: lunatic::Mailbox<()>| $body)
    };
    // A process with a mailbox that is not capturing any variables.
    (|$mailbox:ident : Mailbox<$mailbox_ty:ty>| $body:expr) => {
        lunatic::Process::spawn_link((), |_, $mailbox: lunatic::Mailbox<$mailbox_ty>| $body)
    };
    // A process with a mailbox capturing variable `$argument`.
    (|$argument:ident, $mailbox:ident : Mailbox<$mailbox_ty:ty>| $body:expr) => {
        lunatic::Process::spawn_link(
            $argument,
            |$argument, $mailbox: lunatic::Mailbox<$mailbox_ty>| $body,
        )
    };

    // A protocol that is not capturing any variables.
    (|$protocol:ident : Protocol<$proto_ty:ty>| $body:expr) => {
        lunatic::Process::spawn_link(
            (),
            |_, $protocol: lunatic::protocol::Protocol<$proto_ty>| $body,
        )
    };
    // A protocol capturing variable `$argument`.
    (|$argument:ident, $protocol:ident : Protocol<$proto_ty:ty>| $body:expr) => {
        lunatic::Process::spawn_link(
            $argument,
            |$argument, $protocol: lunatic::protocol::Protocol<$proto_ty>| $body,
        )
    };

    // From functions

    // A background process (no mailbox & not capturing any variables).
    ($function:ident) => {
        lunatic::Process::spawn_link((), |_, _: lunatic::Mailbox<()>| $function() )
    };
    // A background process (no mailbox) that can capture one or more variables.
    ($function:ident($($argument:ident),*)) => {
        lunatic::Process::spawn_link(($($argument),*), |($($argument),*), _: lunatic::Mailbox<()>| $function($($argument),*))
    };
}
