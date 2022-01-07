use std::{marker::PhantomData, time::Duration};

use thiserror::Error;

use crate::{
    host_api::message,
    serializer::{Bincode, DecodeError, Serializer},
    Tag,
};

const SIGNAL: u32 = 1;
const TIMEOUT: u32 = 9027;

/// Mailbox for processes that are not linked, or linked and set to trap on notify signals.
#[derive(Debug)]
pub struct Mailbox<M, S = Bincode>
where
    S: Serializer<M>,
{
    serializer_type: PhantomData<(M, S)>,
}

impl<M, S> Mailbox<M, S>
where
    S: Serializer<M>,
{
    /// Create a mailbox with a specific type.
    ///
    /// ### Safety
    ///
    /// It's not safe to mix different types of mailboxes inside one process. This function should
    /// never be used directly.
    pub unsafe fn new() -> Self {
        Self {
            serializer_type: PhantomData {},
        }
    }

    /// Gets next message from process' mailbox.
    ///
    /// If the mailbox is empty, this function will block until a new message arrives.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be deserialized into `M`.
    pub fn receive(&self) -> M {
        self.receive_(Some(&[1]), None).unwrap()
    }

    /// Gets next message from process' mailbox that is tagged with one of the `tags`.
    ///
    /// If no such message exists, this function will block until a new message arrives.
    /// If `tags` is `None` it will take the first available message.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be deserialized into `M`.
    pub(crate) fn tag_receive(&self, tags: Option<&[Tag]>) -> M {
        match tags {
            Some(tags) => {
                let tags: Vec<i64> = tags.iter().map(|tag| tag.id()).collect();
                self.receive_(Some(&tags), None).unwrap()
            }
            None => self.receive_(None, None).unwrap(),
        }
    }

    /// Same as [`receive`], but only waits for the duration of timeout for the message.
    pub fn receive_timeout(&self, timeout: Duration) -> Result<M, ReceiveError> {
        self.receive_(None, Some(timeout))
    }

    fn receive_(&self, tags: Option<&[i64]>, timeout: Option<Duration>) -> Result<M, ReceiveError> {
        let tags = if let Some(tags) = tags { tags } else { &[] };
        let timeout_ms = match timeout {
            // If waiting time is smaller than 1ms, round it up to 1ms.
            Some(timeout) => match timeout.as_millis() {
                0 => 1,
                other => other as u32,
            },
            None => 0,
        };
        let message_type = unsafe { message::receive(tags.as_ptr(), tags.len(), timeout_ms) };
        // Mailbox can't receive Signal messages.
        assert_ne!(message_type, SIGNAL);
        // In case of timeout, return error.
        if message_type == TIMEOUT {
            return Err(ReceiveError::Timeout);
        }
        S::decode().map_err(|err| err.into())
    }
}

/// Error while receiving a message.
#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("Deserialization failed")]
    DeserializationFailed(#[from] DecodeError),
    #[error("Timed out while waiting for message")]
    Timeout,
}
