use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{process::Process, tag::Tag};

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct Request<T, U>
where
    T: Serialize,
    U: Serialize + DeserializeOwned,
{
    message: T,
    tag: Tag,
    sender_process: Process<U>,
}

impl<T, U> Request<T, U>
where
    T: Serialize + DeserializeOwned,
    U: Serialize + DeserializeOwned,
{
    /// Create a new request
    pub(crate) fn new(message: T, tag: Tag, sender_process: Process<U>) -> Self {
        Self {
            message,
            tag,
            sender_process,
        }
    }

    /// Reply to a request.
    pub fn reply(self, message: U) {
        self.sender_process.tag_send(self.tag, message);
    }

    /// Get the message data from request.
    pub fn data(&self) -> &T {
        &self.message
    }

    /// Get the mutable message data from request.
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.message
    }

    /// Get a reference to the sender process.
    pub fn sender(&self) -> &Process<U> {
        &self.sender_process
    }
}
