This library allows you to build Rust applications that run on top of [Lunatic][1].

[**DOCS**](https://docs.rs/lunatic/latest/lunatic/) |
[**Join our community on Discord!**](https://discord.gg/b7zDqpXpB4)

### Why run on Lunatic?

Lunatic provides an [Erlang][2] like runtime for all programming languages that compile to
[WebAssembly][3]. It's all about spawning _super lightweight_ processes, also known as green
threads or [go-routines][5] in other runtimes. Lunatic processes are fast to create, have a small
memory footprint and a low scheduling overhead. They are designed for **massive** concurrency.

Lunatic processes are completely isolated from each other, they have their own stack, heap and even
syscalls. If one process fails it will not affect the rest of the system. This allows you to create
powerful and fault-tolerant abstraction.

All processes running on Lunatic are preemptively scheduled and executed by a
[work stealing async executor][6]. This gives you the freedom to write simple _blocking_ code, but
the runtime is going to make sure it actually never blocks a thread if waiting on I/O.

To learn more about Lunatic's architecture check out the [runtime repository][1]. It's written in
Rust :)

### Examples

Spawning a new process is as simple as passing a function to it.

```rust

use lunatic::{process, Mailbox};

#[lunatic::main]
fn main(m: Mailbox<()>) {
    // Get handle to itself.
    let this = process::this(&m);
    process::spawn_with(this, |parent, _: Mailbox<()>| {
        // This closure gets a new heap and stack to
        // execute on, and can't access the memory of
        // the parent process.
        println!("Hi! I'm a process.");
    })
    .unwrap();
    // Wait for child to finish.
    let _ignore = m.receive();
}
```

Limit resources and syscalls for process by defining an environment of execution.

```rust
use lunatic::{Config, Environment, Mailbox};

#[lunatic::main]
fn main(m: Mailbox<()>) {
    // Create a new environment where processes can use
    // maximum 17 Wasm pages of memory (17 * 64KB) and one
    // unit of compute (~=100k CPU cycles).
    let mut config = Config::new(1_200_000, Some(1));
    // Allow only syscalls under the
    // "wasi_snapshot_preview1::environ*" namespace
    config.allow_namespace("wasi_snapshot_preview1::environ");
    let mut env = Environment::new(config).unwrap();
    let module = env.add_this_module().unwrap();

    // This process will fail because it can't uses syscalls
    // for std i/o
    let (_, m) = module
        .spawn_link(m, |_: Mailbox<()>| {
            println!("Hi from different env");
        })
        .unwrap();
    assert!(m.receive().is_signal());

    // This process will fail because it uses too much memory
    let (_, m) = module
        .spawn_link(m, |_: Mailbox<()>| {
            vec![0; 150_000];
        })
        .unwrap();
    assert!(m.receive().is_signal());

    // This process will fail because it uses too much
    // compute
    let (_, m) = module.spawn_link(m, |_: Mailbox<()>|
        loop {}
    ).unwrap();
    assert!(m.receive().is_signal());
}
```

TCP echo server:

```rust
use lunatic::{net, process, Mailbox};
use std::io::{BufRead, BufReader, Write};

fn main() {
    let listener =
        net::TcpListener::bind("127.0.0.1:1337").unwrap();
    while let Ok((tcp_stream, _peer)) = listener.accept() {
        // Pass the TCP stream as a context to the new
        // process. We can't use a closures that capture
        // parent variables because no memory is shared
        // between processes.
        process::spawn_with(tcp_stream, handle).unwrap();
    }
}

fn handle(mut tcp_stream: net::TcpStream, _: Mailbox<()>) {
    let mut buf_reader = BufReader::new(tcp_stream.clone());
    loop {
        let mut buffer = String::new();
        let read =
            buf_reader.read_line(&mut buffer).unwrap();
        if buffer.contains("exit") || read == 0 {
            return;
        }
        tcp_stream.write(buffer.as_bytes()).unwrap();
    }
}

```

Check out more examples [here](https://github.com/lunatic-solutions/rust-lib/tree/main/examples).

### Requirements

To run the example you will first need to download the Lunatic runtime by following the
installation steps in [this repository][1].

[Lunatic][1] applications need to be compiled to [WebAssembly][3] before they can be executed by
the runtime. Rust has great support for WebAssembly and you can build a Lunatic compatible application
just by passing the `--target=wasm32-wasi` flag to cargo, e.g:

```
cargo build --release --target=wasm32-wasi
```

This will generate a .wasm file in the `target/wasm32-wasi/release/` folder inside your project.
You can now run your application by passing the generated .wasm file to Lunatic, e.g:

```
lunatic target/wasm32-wasi/release/<name>.wasm
```

#### Setting up Cargo for lunatic

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

### Debugging

If a process dies, either because an unsupported syscall was called or a `Kill` signal was received
there is not going to be any output in the terminal. To get more insight set the `RUST_LOG`
environment variable to `lunatic=debug`. E.g. `RUST_LOG=lunatic=debug cargo run`.

### Supported Features

Some features are directly supported through Rust's standard library, like filesystem access
(`std::fs::File`). Others are specific to Lunatic, like process spawning (`lunatic::Process`).

Some features that are usually available in Rust's standard library (like TCP, e.g.
`std::net::TcpListener`) are not standardised yet by [WASI][4]. So we made them available through
**this library** (e.g. `lunatic::net::TcpListener`). Once WASI gets support for this features you
will be able to just use the standard library versions.

What currently works:

- [x] **Process creation & joining** (with this library)
- [x] **Fine-grained process permissions** (with this library)
- [x] **Message passing between processes** (with this library)
- [x] **TCP networking** (with this library)
- [x] **Filesystem access**
- [x] **Environment variables**
- [ ] **Distributed lunatic**

> **NOTE:**
> Some libraries currently don't compile under the target `wasm32-wasi` and can't be used inside
> Lunatic applications. This includes most of Rust's `async` libraries.

[1]: https://github.com/lunatic-solutions/lunatic
[2]: https://www.erlang.org/
[3]: https://webassembly.org/
[4]: https://wasi.dev/
[5]: https://golangbot.com/goroutines
[6]: https://tokio.rs/
