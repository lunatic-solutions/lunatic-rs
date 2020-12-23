use std::fmt;
use std::io::{IoSlice, IoSliceMut};
use std::marker::PhantomData;

use serde::de::{self, Deserialize, DeserializeOwned, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

mod stdlib {
    use std::io::{IoSlice, IoSliceMut};

    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn channel(bound: u32) -> i32;
        pub fn channel_send(channel: i32, data: *const [IoSlice<'_>]);
        pub fn channel_next_message_size(channel: i32) -> usize;
        pub fn channel_receive(channel: i32, buf: *mut [IoSliceMut<'_>]) -> usize;
        pub fn channel_serialize(channel: i32) -> u64;
        pub fn channel_deserialize(channel: u64) -> i32;
    }
}

/// A channel allows exchanging messages between processes.
/// The message needs to implement `serde::ser::Serializer`, because processes don't share any memory.
#[derive(Clone)]
pub struct Channel<T> {
    id: i32,
    phantom: PhantomData<T>,
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        // TODO
    }
}

impl<'de, T: Serialize + DeserializeOwned> Channel<T> {
    /// If `bound` is 0, returns an unbound channel.
    pub fn new(bound: usize) -> Self {
        let id = unsafe { stdlib::channel(bound as u32) };
        Self {
            id,
            phantom: PhantomData,
        }
    }

    pub fn send(&self, value: T) {
        let value_serialized = bincode::serialize(&value).unwrap();
        let io_slices = &[IoSlice::new(value_serialized.as_slice())];

        unsafe {
            stdlib::channel_send(self.id, io_slices as *const [IoSlice<'_>]);
        }
    }

    pub fn receive(&self) -> T {
        let message_size = unsafe { stdlib::channel_next_message_size(self.id) };
        let mut buffer: Vec<u8> = vec![0; message_size];
        let buffer_ref = buffer.as_mut_slice();
        let mut io_slice = [IoSliceMut::new(buffer_ref)];
        let io_slice_ref = &mut io_slice;
        let _buf_len =
            unsafe { stdlib::channel_receive(self.id, io_slice_ref as *mut [IoSliceMut]) };

        let result: T = bincode::deserialize(&buffer[..]).unwrap();
        result
    }

    pub fn serialize_as_u64(self) -> u64 {
        unsafe { stdlib::channel_serialize(self.id) }
    }

    pub fn deserialize_from_u64(id: u64) -> Self {
        let id = unsafe { stdlib::channel_deserialize(id) };
        Self {
            id,
            phantom: PhantomData,
        }
    }
}

impl<'de, T: Serialize + DeserializeOwned> Serialize for Channel<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serialized_channel = unsafe { stdlib::channel_serialize(self.id) };
        serializer.serialize_u64(serialized_channel)
    }
}

struct ChannelVisitor<T> {
    phantom: PhantomData<T>,
}

impl<'de, T: Serialize + DeserializeOwned> Visitor<'de> for ChannelVisitor<T> {
    type Value = Channel<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an pointer to an externref containing a channel")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let id = unsafe { stdlib::channel_deserialize(value) };
        Ok(Channel {
            id,
            phantom: PhantomData,
        })
    }
}

impl<'de, T: Serialize + DeserializeOwned> Deserialize<'de> for Channel<T> {
    fn deserialize<D>(deserializer: D) -> Result<Channel<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(ChannelVisitor {
            phantom: PhantomData,
        })
    }
}
