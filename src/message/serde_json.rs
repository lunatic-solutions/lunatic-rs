use super::{serializer::Serializer, DeserializeError};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};
pub struct Json<T: Serialize + DeserializeOwned>(T);

impl<T: Serialize + DeserializeOwned> Serializer for Json<T> {
    type Data = T;

    fn serialize(data: &Self::Data, writer: &mut dyn Write) {
        serde_json::to_writer(writer, data).unwrap();
    }

    fn deserialize(reader: &mut dyn Read) -> Result<Self::Data, DeserializeError> {
        serde_json::from_reader(reader).map_err(|e| e.into())
    }
}
