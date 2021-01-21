//! This library allows you to build Rust applications that run on top of [Lunatic][1].
//!
//! [**Join our growing community on Discord!**](https://!discord.gg/b7zDqpXpB4)
//!
//! ### Why would you want to run on top of Lunatic?
//!
//! Lunatic provides an [Erlang][2] like runtime for all programming languages that compile to [WebAssembly][3].
//! It's all about spawning _super lightweight_ processes, also known as green threads or [go-routines][5] in other
//! runtimes. Lunatic processes are fast to create, have a small memory footprint and a low scheduling
//! overhead. They are designed for **MASSIVE** concurrency.
//!
//! Lunatic processes are completely isolated from each other, they have their own stack, heap and even syscalls. If one
//! process fails it will not affect the rest of the system. This allows you to create very powerful and fault-tolerant
//! abstraction.
//!
//! All processes running on Lunatic are preemptively scheduled and executed by a [work steeling async executor][6]. This
//! gives you the freedom to write simple _blocking_ code, but the runtime is going to make sure it actually never blocks
//! a thread if waiting on I/O.
//!
//! To learn more about Lunatic's architecture check out the [runtime repository][1]. It's written in Rust :)
//!
//! ### Example
//!
//! This example application spawns a process and waits for it to print a message on the screen.
//!
//! ```rust
//! use  lunatic::Process;
//!
//! fn  main()  {
//! 	Process::spawn((),  |_:  ()|  {
//! 		println!("Hello from inside the new process!");
//! 	})
//! 	.unwrap()
//! 	.join();
//! }
//! ```
//!
//! Check out more examples [here](https://!github.com/lunatic-solutions/rust-lib/examples).
//!
//! ### Requirements
//!
//! To run the example you will first need to download the Lunatic runtime by following the installation steps in
//! [this repository][1].
//!
//! [Lunatic][1] applications need to be compiled to [WebAssembly][3] before they can be executed by the runtime.
//! Rust has great support for WebAssembly and you can build a Lunatic compatible application just by passing the
//! `--target=wasm32-wasi` flag to cargo, e.g:
//!
//! ```
//! cargo build --release --target=wasm32-wasi
//! ```
//!
//! This will generate a .wasm file in the `target/wasm32-wasi/release/` folder inside your project.
//! You can now run your application by passing the generated .wasm file to Lunatic, e.g:
//!
//! ```
//! lunatic target/wasm32-wasi/release/<name>.wasm
//! ```
//!
//! ### Supported Features
//!
//! Some features are directly supported through Rust's standard library, like filesystem access (`std::fs::File`).
//! Others are specific to Lunatic, like process spawning (`lunatic::Process`).
//!
//! Some features that are usually available in Rust's standard library (like TCP, e.g. `std::net::TcpListener`) are
//! not standardised by [WASI][4] yet. So we made them available through **this library** (e.g. `lunatic::net::TcpListener`).
//! Once WASI gets support for this features you will be able to just use the stand library versions.
//!
//! What currently works:
//!
//! - [x] **Process creation & joining** (with this library)
//! - [ ] **Fine-grained process permissions** (with this library)
//! - [x] **Channel based message passing** (with this library)
//! - [x] **TCP networking** (with this library)
//! - [ ] **Filesystem access**
//! - [x] **Environment variables**
//! - [ ] **Multithreading**
//!
//! > **NOTE:**
//! > Some libraries currently don't compile under the target `wasm32-wasi` and can't be used inside Lunatic applications.
//!
//! [1]: https://!github.com/lunatic-solutions/lunatic
//! [2]: https://!www.erlang.org/
//! [3]: https://!webassembly.org/
//! [4]: https://!wasi.dev/
//! [5]: https://!golangbot.com/goroutines
//! [6]: https://!docs.rs/smol

pub mod channel;
pub mod net;
pub mod process;

pub use process::Process;

mod stdlib {
    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn yield_();
    }
}

/// Yields current process.
pub fn yield_() {
    unsafe {
        stdlib::yield_();
    }
}
