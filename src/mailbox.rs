use std::{marker::PhantomData, time::Duration};

use thiserror::Error;

use crate::{
    function::process::{IntoProcess, NoLink},
    host::{self, api::message},
    serializer::{Bincode, DecodeError, Serializer},
    Process, ProcessConfig, Tag,
};

const LINK_TRAPPED: u32 = 1;
const TIMEOUT: u32 = 9027;

/// Mailbox of a [`Process`](crate::Process).
#[derive(Debug, Clone, Copy)]
pub struct Mailbox<M, S = Bincode>
where
    S: Serializer<M>,
{
    phantom: PhantomData<(M, S)>,
}

impl<M, S> Mailbox<M, S>
where
    S: Serializer<M>,
{
    /// Returns a reference to the currently running process
    pub fn this(&self) -> Process<M, S> {
        Process::new(host::node_id(), host::process_id())
    }

    /// Gets next message from process' mailbox.
    ///
    /// If the mailbox is empty, this function will block until a new message arrives.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be deserialized into `M`
    /// with serializer `S`.
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
    /// This function will panic if the received message can't be deserialized into `M`
    /// with serializer `S`.
    pub fn tag_receive(&self, tags: Option<&[Tag]>) -> M {
        match tags {
            Some(tags) => {
                let tags: Vec<i64> = tags.iter().map(|tag| tag.id()).collect();
                self.receive_(Some(&tags), None).unwrap()
            }
            None => self.receive_(None, None).unwrap(),
        }
    }

    /// Same as `receive`, but only waits for the duration of timeout for the message.
    pub fn receive_timeout(&self, timeout: Duration) -> Result<M, ReceiveError> {
        self.receive_(None, Some(timeout))
    }

    /// Same as `tag_receive`, but only waits for the duration of timeout for the message.
    pub fn tag_receive_timeout(
        &self,
        tags: Option<&[Tag]>,
        timeout: Duration,
    ) -> Result<M, ReceiveError> {
        match tags {
            Some(tags) => {
                let tags: Vec<i64> = tags.iter().map(|tag| tag.id()).collect();
                self.receive_(Some(&tags), Some(timeout))
            }
            None => self.receive_(None, Some(timeout)),
        }
    }

    fn receive_(&self, tags: Option<&[i64]>, timeout: Option<Duration>) -> Result<M, ReceiveError> {
        let tags = if let Some(tags) = tags { tags } else { &[] };
        let timeout_ms = match timeout {
            Some(timeout) => timeout.as_millis() as u64,
            None => u64::MAX,
        };
        let message_type = unsafe { message::receive(tags.as_ptr(), tags.len(), timeout_ms) };
        // Mailbox can't receive LINK_TRAPPED messages.
        assert_ne!(message_type, LINK_TRAPPED);
        // In case of timeout, return error.
        if message_type == TIMEOUT {
            return Err(ReceiveError::Timeout);
        }
        S::decode().map_err(|err| err.into())
    }

    /// Create a mailbox with a specific type.
    ///
    /// ### Safety
    ///
    /// It's not safe to mix different types of mailboxes inside one process. This function should
    /// never be used directly. The only reason it's public is that it's used inside the `main`
    /// macro and needs to be available outside this crate.
    pub unsafe fn new() -> Self {
        Self {
            phantom: PhantomData {},
        }
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

/// A special Mailbox that can catch if links trapped.
#[derive(Debug)]
pub(crate) struct LinkMailbox<M, S = Bincode>
where
    S: Serializer<M>,
{
    serializer_type: PhantomData<(M, S)>,
}

impl<M, S> LinkMailbox<M, S>
where
    S: Serializer<M>,
{
    /// Create a `LinkMailbox` with a specific type.
    ///
    /// ### Safety
    ///
    /// It's not safe to mix different types of mailboxes inside one process. This function should
    /// never be used directly.
    pub(crate) unsafe fn new() -> Self {
        Self {
            serializer_type: PhantomData {},
        }
    }

    /// Gets next message from process' mailbox that is tagged with one of the `tags`.
    ///
    /// If no such message exists, this function will block until a new message arrives.
    /// If `tags` is `None` it will take the first available message.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be deserialized into `M`.
    pub fn tag_receive(&self, tags: Option<&[Tag]>) -> Result<M, LinkTrapped> {
        match tags {
            Some(tags) => {
                let tags: Vec<i64> = tags.iter().map(|tag| tag.id()).collect();
                self.receive_(Some(&tags), None).unwrap()
            }
            None => self.receive_(None, None).unwrap(),
        }
    }

    fn receive_(
        &self,
        tags: Option<&[i64]>,
        timeout: Option<Duration>,
    ) -> Result<Result<M, LinkTrapped>, ReceiveError> {
        let tags = if let Some(tags) = tags { tags } else { &[] };
        let timeout_ms = match timeout {
            Some(timeout) => timeout.as_millis() as u64,
            None => u64::MAX,
        };
        let message_type = unsafe { message::receive(tags.as_ptr(), tags.len(), timeout_ms) };
        // If we received a LINK_TRAPPED message return
        if message_type == LINK_TRAPPED {
            return Ok(Err(LinkTrapped(Tag::from(unsafe { message::get_tag() }))));
        }
        // In case of timeout, return error.
        if message_type == TIMEOUT {
            return Err(ReceiveError::Timeout);
        }
        match S::decode() {
            Ok(message) => Ok(Ok(message)),
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Error, Debug)]
#[error("The link trapped")]
pub(crate) struct LinkTrapped(Tag);

impl LinkTrapped {
    pub(crate) fn tag(&self) -> Tag {
        self.0
    }
}

impl<M, S> NoLink for Mailbox<M, S> where S: Serializer<M> {}

impl<M, S> IntoProcess<M, S> for Mailbox<M, S>
where
    S: Serializer<M>,
{
    type Process = Process<M, S>;

    fn spawn<C>(
        capture: C,
        entry: fn(C, Self),
        link: Option<Tag>,
        config: Option<&ProcessConfig>,
        node: Option<u64>,
    ) -> Self::Process
    where
        S: Serializer<C> + Serializer<M>,
    {
        let entry = entry as usize as i32;
        let node_id = node.unwrap_or_else(host::node_id);

        // The `type_helper_wrapper` function is used here to create a pointer to a function with
        // generic types C, M & S. We can only send pointer data across processes and this is the
        // only way the Rust compiler will let us transfer this information into the new process.
        match host::spawn(node, config, link, type_helper_wrapper::<C, M, S>, entry) {
            Ok(id) => {
                // If the captured variable is of size 0, we don't need to send it to another process.
                if std::mem::size_of::<C>() == 0 {
                    Process::new(node_id, id)
                } else {
                    let child = Process::<C, S>::new(node_id, id);
                    child.send(capture);
                    // Processes can only receive one type of message, but to pass in the captured variable
                    // we pretend for the first message that our process is receiving messages of type `C`.
                    unsafe { std::mem::transmute(child) }
                }
            }
            Err(err) => panic!("Failed to spawn a process: {}", err),
        }
    }
}

// Wrapper function to help transfer the generic types C, M & S into the new process.
fn type_helper_wrapper<C, M, S>(function: i32)
where
    S: Serializer<C> + Serializer<M>,
{
    // If the captured variable is of size 0, don't wait on it.
    let captured = if std::mem::size_of::<C>() == 0 {
        unsafe { std::mem::MaybeUninit::<C>::zeroed().assume_init() }
    } else {
        unsafe { Mailbox::<C, S>::new() }.receive()
    };
    let mailbox = unsafe { Mailbox::new() };
    let function: fn(C, Mailbox<M, S>) = unsafe { std::mem::transmute(function) };
    function(captured, mailbox);
}

#[cfg(test)]
mod tests {
    use lunatic_test::test;
    use std::time::Duration;

    use super::*;
    use crate::{sleep, Mailbox};

    #[test]
    fn mailbox() {
        let child = Process::spawn(1, |capture, mailbox: Mailbox<i32>| {
            assert_eq!(capture, 1);
            assert_eq!(mailbox.receive(), 2);
        });

        child.send(2);
        sleep(Duration::from_millis(100));
    }

    #[test]
    #[should_panic]
    fn mailbox_link() {
        Process::spawn_link((), |_, _: Mailbox<()>| {
            panic!("fails");
        });

        // This process should fail before 100ms, because the link panics.
        sleep(Duration::from_millis(100));
    }
}
