use std::time::Duration;

use crate::{host_api, LunaticError};

mod gen_server;
mod proc;
mod server;
mod task;

/// [`IntoProcess`] is a helper trait to generalize over the [`spawn`] function.
///
/// The `Handler` is usually a function that represents the entry point to the process or handles
/// individual messages. Some types (like ) already define a variety of handlers bound to the type
/// and set this associated type to `()`.
///
/// The generic parameter `C` allows spawned processes to transfer some state to the newly spawned
/// process. It's usually used together in combination with the `Handler` type to define a function
/// signature that receives the transferred state as an argument.
pub trait IntoProcess<C> {
    // The type of the 2nd argument passed to the [`spawn`] function.
    type Handler;
    // Spawn's a new process and returns a handle to it.
    fn spawn(module: Option<u64>, capture: C, handler: Self::Handler) -> Result<Self, LunaticError>
    where
        Self: Sized;
}

/// Spawns a new process.
///
/// TODO: Research if `spawn` and `spawn_link` could move the whole spawning procedure into the new
///       async task, so that there can't be any failure during the host call and we can return `T`
///       instead of a `Result` here.
pub fn spawn<T, C>(capture: C, handler: T::Handler) -> Result<T, LunaticError>
where
    T: IntoProcess<C>,
{
    <T as IntoProcess<C>>::spawn(None, capture, handler)
}

/// [`IntoLinkProcess`] is a helper trait to generalize over the [`spawn_link`] function.
///
/// The `Handler` is usually a function that represents the entry point to the process or handles
/// individual messages. Some types (like ) already define a variety of handlers bound to the type
/// and set this associated type to `()`.
///
/// The generic parameter `C` allows spawned processes to transfer some state to the newly spawned
/// process. It's usually used together in combination with the `Handler` type to define a function
/// signature that receives the transferred state as an argument.
pub trait IntoProcessLink<C> {
    // The type of the 2nd argument passed to the [`spawn`] function.
    type Handler;
    // Spawn's a new process and returns a handle to it.
    fn spawn_link(
        module: Option<u64>,
        capture: C,
        handler: Self::Handler,
    ) -> Result<Self, LunaticError>
    where
        Self: Sized;
}

/// Spawns a new process and link it to the parent.
///
/// TODO: Research if `spawn` and `spawn_link` could move the whole spawning procedure into the new
///       async task, so that there can't be any failure during the host call and we can return `T`
///       instead of a `Result` here.
pub fn spawn_link<T, C>(capture: C, handler: T::Handler) -> Result<T, LunaticError>
where
    T: IntoProcessLink<C>,
{
    <T as IntoProcessLink<C>>::spawn_link(None, capture, handler)
}

/// Suspends the current process for `milliseconds`.
pub fn sleep(duration: Duration) {
    unsafe { host_api::process::sleep_ms(duration.as_millis() as u64) };
}

// re-export [`GenericServer`], [`Process`], [`Server`], [`Task`]
pub use gen_server::GenericServer;
pub use proc::Process;
pub use server::Server;
pub use task::Task;
