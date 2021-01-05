use std::mem::{forget, transmute};

use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use crate::Channel;

mod stdlib {
    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn spawn(
            function: unsafe extern "C" fn(usize, u32),
            argument1: usize,
            argument2: u32,
        ) -> i32;

        pub fn join(pid: i32);
        pub fn sleep_ms(millis: u64);
    }
}

#[derive(Debug)]
pub struct SpawnError {}

/// A process consists of its own stack and heap. It can only share data with other processes by
/// exchanging the data with messages passing.
pub struct Process {
    id: i32,
}

impl Drop for Process {
    fn drop(&mut self) {
        drop(self.id);
    }
}

impl Process {
    /// Spawn a new process from a function and cotext.
    /// `function` is going to be starting point of the new process.
    /// `context` is some data that we want to pass to the newly spawned process.
    pub fn spawn<T>(context: T, function: fn(T)) -> Result<Process, SpawnError>
    where
        T: Serialize + DeserializeOwned,
    {
        unsafe extern "C" fn spawn_with_context<'de, T>(function: usize, channel_id: u32)
        where
            T: Serialize + DeserializeOwned,
        {
            let channel: Channel<T> = Channel::from(channel_id);
            let context: T = channel.receive();
            let function: fn(T) = transmute(function);
            function(context);
        }

        let channel = Channel::new(1);
        channel.send(context);
        let channel_raw_id = channel.id();

        let id =
            unsafe { stdlib::spawn(spawn_with_context::<T>, transmute(function), channel_raw_id) };

        Ok(Self { id })
    }

    /// Wait on a process to finish.
    pub fn join(self) {
        unsafe {
            stdlib::join(self.id);
        };
        forget(self);
    }

    /// Suspends the current process for `milliseconds`.
    pub fn sleep(milliseconds: u64) {
        unsafe {
            stdlib::sleep_ms(milliseconds);
        };
    }
}
