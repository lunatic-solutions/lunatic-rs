mod serde_bincode;
mod serializer;

#[cfg(feature = "serde_json")]
mod serde_json;

#[cfg(feature = "serde_messagepack")]
mod serde_messagepack;

use crate::{mailbox::MessageRw, ReceiveError};

pub trait Msg: Sized {
    type Serializer: Serializer<Self>;

    fn write(&self) {
        Self::Serializer::serialize(self, &mut MessageRw {});
    }

    fn read() -> Result<Self, ReceiveError> {
        Self::Serializer::deserialize(&mut MessageRw {}).map_err(ReceiveError::SerializationError)
    }
}

pub use serializer::{DeserializeError, Serializer};

#[cfg(feature = "serde_json")]
pub use self::serde_json::Json;

#[cfg(feature = "serde_messagepack")]
pub use serde_messagepack::MessagePack;
