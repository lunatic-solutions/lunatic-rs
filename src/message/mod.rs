mod msg;
mod serializer;

#[cfg(feature = "serde_json")]
mod serde_json;

#[cfg(feature = "serde_messagepack")]
mod serde_messagepack;

mod bincode;

pub use msg::Msg;
pub use serializer::{DeserializeError, Serializer};

#[cfg(feature = "serde_json")]
pub use self::serde_json::Json;

#[cfg(feature = "serde_messagepack")]
pub use serde_messagepack::MessagePack;

// Testing types TODO remove

#[cfg(feature = "serde_messagepack")]
mod testing {
    use super::{MessagePack, Msg};

    #[derive(serde::Serialize, serde::Deserialize)]
    struct X {}

    impl Msg for X {
        type Serializer = MessagePack<Self>;
    }
}
