/// Returns either `spawn`, `spawn_link` or `spawn_link_config`, depending on
/// the arguments.
#[doc(hidden)]
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

/// Helper macro for spawning processes.
///
/// The [`Process::spawn`](crate::Process::spawn) function can be too verbose
/// for simple processes. This macro should cover most common cases of spawning
/// a process from non-capturing closures.
///
/// # Example
///
/// ```
/// // Background process
/// spawn!(|| {});
/// // Mailbox process
/// spawn!(|_mailbox: Mailbox<()>| {});
/// // Capture local var
/// let local_var = "Hello".to_owned();
/// spawn!(|local_var| assert_eq!(local_var, "Hello"));
/// // Give variable during invocation
/// spawn!(|local_var = {"Hello".to_owned()}| assert_eq!(local_var, "Hello"));
/// // Background process with config
/// let config = ProcessConfig::new().unwrap();
/// spawn!(&config, || {});
/// ```
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
                |($(mut $argument),*), _: lunatic::Mailbox<()>| $body
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
                |($(mut $argument),*), _: lunatic::Mailbox<()>| $body
            )
        }
    };
    // A process with a mailbox that is not capturing any variables.
    ($(&$config:ident,)? |$mailbox:ident : Mailbox<$mailbox_ty:ty $( , $mailbox_s:ty )?>| $body:expr) => {
        lunatic::spawn_link_config!($($config)?) (
            $(&$config,)?
            (),
            |_, $mailbox: lunatic::Mailbox<$mailbox_ty $( , $mailbox_s )?>| $body
        )
    };
    // A process capturing variable `$argument`.
    ($(&$config:ident,)? |$argument:ident, $mailbox:ident : Mailbox<$mailbox_ty:ty $( , $mailbox_s:ty )?>| $body:expr) => {
        lunatic::spawn_link_config!($($config)?) (
            $(&$config,)?
            $argument,
            |mut $argument, $mailbox: lunatic::Mailbox<$mailbox_ty $( , $mailbox_s )?>| $body,
        )
    };
}

/// Helper macro for spawning linked processes.
///
/// The [`Process::spawn_link`](crate::Process::spawn_link) function can be too
/// verbose for simple processes. This macro should cover most common cases of
/// spawning a process from non-capturing closures.
///
/// # Example
///
/// ```
/// // Background process
/// spawn_link!(|| {});
/// // Mailbox process
/// spawn_link!(|_mailbox: Mailbox<()>| {});
/// // Capture local var
/// let local_var = "Hello".to_owned();
/// spawn_link!(|local_var| assert_eq!(local_var, "Hello"));
/// // Give variable during invocation
/// spawn_link!(|local_var = {"Hello".to_owned()}| assert_eq!(local_var, "Hello"));
/// // Protocol, no capture
/// spawn_link!(|_proto: Protocol<End>| {});
/// // Protocol, capture local_var
/// let local_var = "Hello".to_owned();
/// spawn_link!(|local_var, _proto: Protocol<End>| assert_eq!(local_var, "Hello"));
/// // Background process with config
/// let config = ProcessConfig::new().unwrap();
/// spawn_link!(&config, || {});
/// ```
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
                |($(mut $argument),*), _: lunatic::Mailbox<()>| $body
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
                |($(mut $argument),*), _: lunatic::Mailbox<()>| $body
            )
        }
    };
    // A process with a mailbox that is not capturing any variables.
    ($(&$config:ident,)? |$mailbox:ident : Mailbox<$mailbox_ty:ty $( , $mailbox_s:ty )?>| $body:expr) => {
        lunatic::spawn_link_config!(@link $($config)?) (
            $(&$config,)?
            (),
            |_, $mailbox: lunatic::Mailbox<$mailbox_ty $( , $mailbox_s )?>| $body
        )
    };
    // A process with a mailbox capturing variable `$argument`.
    ($(&$config:ident,)? |$argument:ident, $mailbox:ident : Mailbox<$mailbox_ty:ty $( , $mailbox_s:ty )?>| $body:expr) => {
        lunatic::spawn_link_config!(@link $($config)?) (
            $(&$config,)?
            $argument,
            |mut $argument, $mailbox: lunatic::Mailbox<$mailbox_ty $( , $mailbox_s )?>| $body,
        )
    };

     // A @task that is not capturing any variables.
     (@task $(&$config:ident,)? || $body:expr) => {
        lunatic::spawn_link_config!(@link $($config)?) (
            $(&$config,)?
            (),
            |_, protocol: lunatic::protocol::Protocol<lunatic::protocol::Send<_,lunatic::protocol::TaskEnd>>| {
                let _ = protocol.send((move || $body)());
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
                |($(mut $argument),*), protocol: lunatic::protocol::Protocol<
                        lunatic::protocol::Send<_,lunatic::protocol::TaskEnd>>| {
                    let _ = protocol.send((move || $body)());
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
                |($(mut $argument),*), protocol: lunatic::protocol::Protocol<
                        lunatic::protocol::Send<_,lunatic::protocol::TaskEnd>>| {
                    let _ = protocol.send((move || $body)());
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
            |mut $argument, $protocol: lunatic::protocol::Protocol<$proto_ty>| $body,
        )
    };
}
