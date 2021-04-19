mod receiver;
mod sender;

pub use receiver::Receiver;
pub use sender::Sender;

use serde::{de, ser};

mod stdlib {
    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn channel(bound: usize, receiver: *mut u32) -> u32;
    }
}

/// Creates a bounded channel.
///
/// The created channel has space to hold at most `cap` messages at a time.
///
/// # Panics
///
/// Capacity must be a positive number. If `cap` is zero, this function will panic.
pub fn bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>)
where
    T: ser::Serialize + de::DeserializeOwned,
{
    assert!(cap > 0, "capacity cannot be zero");
    unbounded()
}

/// Creates an unbounded channel.
///
/// The created channel can hold an unlimited number of messages.
pub fn unbounded<T>() -> (Sender<T>, Receiver<T>)
where
    T: ser::Serialize + de::DeserializeOwned,
{
    let mut receiver_id: u32 = 0;
    let sender_id = unsafe { stdlib::channel(0, &mut receiver_id as *mut u32) };
    (Sender::from(sender_id), Receiver::from(receiver_id))
}
