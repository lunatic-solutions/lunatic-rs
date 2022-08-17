//! Serializer implementations for messages.

use crate::host::api::message;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    #[error("deserialization failed: {0}")]
    Custom(String),
}

/// The `Serializer` defines the format that messages are encoded to or decoded from when they
/// cross process boundaries.
///
/// Lunatic already ships with support for a few well known serialization formats like json,
/// message pack, protocol buffers and bincode, that can be enabled with feature flags. You can
/// add others by implementing this trait.
///
/// The generic parameter `M` can be used to express trait dependencies on messages for each
/// concrete serializer type. Let's say we want to use `Bincode` to serialize messages, we still
/// need to limit the message types to a specific subset that implement the `serde::Serialize`
/// and `serde::Deserialize` traits. We can express this dependency in the following way:
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
/// Serializers that can work with the [`Read`](std::io::Read) & [`Write`](std::io::Write) traits
/// are generally better suited for lunatic's ffi, that works on a streaming basis and can avoid
/// unnecessary copies. Serializer that require raw access to chunks of mutable memories (e.g.
/// Prost) require additional copies between guest and host memories.
pub trait Serializer<M> {
    fn encode(message: &M) -> Result<(), EncodeError>;
    fn decode() -> Result<M, DecodeError>;
}

/// A `Bincode` serializer.
///
/// It can serialize any message that satisfies the traits:
/// - `serde::Serialize`
/// - `serde::de::DeserializeOwned`
///
/// `serde::de::DeserializeOwned` is used here instead of `serde::Deserialize<'de>` because the
/// messages are extracted from a stream that lives inside of the VM, has an unknown lifetime and
/// can't be referenced from the guest. `serde::de::DeserializeOwned` is automatically implemented
/// for each type that also implements `serde::Deserialize<'de>`.
#[derive(Hash, Debug, Serialize, Deserialize)]
pub struct Bincode {}

impl<M> Serializer<M> for Bincode
where
    M: serde::Serialize + serde::de::DeserializeOwned,
{
    fn encode(message: &M) -> Result<(), EncodeError> {
        bincode::serialize_into(MessageRw {}, message).map_err(|err| err.into())
    }

    fn decode() -> Result<M, DecodeError> {
        bincode::deserialize_from(MessageRw {}).map_err(|err| err.into())
    }
}

/// A `MessagePack` serializer.
///
/// It can serialize any message that satisfies the traits:
/// - `serde::Serialize`
/// - `serde::de::DeserializeOwned`
///
/// Refer to the [`Bincode`] docs for the difference between `serde::de::DeserializeOwned` and
/// `serde::Deserialize<'de>`.
#[cfg(feature = "msgpack_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "msgpack_serializer")))]
#[derive(Debug, Hash, Serialize, Deserialize)]
pub struct MessagePack {}

#[cfg(feature = "msgpack_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "msgpack_serializer")))]
impl<M> Serializer<M> for MessagePack
where
    M: serde::Serialize + serde::de::DeserializeOwned,
{
    fn encode(message: &M) -> Result<(), EncodeError> {
        rmp_serde::encode::write(&mut MessageRw {}, message).map_err(|err| err.into())
    }

    fn decode() -> Result<M, DecodeError> {
        rmp_serde::decode::from_read(MessageRw {}).map_err(|err| err.into())
    }
}

/// A `Json` serializer.
///
/// It can serialize any message that satisfies the traits:
/// - `serde::Serialize`
/// - `serde::de::DeserializeOwned`
///
/// Refer to the [`Bincode`] docs for the difference between `serde::de::DeserializeOwned` and
/// `serde::Deserialize<'de>`.
#[cfg(feature = "json_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "json_serializer")))]
#[derive(Debug, Hash, Serialize, Deserialize)]
pub struct Json {}

#[cfg(feature = "json_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "json_serializer")))]
impl<M> Serializer<M> for Json
where
    M: serde::Serialize + serde::de::DeserializeOwned,
{
    fn encode(message: &M) -> Result<(), EncodeError> {
        use std::io::Write;

        let bytes = serde_json::to_vec(message)?;
        MessageRw {}.write_all(&bytes.len().to_le_bytes()).unwrap();
        MessageRw {}.write_all(&bytes).unwrap();
        Ok(())
    }

    fn decode() -> Result<M, DecodeError> {
        use std::io::{Read, Write};

        let mut le_bytes = [0; 4];
        MessageRw {}.read_exact(&mut le_bytes).unwrap();

        let size = usize::from_le_bytes(le_bytes);
        let mut buf = vec![0; size];
        MessageRw {}.read_exact(&mut buf).unwrap();

        serde_json::from_slice(&buf).map_err(|err| err.into())
    }
}

/// The `ProtocolBuffers` serializer can serialize any message that satisfies the trait
/// `protobuf::Message`.
#[cfg(feature = "protobuf_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "protobuf_serializer")))]
#[derive(Debug, Hash, Serialize, Deserialize)]
pub struct ProtocolBuffers {}

#[cfg(feature = "protobuf_serializer")]
#[cfg_attr(docsrs, doc(cfg(feature = "protobuf_serializer")))]
impl<M> Serializer<M> for ProtocolBuffers
where
    M: protobuf::Message,
{
    fn encode(message: &M) -> Result<(), EncodeError> {
        message
            .write_to_writer(&mut MessageRw {})
            .map_err(|err| err.into())
    }

    fn decode() -> Result<M, DecodeError> {
        M::parse_from_reader(&mut MessageRw {}).map_err(|err| err.into())
    }
}

/// A helper struct to read from and write to the message scratch buffer.
///
/// It simplifies streaming serialization/deserialization directly from the host and avoids copies.
/// Most serde based serializers can work directly with streaming serialization.
#[derive(Debug, Hash, Serialize, Deserialize)]
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
