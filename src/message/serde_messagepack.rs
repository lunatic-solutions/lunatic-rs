use super::{serializer::Serializer, DeserializeError};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};
pub struct MessagePack<T: Serialize + DeserializeOwned>(pub T);

impl<T: Serialize + DeserializeOwned> Serializer for MessagePack<T> {
    type Data = T;

    fn serialize(data: &Self::Data, writer: &mut dyn Write) {
        rmp_serde::encode::write(writer, data).unwrap();
    }

    fn deserialize(reader: &mut dyn Read) -> Result<Self::Data, DeserializeError> {
        rmp_serde::decode::from_read(reader).map_err(|e| e.into())
    }
}
