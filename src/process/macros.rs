#[macro_export]
macro_rules! spawn {
    // Regular process that is not capturing any variables, spawned from a closure.
    (@process |$mailbox:ident : Mailbox<$mailbox_ty:ty>| $body:expr) => {
        spawn::<lunatic::Process<$mailbox_ty>, _>((), |_, $mailbox| $body).unwrap()
    };
    // Regular process capturing variable `$argument`, spawned from a closure.
    (@process |$argument:ident, $mailbox:ident : Mailbox<$mailbox_ty:ty>| $body:expr) => {
        spawn::<lunatic::Process<$mailbox_ty>, _>($argument, |$argument, $mailbox| $body).unwrap()
    };
    // Regular process that is not capturing any variables, spawned from a function name.
    (@process $function:path) => {
        spawn::<lunatic::Process<_>, _>((), $function).unwrap()
    };
    // Regular process capturing variable `$argument`, spawned from a function name.
    (@process $argument:ident, $function:path) => {
        spawn::<lunatic::Process<_>, _>($argument, $function).unwrap()
    };

    // Protocol that is not capturing any variables, spawned from a closure.
    (@protocol |$mailbox:ident : Protocol<$mailbox_ty:ty>| $body:expr) => {
        spawn::<lunatic::Protocol<<$mailbox_tyas lunatic::HasDual>::Dual>, _>((), |_, $mailbox| $body).unwrap()
    };
    // Protocol capturing variable `$argument`, spawned from a closure.
    (@protocol |$argument:ident, $mailbox:ident : Protocol<$mailbox_ty:ty>| $body:expr) => {
        spawn::<lunatic::Protocol<$mailbox_ty>, _>($argument, |$argument, $mailbox| $body).unwrap()
    };
    // Protocol that is not capturing any variables, spawned from a function name.
    (@protocol $function:path) => {
        spawn::<lunatic::Protocol<_>, _>((), $function).unwrap()
    };
    // Protocol capturing variable `$argument`, spawned from a function name.
    (@protocol $argument:ident, $function:path) => {
        spawn::<lunatic::Protocol<_>, _>($argument, $function).unwrap()
    };

    // Task spawned from a closure.
    (@task |$argument:ident| $body:expr) => {
        spawn::<lunatic::Task<_>, _>($argument, |$argument| $body).unwrap()
    };
    // Task spawned from a function name.
    (@task $argument:ident, $function:path) => {
        spawn::<lunatic::Task<_>, _>($argument, $function).unwrap()
    };

    // Background task that is not capturing any variables, spawned from a closure.
    (@background || $body:expr) => {
        spawn::<lunatic::BackgroundTask, _>((), |_| $body).unwrap()
    };
    // Background task capturing variable `$argument`, spawned from a closure.
    (@background |$argument:ident| $body:expr) => {
        spawn::<lunatic::BackgroundTask, _>($argument, |$argument| $body).unwrap()
    };
    // Background task that is not capturing any variables, spawned from a function name.
    (@background $function:path) => {
        spawn::<lunatic::BackgroundTask, _>((), $function).unwrap()
    };
    // Background task capturing variable `$argument`, spawned from a function name.
    (@background $argument:ident, $function:path) => {
        spawn::<lunatic::BackgroundTask, _>($argument, $function).unwrap()
    };

    // Server capturing state `$state`, spawned from a closure.
    (@server |$state:ident, $message:ident : $message_ty:ty| $body:expr) => {
        spawn::<lunatic::Server<$mailbox_ty, _>, _>($state, |$state, $message| $body).unwrap()
    };
    // Server capturing state `$state`, spawned from a function name.
    (@server $state:ident, $function:path) => {
        spawn::<lunatic::Server<_, _>, _>($state, $function).unwrap()
    };

    // Generic server from `$state`.
    (@gen_server $state:path) => {
        spawn::<lunatic::GenericServer<_>, _>($state, |_state| {}).unwrap()
    };

    // Supervisor server from `$state`.
    (@gen_server $state:path) => {
        spawn::<lunatic::Supervisor<_>, _>($state, |_state| {}).unwrap()
    };
}

#[macro_export]
macro_rules! spawn_in {
    ($env:ident, |$argument:ident : Mailbox<$mailbox:ty>| $body:expr) => {
        $env.spawn::<lunatic::Process<$mailbox>, _>((), |_, $argument| $body)
            .unwrap()
    };
}
