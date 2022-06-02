/*!
Framework for building Rust applications that run on [lunatic][1].

# Main concepts

The main abstraction in [lunatic][1] is a process. Contrary to operating system processes,
lunatic processes are lightweight and fast to spawn. They are designed for **massive**
concurrency. Like operating system processes, they are sandboxed. Each of them gets a separate
memory and they can't access the memory from other processes, not even through raw pointers.
If we want to exchange any information between process we need to do it through message passing.

This library makes processes feel native to the Rust language. They can be spawned from just a
function.

### Process types:

* **[`Process`]** - A process that can receive messages through a [`Mailbox`] or
    [`Protocol`](protocol::Protocol).
* **[`AbstractProcess`](process::AbstractProcess)** - Abstracts state managment and message/request
    handling.
* **[`Supervisor`](supervisor::Supervisor)** - A process that can supervise others and re-spawn
    them if they fail.

### Linking

Processes can be linked together. This means that if one of them fails, all linked ones will be
killed too.

To spawn a linked process use the [`spawn_link`] function.

### Process configuration

Spawn functions have a variant that takes a [`ProcessConfig`]. This configuration can be used
to set a memory or CPU limit on the newly spawned process. It can also be used to control file
and network access permissions of processes.

### Setup

To run the example you will first need to download the lunatic runtime by following the
installation steps in [this repository][1]. The runtime is just single executable and runs on
Windows, macOS and Linux. If you have already Rust installed, you can get it with:
```bash
cargo install lunatic-runtime
```

[Lunatic][1] applications need to be compiled to [WebAssembly][2] before they can be executed by
the runtime. Rust has great support for WebAssembly and you can build a lunatic compatible
application just by passing the `--target=wasm32-wasi` flag to cargo, e.g:

```bash
# Add the WebAssembly target
rustup target add wasm32-wasi
# Build the app
cargo build --release --target=wasm32-wasi
```

This will generate a .wasm file in the `target/wasm32-wasi/release/` folder inside your project.
You can now run your application by passing the generated .wasm file to Lunatic, e.g:

```
lunatic target/wasm32-wasi/release/<name>.wasm
```

#### Better developer experience

To simplify developing, testing and running lunatic applications with cargo, you can add a
`.cargo/config.toml` file to your project with the following content:

```toml
[build]
target = "wasm32-wasi"

[target.wasm32-wasi]
runner = "lunatic"
```

Now you can just use the commands you were already familiar with, such as `cargo run`, `cargo test`
and cargo is going to automatically build your project as a WebAssembly module and run it inside
`lunatic`.

### Testing

Lunatic provides a macro [`test`] to turn your tests into processes. Check out the [`tests`][3]
directory for examples.

[1]: https://github.com/lunatic-solutions/lunatic
[2]: https://webassembly.org/
[3]: https://github.com/lunatic-solutions/rust-lib/tree/main/tests

*/

mod config;
mod error;
mod function_process;
mod macros;
mod mailbox;
mod module;
mod tag;

pub mod host;
pub mod net;
pub mod process;
pub mod protocol;
pub mod serializer;
pub mod supervisor;
pub mod timer;

pub use config::ProcessConfig;
pub use error::LunaticError;
pub use function_process::Process;
pub use mailbox::{Mailbox, ReceiveError};
pub use module::WasmModule;
pub use tag::Tag;

pub use lunatic_macros::main;
pub use lunatic_test::test;

/// Implemented for all resources held by the VM.
pub trait Resource {
    /// Returns process local resource ID.
    fn id(&self) -> u64;
    /// Turns process local resource ID into resource handle.
    ///
    /// # Safety
    ///
    /// Extra care needs to be taken when balancing host side resources. It's easy to create an
    /// invalid resource reference.
    unsafe fn from_id(id: u64) -> Self;
}

/// Suspends the current process for `duration` of time.
pub fn sleep(duration: std::time::Duration) {
    unsafe { host::api::process::sleep_ms(duration.as_millis() as u64) };
}
