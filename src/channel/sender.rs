use serde::{
    de::{self, Visitor},
    ser, Serializer,
};
use std::{fmt, marker::PhantomData, rc::Rc};

mod stdlib {
    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn close_sender(sender: u32);
        pub fn sender_serialize(sender: u32) -> u32;
        pub fn sender_deserialize(index: u32) -> u32;
        pub fn channel_send(channel: u32, data: *const u8, data_len: usize) -> u32;
    }
}

/// The sending side of a channel.
///
/// Senders can be cloned and shared among processes. When all senders associated with a channel are
/// dropped, the channel becomes closed.
pub struct Sender<T> {
    inner: Rc<SenderInner>,
    phantom: PhantomData<T>,
}

// See: https://github.com/rust-lang/rust/issues/26925
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom: PhantomData,
        }
    }
}

struct SenderInner {
    id: u32,
}

impl Drop for SenderInner {
    fn drop(&mut self) {
        unsafe { stdlib::close_sender(self.id) };
    }
}

impl<T> Sender<T>
where
    T: ser::Serialize + de::DeserializeOwned,
{
    pub(crate) fn from(id: u32) -> Sender<T> {
        Sender {
            inner: Rc::new(SenderInner { id }),
            phantom: PhantomData,
        }
    }

    /// Sends a message into the channel.
    ///
    /// If the channel is full, this method waits until there is space for a message.
    ///
    /// If the channel is closed, this method returns an error.
    pub fn send(&self, value: T) -> Result<(), ()> {
        let value_serialized = bincode::serialize(&value).unwrap();
        let result = unsafe {
            stdlib::channel_send(
                self.inner.id,
                value_serialized.as_ptr(),
                value_serialized.len(),
            )
        };

        if result == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<T> ser::Serialize for Sender<T>
where
    T: ser::Serialize + de::DeserializeOwned,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let index = unsafe { stdlib::sender_serialize(self.inner.id) };
        serializer.serialize_u32(index)
    }
}

struct SenderVisitor<T> {
    phantom: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for SenderVisitor<T>
where
    T: ser::Serialize + de::DeserializeOwned,
{
    type Value = Sender<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -0 and 2^32")
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let id = unsafe { stdlib::sender_deserialize(value) };
        Ok(Sender::from(id))
    }
}

impl<'de, T> de::Deserialize<'de> for Sender<T>
where
    T: ser::Serialize + de::DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Sender<T>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_u32(SenderVisitor {
            phantom: PhantomData,
        })
    }
}
