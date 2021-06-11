use std::mem::{forget, transmute};

use serde::{de, ser, Deserialize, Serialize};

use crate::channel::Receiver;

mod stdlib {
    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn spawn_with_context(
            function: unsafe extern "C" fn(),
            buf_ptr: *const u8,
            buf_len: usize,
        ) -> u32;

        pub fn detach_process(pid: u32);
        pub fn cancel_process(pid: u32);
        pub fn join(pid: u32) -> u32;
        pub fn sleep_ms(millis: u64);
    }
}

/// A `Process` consists of its own stack and heap. It can only share data with other processes
/// through message passing.
///
/// Dropping a `Process` cancels it. To drop the Task handle without canceling it, use `detach()`
/// instead.
#[must_use = "`Process`es are cancelled when they are dropped. To avoid dropping this process
immediately, you should use it (e.g. by calling `Process::detach` or `Process::join`)."]
pub struct Process {
    id: u32,
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { stdlib::cancel_process(self.id) };
    }
}

impl Process {
    pub(crate) fn from(id: u32) -> Process {
        Process { id }
    }

    /// Spawns a new process from a function and context.
    ///
    /// - `function` is the starting point of the new process. The new process doesn't share
    ///   memory with its parent, because of this the function can't capture anything from parents.
    ///
    /// - `context` is  data that we want to pass to the newly spawned process. It needs to be
    ///    serializable.
    ///
    /// Safety:
    /// Rust doesn't have a concept of "separate" memories and you will still be able to reference
    /// global static variables, but **IT'S NOT SAFE** to do so.
    pub fn spawn_with<T>(context: T, function: fn(T)) -> Process
    where
        T: ser::Serialize + de::DeserializeOwned,
    {
        unsafe extern "C" fn spawn_with_context<'de, T>()
        where
            T: ser::Serialize + de::DeserializeOwned,
        {
            let receiver: Receiver<Context<T>> = Receiver::from(0);
            let context: Context<T> = receiver.receive().unwrap();
            let function: fn(T) = transmute(context.function_ptr);
            function(context.context);
        }

        let context = Context {
            function_ptr: unsafe { transmute(function) },
            context: context,
        };

        let context_serialized = bincode::serialize(&context).unwrap();

        let id = unsafe {
            stdlib::spawn_with_context(
                spawn_with_context::<T>,
                context_serialized.as_ptr(),
                context_serialized.len(),
            )
        };

        Self { id }
    }

    /// Detaches the `Process` to let it keep running in the background.
    pub fn detach(self) {
        unsafe { stdlib::detach_process(self.id) };
        // Avoid calling stdlib::cancel_process in the Drop implementation
        forget(self);
    }

    /// Waits on a `Process` to finish.
    ///
    /// Returns an error if the process failed.
    pub fn join(self) -> Result<(), ()> {
        let result = unsafe { stdlib::join(self.id) };
        // Avoid calling stdlib::cancel_process in the Drop implementation
        forget(self);
        match result {
            0 => Ok(()),
            _ => Err(()),
        }
    }

    /// Suspends the current process for `milliseconds`.
    pub fn sleep(milliseconds: u64) {
        unsafe {
            stdlib::sleep_ms(milliseconds);
        };
    }
}

#[derive(Serialize, Deserialize)]
struct Context<T> {
    function_ptr: usize,
    context: T,
}
