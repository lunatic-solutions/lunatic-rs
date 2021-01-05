use std::ffi::c_void;
use std::io::{IoSlice, IoSliceMut};
use std::marker::PhantomData;

use serde::{de, ser, Deserialize, Serialize};

mod stdlib {
    use std::ffi::c_void;

    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn channel_open(bound: usize) -> u32;
        pub fn channel_close(channel: u32);
        pub fn channel_send(channel: u32, data: *const c_void, data_len: usize);
        pub fn channel_receive_prepare(channel: u32) -> usize;
        pub fn channel_receive(buf: *mut c_void, buf_len: usize);
    }
}

/// A channel allows exchanging messages between processes.
/// The message needs to implement `serde::ser::Serializer`, because processes don't share any memory.
#[derive(Clone, Serialize, Deserialize)]
pub struct Channel<T> {
    id: u32,
    phantom: PhantomData<T>,
}

impl<T: ser::Serialize + de::DeserializeOwned> Channel<T> {
    /// Create a new channel.
    /// If `bound` is 0, returns an unbound channel.
    pub fn new(bound: usize) -> Self {
        let id = unsafe { stdlib::channel_open(bound) };
        Self {
            id,
            phantom: PhantomData,
        }
    }

    pub fn from(id: u32) -> Self {
        Self {
            id,
            phantom: PhantomData,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    /// Send message to channel.
    pub fn send(&self, value: T) {
        let value_serialized = bincode::serialize(&value).unwrap();
        let io_slices = &[IoSlice::new(value_serialized.as_slice())];

        unsafe {
            stdlib::channel_send(
                self.id,
                io_slices.as_ptr() as *const c_void,
                io_slices.len(),
            );
        }
    }

    /// Receive message from channel.
    pub fn receive(&self) -> T {
        let message_size = unsafe { stdlib::channel_receive_prepare(self.id) };
        let mut buffer: Vec<u8> = vec![0; message_size];
        let buffer_ref = buffer.as_mut_slice();
        let mut io_slice = [IoSliceMut::new(buffer_ref)];
        let io_slice_ref = &mut io_slice;
        let _buf_len = unsafe {
            stdlib::channel_receive(io_slice_ref.as_mut_ptr() as *mut c_void, io_slice_ref.len())
        };

        let result: T = bincode::deserialize(&buffer[..]).unwrap();
        result
    }
}
