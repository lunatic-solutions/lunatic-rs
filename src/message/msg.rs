use super::serializer::Serializer;
use crate::{mailbox::MessageRw, ReceiveError};

pub trait Msg: Sized {
    type Serializer: Serializer<Data = Self>;

    fn prepare_draft(&self) {
        Self::Serializer::serialize(self, &mut MessageRw {});
    }

    fn from_message_buffer() -> Result<Self, ReceiveError> {
        Self::Serializer::deserialize(&mut MessageRw {}).map_err(ReceiveError::SerializationError)
    }
}
