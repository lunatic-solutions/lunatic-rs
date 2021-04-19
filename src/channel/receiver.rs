use serde::{
    de::{self, Visitor},
    ser, Serializer,
};
use std::{
    alloc::{alloc, dealloc, Layout},
    fmt,
    marker::PhantomData,
    rc::Rc,
    slice::from_raw_parts,
};

mod stdlib {
    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn close_receiver(sender: u32);
        pub fn drop_last_message();
        pub fn receiver_serialize(sender: u32) -> u32;
        pub fn receiver_deserialize(index: u32) -> u32;
        pub fn channel_receive_prepare(channel: u32, buf_len: *mut u32) -> usize;
        pub fn channel_receive(buf: *mut u8, buf_len: usize) -> u32;
    }
}

/// The receiving side of a channel.
///
/// Receivers can be cloned and shared among processes. When all receivers associated with a channel
/// are dropped, the channel becomes closed.
pub struct Receiver<T> {
    inner: Rc<ReceiverInner>,
    phantom: PhantomData<T>,
}

// See: https://github.com/rust-lang/rust/issues/26925
impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom: PhantomData,
        }
    }
}

struct ReceiverInner {
    id: u32,
}

impl Drop for ReceiverInner {
    fn drop(&mut self) {
        unsafe { stdlib::close_receiver(self.id) };
    }
}

impl<T> Receiver<T>
where
    T: ser::Serialize + de::DeserializeOwned,
{
    pub(crate) fn from(id: u32) -> Receiver<T> {
        Receiver {
            inner: Rc::new(ReceiverInner { id }),
            phantom: PhantomData,
        }
    }

    /// Receives a message from the channel.
    ///
    /// If the channel is empty, this method waits until there is a message.
    ///
    /// If the channel is closed, this method receives a message or returns an error if there are
    /// no more messages.
    pub fn receive(&self) -> Result<T, ()> {
        let mut buf_len: u32 = 0;
        let result =
            unsafe { stdlib::channel_receive_prepare(self.inner.id, &mut buf_len as *mut u32) };
        if result == 1 {
            return Err(());
        }

        // Allocate buffer on guest to copy serialized data into.
        let buffer: *mut u8 = unsafe {
            let layout = Layout::from_size_align(buf_len as usize, 16).expect("Invalid layout");
            alloc(layout)
        };

        let result = unsafe { stdlib::channel_receive(buffer, buf_len as usize) };
        if result == 1 {
            return Err(());
        }

        let slice = unsafe { from_raw_parts(buffer, buf_len as usize) };
        let result: T = bincode::deserialize(slice).unwrap();

        unsafe {
            // Free buffer on host side
            stdlib::drop_last_message();
            // Free guest buffer
            let layout = Layout::from_size_align(buf_len as usize, 16).expect("Invalid layout");
            dealloc(buffer, layout);
        }

        Ok(result)
    }
}

impl<T> ser::Serialize for Receiver<T>
where
    T: ser::Serialize + de::DeserializeOwned,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let index = unsafe { stdlib::receiver_serialize(self.inner.id) };
        serializer.serialize_u32(index)
    }
}

struct ReceiverVisitor<T> {
    phantom: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for ReceiverVisitor<T>
where
    T: ser::Serialize + de::DeserializeOwned,
{
    type Value = Receiver<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -0 and 2^32")
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let id = unsafe { stdlib::receiver_deserialize(value) };
        Ok(Receiver::from(id))
    }
}

impl<'de, T> de::Deserialize<'de> for Receiver<T>
where
    T: ser::Serialize + de::DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Receiver<T>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_u32(ReceiverVisitor {
            phantom: PhantomData,
        })
    }
}
