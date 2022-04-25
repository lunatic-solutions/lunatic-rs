This library allows you to build Rust applications that run on top of [Lunatic][1].

[**DOCS**](https://docs.rs/lunatic/latest/lunatic/) |
[**Join our community on Discord!**](https://discord.gg/b7zDqpXpB4)

### Why run on Lunatic?

Lunatic provides an [Erlang][2]-like runtime for all programming languages that compile to
[WebAssembly][3]. It's all about spawning _super lightweight_ processes, also known as green
threads or [go-routines][5] in other runtimes. Lunatic processes are fast to create, have a small
memory footprint and a low scheduling overhead. They are designed for **massive** concurrency.

Lunatic processes are completely isolated from each other, they have their own stack, heap and even
syscalls. If one process fails it will not affect the rest of the system. This allows you to create
powerful and fault-tolerant applications.

All processes running on lunatic are preemptively scheduled and executed by a
[work stealing async executor][6]. This gives you the freedom to write simple _blocking_ code, but
the runtime is going to make sure it actually never blocks a thread if waiting on I/O.

To learn more about lunatic's architecture check out the [runtime repository][1]. It's written in
Rust :)

### Example

Spawning a new process is as simple as defining an entry function.

```rust
use lunatic::{spawn_link, Mailbox};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let child = spawn_link!(@task || {
        // This closure gets a new heap and stack to
        // execute on, and can't access the memory of
        // the parent process.
        println!("Hi! I'm a process.");
    });
    // Wait for child to finish
    let _ignore = child.result();
}
```

Check out more examples [here](https://github.com/lunatic-solutions/rust-lib/tree/main/examples).

### Setup

To run the example you will first need to download the lunatic runtime by following the
installation steps in [this repository][1]. The runtime is just single executable and runs on
Windows, macOS and Linux. If you have already Rust installed, you can get it with:
```bash
cargo install lunatic-runtime
```

[Lunatic][1] applications need to be compiled to [WebAssembly][3] before they can be executed by
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

Lunatic provides a macro `#[lunatic::test]` to turn your tests into processes. Check out the
`tests` folder for examples.

### Supported lunatic features

Some features are directly supported through Rust's standard library, like filesystem access
(`std::fs::File`). Others are specific to lunatic, like process spawning (`lunatic::Process`).

Some features that are usually available in Rust's standard library (like TCP, e.g.
`std::net::TcpListener`) are not standardized yet by [WASI][4]. So we made them available through
**this library** (e.g. `lunatic::net::TcpListener`). Once WASI gets support for these features you
will be able to just use the standard library instead.

What currently works:

- [x] **Process creation** (with this library)
- [x] **Fine-grained process permissions** (with this library)
- [x] **Message passing between processes** (with this library)
- [x] **TCP networking** (with this library)
- [x] **Filesystem access**
- [x] **Environment variables**
- [ ] **Distributed lunatic**

> **NOTE:**
> Some libraries currently don't compile under the target `wasm32-wasi` and can't be used inside
> lunatic applications. This includes most of Rust's `async` ecosystem.

[1]: https://github.com/lunatic-solutions/lunatic
[2]: https://www.erlang.org/
[3]: https://webassembly.org/
[4]: https://wasi.dev/
[5]: https://golangbot.com/goroutines
[6]: https://tokio.rs/
