/*!
Helper library for building Rust applications that run on [lunatic][1].

# Main concepts

The main abstraction in [lunatic][1] is a process. Contrary to operating system processes,
lunatic processes are lightweight and fast to spawn. They are designed for **massive**
concurrency. Like operating system processes, they are sandboxed. Each of them gets a separate
memory and they can't access the memory from other processes, not even through raw pointers.
If we want to exchange any information between process we need to do it through message passing.

This library makes processes feel native to the Rust language. They can be spawned from just a
function.

### Process types:

* **[`Process`]** - A process that can receive messages through a [`Mailbox`].
* **[`Task`]** - One-off process that returns a value.
* **[`Server`]** - Abstracts the common client-server interaction and can handle requests.
* **[`Supervisor`]** - A process that can supervise servers and re-spawn them if they panic.

### Linking

Processes can be linked together. This means that if one of them fails, all linked ones will be
killed too. The only exception is the Supervisor. The supervisor can define actions when one of the
children dies.

To spawn a linked process use the [`spawn_link`] function.

### Process configuration

TODO

# Setup

To run Rust applications on lunatic, you will first need to download the lunatic runtime by
following the installation steps in [this repository][1]. The runtime is just single executable
and runs on Windows, macOS and Linux.

Lunatic applications need to be compiled to WebAssembly before they can be executed by the
runtime. Rust has great support for WebAssembly and you can build a lunatic compatible
applications just by passing the `--target=wasm32-wasi` flag to cargo, e.g:

```
cargo build --release --target=wasm32-wasi
```

This will generate a .wasm file in the `target/wasm32-wasi/release/` folder inside your project.
You can now run your application by passing the generated .wasm file to lunatic, e.g:

```
lunatic target/wasm32-wasi/release/<name>.wasm
```

# Better developer experience

To simplify developing, testing and running lunatic applications with cargo, you can add a
`.cargo/config.toml` file to your project with the following content:

```toml
[build]
target = "wasm32-wasi"

[target.wasm32-wasi]
runner = "lunatic"
```

Now you can just use the commands you were already familiar with, such as `cargo run`, `cargo test`
and cargo is going to automatically build your project as a WebAssembly module and run it inside of
`lunatic`.

[1]: https://github.com/lunatic-solutions/lunatic
*/

mod config;
mod error;
pub mod host;
mod macros;
mod mailbox;
mod module;
pub mod net;
pub(crate) mod process;
pub mod serializer;
pub(crate) mod server;
pub(crate) mod supervisor;
mod tag;
pub(crate) mod task;

use std::marker::PhantomData;

pub use config::ProcessConfig;
pub use error::LunaticError;
pub use macros::*;
pub use mailbox::{Mailbox, ReceiveError};
pub use module::WasmModule;
pub use process::Process;
pub use server::{Message, Request, Server, ServerMessage, ServerRef, ServerRequest, StartServer};
pub use tag::Tag;
pub use task::Task;

pub use lunatic_macros::main;
// TODO: Figure out testing (https://github.com/lunatic-solutions/rust-lib/issues/8)
// pub use lunatic_macros::test;

/// Implemented for all resources held by the VM.
pub trait Resource {
    /// Returns process local resource id.
    fn id(&self) -> u64;
    /// Turns process local resource id into resource handle.
    ///
    /// # Safety
    ///
    /// Extra care needs to be taken when balancing host side resources. It's easy to create an
    /// invalid resource reference.
    unsafe fn from_id(id: u64) -> Self;
}

/// Returns a handle to the current process.
///
/// The reference to the current mailbox is required to assure that the returned process matches
/// the mailbox.
pub fn this_process<M, S>(_mailbox: &Mailbox<M, S>) -> Process<M, S>
where
    S: serializer::Serializer<M>,
{
    let id = unsafe { host::api::process::this() };
    unsafe { Process::from_id(id) }
}

/// Suspends the current process for `duration` of time.
pub fn sleep(duration: std::time::Duration) {
    unsafe { host::api::process::sleep_ms(duration.as_millis() as u64) };
}
