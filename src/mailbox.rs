use std::{
    io::{Read, Write},
    marker::PhantomData,
    time::Duration,
};

use rmp_serde::decode;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

use crate::{
    host_api::{message, process},
    tag::Tag,
};

const SIGNAL: u32 = 1;
const TIMEOUT: u32 = 9027;

pub trait Msg: Sized {
    fn prepare_draft(&self);
    fn from_message_buffer() -> Result<Self, ReceiveError>;
}

pub trait MessagePackMsg {}

// TODO Just some quick impls to make tests pass. Think more about primitive types
// TODO we can implement a simpler serialization for primitive types with bytes representation
impl MessagePackMsg for u8 {}
impl MessagePackMsg for u16 {}
impl MessagePackMsg for i16 {}
impl MessagePackMsg for u32 {}
impl MessagePackMsg for i32 {}
impl MessagePackMsg for u64 {}
impl MessagePackMsg for i64 {}
impl MessagePackMsg for usize {}
impl MessagePackMsg for bool {}
impl MessagePackMsg for Vec<i32> {}
impl MessagePackMsg for String {}
impl<A: MessagePackMsg, B: MessagePackMsg> MessagePackMsg for (A, B) {}

impl Msg for () {
    fn prepare_draft(&self) {
        // do nothing
    }

    fn from_message_buffer() -> Result<Self, ReceiveError> {
        Ok(())
    }
}

impl<T: Serialize + DeserializeOwned + MessagePackMsg> Msg for T {
    fn prepare_draft(&self) {
        rmp_serde::encode::write(&mut MessageRw {}, &self).unwrap();
    }

    fn from_message_buffer() -> Result<Self, ReceiveError> {
        match rmp_serde::from_read(MessageRw {}) {
            Ok(result) => Ok(result),
            Err(decode_error) => Err(ReceiveError::DeserializationFailed(decode_error)),
        }
    }
}

/// Mailbox for processes that are not linked, or linked and set to trap on notify signals.
#[derive(Debug)]
pub struct Mailbox<T: Msg> {
    _phantom: PhantomData<T>,
}

impl<T: Msg> Mailbox<T> {
    /// Create a mailbox with a specific type.
    ///
    /// ### Safety
    ///
    /// It's not safe to mix different types of mailboxes inside one process. This function should
    /// never be used directly.
    pub unsafe fn new() -> Self {
        Self {
            _phantom: PhantomData {},
        }
    }

    /// Gets next message from process' mailbox.
    ///
    /// If the mailbox is empty, this function will block until a new message arrives.
    pub fn receive(&self) -> Result<T, ReceiveError> {
        self.receive_(None, None)
    }

    /// Same as [`receive`], but only waits for the duration of timeout for the message.
    pub fn receive_timeout(&self, timeout: Duration) -> Result<T, ReceiveError> {
        self.receive_(None, Some(timeout))
    }

    /// Gets next message from process' mailbox & its tag.
    ///
    /// If the mailbox is empty, this function will block until a new message arrives.
    pub fn receive_with_tag(&self) -> Result<(T, Tag), ReceiveError> {
        let message = self.receive_(None, None)?;
        let tag = unsafe { message::get_tag() };
        Ok((message, Tag::from(tag)))
    }

    /// Gets a message with a specific tag from the mailbox.
    ///
    /// If the mailbox is empty, this function will block until a new message arrives.
    pub fn tag_receive(&self, tags: &[Tag]) -> Result<T, ReceiveError> {
        let tags: Vec<i64> = tags.iter().map(|tag| tag.id()).collect();
        self.receive_(Some(&tags), None)
    }

    /// Same as [`tag_receive`], but only waits for the duration of timeout for the tagged message.
    pub fn tag_receive_timeout(&self, tags: &[Tag], timeout: Duration) -> Result<T, ReceiveError> {
        let tags: Vec<i64> = tags.iter().map(|tag| tag.id()).collect();
        self.receive_(Some(&tags), Some(timeout))
    }

    fn receive_(&self, tags: Option<&[i64]>, timeout: Option<Duration>) -> Result<T, ReceiveError> {
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

        T::from_message_buffer()
        //match rmp_serde::from_read(MessageRw {}) {
        //    Ok(result) => Ok(result),
        //    Err(decode_error) => Err(ReceiveError::DeserializationFailed(decode_error)),
        //}
    }
}

