use super::{serializer::Serializer, DeserializeError};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};
pub struct Json<T: Serialize + DeserializeOwned>(pub T);

impl<T: Serialize + DeserializeOwned> Serializer<T> for Json<T> {
    fn serialize(data: &T, writer: &mut dyn Write) {
        serde_json::to_writer(writer, data).unwrap();
    }

    fn deserialize(reader: &mut dyn Read) -> Result<T, DeserializeError> {
        serde_json::from_reader(reader).map_err(|e| e.into())
    }
}

impl<T: Serialize + DeserializeOwned> Serializer<Self> for Json<T> {
    fn serialize(data: &Self, writer: &mut dyn Write) {
        serde_json::to_writer(writer, &data.0).unwrap();
    }

    fn deserialize(reader: &mut dyn Read) -> Result<Self, DeserializeError> {
        serde_json::from_reader(reader)
            .map_err(|e| e.into())
            .map(Json)
    }
}
