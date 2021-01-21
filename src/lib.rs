//! This library contains higher level wrappers for low level Lunatic syscalls.

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