impl<T: Msg> TransformMailbox<T> for Mailbox<T> {
    fn catch_link_panic(self) -> LinkMailbox<T> {
        unsafe { process::die_when_link_dies(0) };
        LinkMailbox::new()
    }
    fn panic_if_link_panics(self) -> Mailbox<T> {
        self
    }
}

/// Mailbox for linked processes.
///
/// When a process is linked to others it will also receive messages if one of the others dies.
#[derive(Debug)]
pub struct LinkMailbox<T: Msg> {
    _phantom: PhantomData<T>,
}

impl<T: Msg> LinkMailbox<T> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData {},
        }
    }

    /// Gets next message from process' mailbox.
    ///
    /// If the mailbox is empty, this function will block until a new message arrives.
    pub fn receive(&self) -> Message<T> {
        self.receive_(None, None)
    }

    /// Same as [`receive`], but only waits for the duration of timeout for the message.
    pub fn receive_timeout(&self, timeout: Duration) -> Message<T> {
        self.receive_(None, Some(timeout))
    }

    /// Gets a message with a specific tag from the mailbox.
    ///
    /// If the mailbox is empty, this function will block until a new message arrives.
    pub fn tag_receive(&self, tags: &[Tag]) -> Message<T> {
        let tags: Vec<i64> = tags.iter().map(|tag| tag.id()).collect();
        self.receive_(Some(&tags), None)
    }

    /// Same as [`tag_receive`], but only waits for the duration of timeout for the tagged message.
    pub fn tag_receive_timeout(&self, tags: &[Tag], timeout: Duration) -> Message<T> {
        let tags: Vec<i64> = tags.iter().map(|tag| tag.id()).collect();
        self.receive_(Some(&tags), Some(timeout))
    }

    fn receive_(&self, tags: Option<&[i64]>, timeout: Option<Duration>) -> Message<T> {
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

        if message_type == SIGNAL {
            let tag = unsafe { message::get_tag() };
            return Message::Signal(Tag::from(tag));
        }
        // In case of timeout, return error.
        else if message_type == TIMEOUT {
            return Message::Normal(Err(ReceiveError::Timeout));
        }

        Message::Normal(T::from_message_buffer())
    }
}

impl<T: Msg> TransformMailbox<T> for LinkMailbox<T> {
    fn catch_link_panic(self) -> LinkMailbox<T> {
        self
    }
    fn panic_if_link_panics(self) -> Mailbox<T> {
        unsafe { process::die_when_link_dies(1) };
        unsafe { Mailbox::new() }
    }
}

/// Represents an error while receiving a message.
#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("Deserialization failed")]
    DeserializationFailed(#[from] decode::Error),
    #[error("Timed out while waiting for message")]
    Timeout,
}

/// Returned from [`LinkMailbox::receive`] to indicate if the received message was a signal or a
/// normal message.
#[derive(Debug)]
pub enum Message<T> {
    Normal(Result<T, ReceiveError>),
    Signal(Tag),
}

impl<T> Message<T> {
    /// Returns true if received message is a signal.
    pub fn is_signal(&self) -> bool {
        match self {
            Message::Normal(_) => false,
            Message::Signal(_) => true,
        }
    }

    /// Returns the message if it's a normal one or panics if not.
    pub fn normal_or_unwrap(self) -> Result<T, ReceiveError> {
        match self {
            Message::Normal(message) => message,
            Message::Signal(_) => panic!("Message is of type Signal"),
        }
    }
}

/// A Signal that was turned into a message.
#[derive(Debug, Clone, Copy)]
pub struct Signal {}

pub trait TransformMailbox<T: Msg> {
    fn catch_link_panic(self) -> LinkMailbox<T>;
    fn panic_if_link_panics(self) -> Mailbox<T>;
}

// A helper struct to read and write into the message scratch buffer.
pub(crate) struct MessageRw {}
impl Read for MessageRw {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(unsafe { message::read_data(buf.as_mut_ptr(), buf.len()) })
    }
}
impl Write for MessageRw {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(unsafe { message::write_data(buf.as_ptr(), buf.len()) })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
