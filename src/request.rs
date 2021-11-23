use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{mailbox::Msg, process::Process, tag::Tag};

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct Request<T, U>
where
    T: Msg,
    U: Msg,
{
    message: T,
    tag: Tag,
    sender_process: Process<U>,
}

impl<T, U> Request<T, U>
where
    T: Msg,
    U: Msg,
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
