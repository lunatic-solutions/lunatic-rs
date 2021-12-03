use super::{serializer::Serializer, DeserializeError};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};
pub struct MessagePack<T: Serialize + DeserializeOwned>(pub T);

impl<T: Serialize + DeserializeOwned> Serializer<T> for MessagePack<T> {
    fn serialize(data: &T, writer: &mut dyn Write) {
        rmp_serde::encode::write(writer, data).unwrap();
    }

    fn deserialize(reader: &mut dyn Read) -> Result<T, DeserializeError> {
        rmp_serde::decode::from_read(reader).map_err(|e| e.into())
    }
}

impl<T: Serialize + DeserializeOwned> Serializer<Self> for MessagePack<T> {
    fn serialize(data: &Self, writer: &mut dyn Write) {
        rmp_serde::encode::write(writer, &data.0).unwrap();
    }

    fn deserialize(reader: &mut dyn Read) -> Result<Self, DeserializeError> {
        rmp_serde::decode::from_read(reader)
            .map_err(|e| e.into())
            .map(MessagePack)
    }
}
