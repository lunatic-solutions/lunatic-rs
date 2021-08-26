use std::{
    io::{Read, Write},
    marker::PhantomData,
    time::Duration,
};

use rmp_serde::decode;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    host_api::{message, process},
    tag::Tag,
};

/// Mailbox for processes that are not linked, or linked and set to trap on notify signals.
#[derive(Debug)]
pub struct Mailbox<T: Serialize + DeserializeOwned> {
    _phantom: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> Mailbox<T> {
    /// Create a mailbox with a specific type.
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
    pub fn receive(&self) -> Result<T, decode::Error> {
        self.receive_(None, None)
    }

    fn receive_(&self, tag: Option<i64>, timeout: Option<Duration>) -> Result<T, decode::Error> {
        let tag = match tag {
            Some(tag) => tag,
            None => 0,
        };
        let timeout_ms = match timeout {
            Some(timeout) => timeout.as_millis() as u32,
            None => 0,
        };
        let message_type = unsafe { message::receive(tag, timeout_ms) };
        // Mailbox can't receive Signal messages.
        assert_eq!(message_type, 0);
        rmp_serde::from_read(MessageRw {})
    }
}

impl<T: Serialize + DeserializeOwned> TransformMailbox<T> for Mailbox<T> {
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
pub struct LinkMailbox<T: Serialize + DeserializeOwned> {
    _phantom: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> LinkMailbox<T> {
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

    fn receive_(&self, tag: Option<i64>, timeout: Option<Duration>) -> Message<T> {
        let tag = match tag {
            Some(tag) => tag,
            None => 0,
        };
        let timeout_ms = match timeout {
            Some(timeout) => timeout.as_millis() as u32,
            None => 0,
        };
        let message_type = unsafe { message::receive(tag, timeout_ms) };

        if message_type == 1 {
            let tag = unsafe { message::get_tag() };
            return Message::Signal(Tag(tag));
        }

        Message::Normal(rmp_serde::from_read(MessageRw {}))
    }
}

impl<T: Serialize + DeserializeOwned> TransformMailbox<T> for LinkMailbox<T> {
    fn catch_link_panic(self) -> LinkMailbox<T> {
        self
    }
    fn panic_if_link_panics(self) -> Mailbox<T> {
        unsafe { process::die_when_link_dies(1) };
        unsafe { Mailbox::new() }
    }
}

/// Returned from [`LinkMailbox::receive`] to indicate if the received message was a signal or a
/// normal message.
#[derive(Debug)]
pub enum Message<T> {
    Normal(Result<T, decode::Error>),
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
    pub fn normal_or_unwrap(self) -> Result<T, decode::Error> {
        match self {
            Message::Normal(message) => message,
            Message::Signal(_) => panic!("Message is of type Signal"),
        }
    }
}

/// A Signal that was turned into a message.
#[derive(Debug, Clone, Copy)]
pub struct Signal {}

pub trait TransformMailbox<T: Serialize + DeserializeOwned> {
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
