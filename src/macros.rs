/// Returns either `spawn`, `spawn_link` or `spawn_link_config`, depending on the arguments.
#[macro_export]
macro_rules! spawn_link_config {
    () => {
        lunatic::Process::spawn
    };
    (@link) => {
        lunatic::Process::spawn_link
    };
    ($config:ident) => {
        lunatic::Process::spawn_config
    };
    (@link $config:ident) => {
        lunatic::Process::spawn_link_config
    };
}

#[macro_export]
macro_rules! spawn {
    // A background process (no mailbox & not capturing any variables).
    ($(&$config:ident,)? || $body:expr) => {
        lunatic::spawn_link_config!($($config)?) ($(&$config,)? (), |_, _: lunatic::Mailbox<()>| $body)
    };
    // A background process (no mailbox) that can capture one or more variables.
    ($(&$config:ident,)? |$($argument:ident $(= $value:tt)? ),*| $body:expr) => {
        {
            // Re-assign variables if value is passed to the function
            $($(let $argument = $value)?;)*
            lunatic::spawn_link_config!($($config)?) (
                $(&$config,)?
                ($($argument),*),
                |($($argument),*), _: lunatic::Mailbox<()>| $body
            )
        }
    };
    ($(&$config:ident,)? |$($argument:ident $(= $value:block)? ),*| $body:expr) => {
        {
            // Re-assign variables if value is passed to the function
            $($(let $argument = $value)?;)*
            lunatic::spawn_link_config!($($config)?) (
                $(&$config,)?
                ($($argument),*),
                |($($argument),*), _: lunatic::Mailbox<()>| $body
            )
        }
    };
    // A process with a mailbox that is not capturing any variables.
    ($(&$config:ident,)? |$mailbox:ident : Mailbox<$mailbox_ty:ty>| $body:expr) => {
        lunatic::spawn_link_config!($($config)?) (
            $(&$config,)?
            (),
            |_, $mailbox: lunatic::Mailbox<$mailbox_ty>| $body
        )
    };
    // A process capturing variable `$argument`.
    ($(&$config:ident,)? |$argument:ident, $mailbox:ident : Mailbox<$mailbox_ty:ty>| $body:expr) => {
        lunatic::spawn_link_config!($($config)?) (
            $(&$config,)?
            $argument,
            |$argument, $mailbox: lunatic::Mailbox<$mailbox_ty>| $body,
        )
    };
}

#[macro_export]
macro_rules! spawn_link {
    // From closure

    // A background process (no mailbox & not capturing any variables).
    ($(&$config:ident,)? || $body:expr) => {
        lunatic::spawn_link_config!(@link $($config)?) (
            $(&$config,)?
            (),
            |_, _: lunatic::Mailbox<()>| $body
        )
    };
    // A background process (no mailbox) that can capture one or more variables.
    ($(&$config:ident,)? |$($argument:ident $(= $value:tt)? ),*| $body:expr) => {
        {
            // Re-assign variables if value is passed to the function
            $($(let $argument = $value)?;)*
            lunatic::spawn_link_config!(@link $($config)?) (
                $(&$config,)?
                ($($argument),*),
                |($($argument),*), _: lunatic::Mailbox<()>| $body
            )
        }
    };
    ($(&$config:ident,)? |$($argument:ident $(= $value:block)? ),*| $body:expr) => {
        {
            // Re-assign variables if value is passed to the function
            $($(let $argument = $value)?;)*
            lunatic::spawn_link_config!(@link $($config)?) (
                $(&$config,)?
                ($($argument),*),
                |($($argument),*), _: lunatic::Mailbox<()>| $body
            )
        }
    };
    // A process with a mailbox that is not capturing any variables.
    ($(&$config:ident,)? |$mailbox:ident : Mailbox<$mailbox_ty:ty>| $body:expr) => {
        lunatic::spawn_link_config!(@link $($config)?) (
            $(&$config,)?
            (),
            |_, $mailbox: lunatic::Mailbox<$mailbox_ty>| $body
        )
    };
    // A process with a mailbox capturing variable `$argument`.
    ($(&$config:ident,)? |$argument:ident, $mailbox:ident : Mailbox<$mailbox_ty:ty>| $body:expr) => {
        lunatic::spawn_link_config!(@link $($config)?) (
            $(&$config,)?
            $argument,
            |$argument, $mailbox: lunatic::Mailbox<$mailbox_ty>| $body,
        )
    };

     // A @task that is not capturing any variables.
     (@task $(&$config:ident,)? || $body:expr) => {
        lunatic::spawn_link_config!(@link $($config)?) (
            $(&$config,)?
            (),
            |_, protocol: lunatic::protocol::Protocol<lunatic::protocol::Send<_,lunatic::protocol::TaskEnd>>| {
                let _ = protocol.send($body);
            },
        )
    };
    // A @task capturing variables.
    (@task $(&$config:ident,)? |$($argument:ident $(= $value:block)? ),*| $body:expr) => {
        {
            // Re-assign variables if value is passed to the function
            $($(let $argument = $value)?;)*
            lunatic::spawn_link_config!(@link $($config)?) (
                $(&$config,)?
                ($($argument),*),
                |($($argument),*), protocol: lunatic::protocol::Protocol<
                        lunatic::protocol::Send<_,lunatic::protocol::TaskEnd>>| {
                    let _ = protocol.send($body);
                },
            )
        }
    };
    (@task $(&$config:ident,)? |$($argument:ident $(= $value:tt)? ),*| $body:expr) => {
        {
            // Re-assign variables if value is passed to the function
            $($(let $argument = $value)?;)*
            lunatic::spawn_link_config!(@link $($config)?) (
                $(&$config,)?
                ($($argument),*),
                |($($argument),*), protocol: lunatic::protocol::Protocol<
                        lunatic::protocol::Send<_,lunatic::protocol::TaskEnd>>| {
                    let _ = protocol.send($body);
                },
            )
        }
    };

    // A protocol that is not capturing any variables.
    ($(&$config:ident,)? |$protocol:ident : Protocol<$proto_ty:ty>| $body:expr) => {
        lunatic::spawn_link_config!(@link $($config)?) (
            $(&$config,)?
            (),
            |_, $protocol: lunatic::protocol::Protocol<$proto_ty>| $body,
        )
    };
    // A protocol capturing variable `$argument`.
    ($(&$config:ident,)? |$argument:ident, $protocol:ident : Protocol<$proto_ty:ty>| $body:expr) => {
        lunatic::spawn_link_config!(@link $($config)?) (
            $(&$config,)?
            $argument,
            |$argument, $protocol: lunatic::protocol::Protocol<$proto_ty>| $body,
        )
    };
}
