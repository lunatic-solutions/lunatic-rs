use crate::Msg;
use std::io::{Read, Write};
use thiserror::Error;

pub trait Serializer {
    type Data;
    fn serialize(data: &Self::Data, writer: &mut dyn Write);
    fn deserialize(reader: &mut dyn Read) -> Result<Self::Data, DeserializeError>;
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

impl<S: Serializer<Data = Self>> Msg for S {
    type Serializer = S;
}
