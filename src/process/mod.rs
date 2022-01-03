use crate::{host_api, LunaticError};

mod process;
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
    fn spawn(capture: C, handler: Self::Handler) -> Result<Self, LunaticError>
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
    <T as IntoProcess<C>>::spawn(capture, handler)
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
    fn spawn_link(capture: C, handler: Self::Handler) -> Result<Self, LunaticError>
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
    <T as IntoProcessLink<C>>::spawn_link(capture, handler)
}

// re-export [`Process`], [`Server`], [`Task`]
pub use process::Process;
pub use server::Server;
pub use task::Task;

/// Suspends the current process for `milliseconds`.
pub fn sleep(milliseconds: u64) {
    unsafe { host_api::process::sleep_ms(milliseconds) };
}
