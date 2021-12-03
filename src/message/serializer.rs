use crate::Msg;
use std::io::{Read, Write};
use thiserror::Error;

pub trait Serializer<T> {
    fn serialize(data: &T, writer: &mut dyn Write);
    fn deserialize(reader: &mut dyn Read) -> Result<T, DeserializeError>;
}

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[cfg(feature = "serde_messagepack")]
    #[error("MessagePack error")]
    MessagePack(#[from] rmp_serde::decode::Error),

    #[error("Bincode error")]
    Bincode(#[from] bincode::Error),

    #[cfg(feature = "serde_json")]
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
}

impl<S: Serializer<Self>> Msg for S {
    type Serializer = S;
}
