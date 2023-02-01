//! Serializer implementations for messages.
use thiserror::Error;

use crate::host::api::message;

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("serialization to Bincode failed: {0}")]
    Bincode(#[from] bincode::Error),
    #[cfg(feature = "msgpack_serializer")]
    #[cfg_attr(docsrs, doc(cfg(feature = "msgpack_serializer")))]
    #[error("serialization to MessagePack failed: {0}")]
    MessagePack(#[from] rmp_serde::encode::Error),
    #[cfg(feature = "json_serializer")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json_serializer")))]
    #[error("serialization to Json failed: {0}")]
    Json(#[from] serde_json::error::Error),
    #[cfg(feature = "protobuf_serializer")]
    #[cfg_attr(docsrs, doc(cfg(feature = "protobuf_serializer")))]
    #[error("serialization to Protocol Buffers failed: {0}")]
    ProtocolBuffers(#[from] protobuf::Error),
    #[error("serialization failed: {0}")]
    IO(#[from] std::io::Error),
    #[error("serialization failed: {0}")]
    Custom(String),
}

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("deserialization from Bincode failed: {0}")]
    Bincode(#[from] bincode::Error),
    #[cfg(feature = "msgpack_serializer")]
    #[cfg_attr(docsrs, doc(cfg(feature = "msgpack_serializer")))]
    #[error("deserialization from MessagePack failed: {0}")]
    MessagePack(#[from] rmp_serde::decode::Error),
    #[cfg(feature = "json_serializer")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json_serializer")))]
    #[error("deserialization from Json failed: {0}")]
    Json(#[from] serde_json::error::Error),
    #[cfg(feature = "protobuf_serializer")]
    #[cfg_attr(docsrs, doc(cfg(feature = "protobuf_serializer")))]
    #[error("deserialization from Protocol Buffers failed: {0}")]
    ProtocolBuffers(#[from] protobuf::Error),
    #[error("serialization failed: {0}")]
    IO(#[from] std::io::Error),
    #[error("deserialization failed: {0}")]
    Custom(String),
}

/// The `CanSerialize` trait is implemented for serializers that can encode and
/// decode the type `M`.
///
/// This trait is used across the whole lunatic API to allow us to switch out
/// serializers when sending messages between processes. Lunatic already ships
/// with support for a few well known serialization formats like json, message
/// pack, protocol buffers and bincode, that can be enabled with feature flags.
/// You can add others by implementing this trait.
///
/// The generic parameter `M` can be used to express trait dependencies on
/// messages for each concrete serializer type. Let's say we want to use
/// `Bincode` to serialize messages, we still need to limit the message types to
/// a specific subset that implement the `serde::Serialize` and
/// `serde::Deserialize` traits. We can express this dependency in the following
/// way:
///
/// ```no_run
/// impl<M: serde::Serialize + serde::Deserialize> Serializer<M> for Bincode {
///     fn encode(message: M) -> Error {
///         // `message` is guaranteed to implement the `serde::Serialize`
///         // trait and can be encoded here using `Bincode`.
///     }
///     // ...
/// }
/// ```
///
/// Serializers that can work with the [`Read`](std::io::Read) &
/// [`Write`](std::io::Write) traits are generally better suited for lunatic's
/// FFI, that works on a streaming basis and can avoid unnecessary copies.
/// Serializer that require raw access to chunks of mutable memories (e.g.
/// `Prost`) require additional copies between guest and host memories.
pub trait CanSerialize<M> {
    fn encode(message: &M) -> Result<(), EncodeError>;
    fn decode() -> Result<M, DecodeError>;
}

/// A `Bincode` serializer.
///
/// It can serialize any message that satisfies the traits:
/// - `serde::Serialize`
/// - `serde::de::DeserializeOwned`
///
/// `serde::de::DeserializeOwned` is used here instead of
/// `serde::Deserialize<'de>` because the messages are extracted from a stream
/// that lives inside the VM, has an unknown lifetime and can't be referenced
/// from the guest. `serde::de::DeserializeOwned` is automatically implemented
/// for each type that also implements `serde::Deserialize<'de>`.
#[derive(Hash, Debug)]
pub struct Bincode {}

impl<M> CanSerialize<M> for Bincode
where
    M: serde::Serialize + serde::de::DeserializeOwned,
{
    fn encode(message: &M) -> Result<(), EncodeError> {
        Ok(bincode::serialize_into(MessageRw {}, message)?)
    }

    fn decode() -> Result<M, DecodeError> {
        Ok(bincode::deserialize_from(MessageRw {})?)
    }
}

/// A `MessagePack` serializer.
///
/// It can serialize any message that satisfies the traits:
/// - `serde::Serialize`
/// - `serde::de::DeserializeOwned`
///
/// Refer to the [`Bincode`] docs for the difference between
/// `serde::de::DeserializeOwned` and `serde::Deserialize<'de>`.
#[cfg(feature = "msgpack_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "msgpack_serializer")))]
#[derive(Debug, Hash)]
pub struct MessagePack {}

#[cfg(feature = "msgpack_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "msgpack_serializer")))]
impl<M> CanSerialize<M> for MessagePack
where
    M: serde::Serialize + serde::de::DeserializeOwned,
{
    fn encode(message: &M) -> Result<(), EncodeError> {
        use std::io::Write;
        let data = rmp_serde::to_vec(message)?;
        Ok(MessageRw {}.write_all(&data)?)
    }

    fn decode() -> Result<M, DecodeError> {
        Ok(rmp_serde::decode::from_read(MessageRw {})?)
    }
}

/// A `Json` serializer.
///
/// It can serialize any message that satisfies the traits:
/// - `serde::Serialize`
/// - `serde::de::DeserializeOwned`
///
/// Refer to the [`Bincode`] docs for the difference between
/// `serde::de::DeserializeOwned` and `serde::Deserialize<'de>`.
#[cfg(feature = "json_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "json_serializer")))]
#[derive(Debug, Hash)]
pub struct Json {}

#[cfg(feature = "json_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "json_serializer")))]
impl<M> CanSerialize<M> for Json
where
    M: serde::Serialize + serde::de::DeserializeOwned,
{
    fn encode(message: &M) -> Result<(), EncodeError> {
        use std::io::Write;
        let data = serde_json::to_vec(message)?;
        Ok(MessageRw {}.write_all(&data)?)
    }

    fn decode() -> Result<M, DecodeError> {
        Ok(serde_json::from_reader(MessageRw {})?)
    }
}

/// The `ProtocolBuffers` serializer can serialize any message that satisfies
/// the trait `protobuf::Message`.
#[cfg(feature = "protobuf_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "protobuf_serializer")))]
#[derive(Debug, Hash)]
pub struct ProtocolBuffers {}

#[cfg(feature = "protobuf_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "protobuf_serializer")))]
impl<M> CanSerialize<M> for ProtocolBuffers
where
    M: protobuf::Message,
{
    fn encode(message: &M) -> Result<(), EncodeError> {
        use std::io::Write;
        let mut data = Vec::new();
        message.write_to_vec(&mut data)?;
        Ok(MessageRw {}.write_all(&data)?)
    }

    fn decode() -> Result<M, DecodeError> {
        Ok(M::parse_from_reader(&mut MessageRw {})?)
    }
}

/// A helper struct to read from and write to the message scratch buffer.
///
/// It simplifies streaming serialization/deserialization directly from the host
/// and avoids copies. Most serde based serializers can work directly with
/// streaming serialization.
#[derive(Debug, Hash)]
pub struct MessageRw {}

impl std::io::Read for MessageRw {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(unsafe { message::read_data(buf.as_mut_ptr(), buf.len()) })
    }
}

impl std::io::Write for MessageRw {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(unsafe { message::write_data(buf.as_ptr(), buf.len()) })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
