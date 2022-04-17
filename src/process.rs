use std::{cell::UnsafeCell, marker::PhantomData};

use crate::{
    host,
    serializer::{Bincode, Serializer},
    Mailbox, ProcessConfig, Resource, Tag,
};

/// A process that can receive messages through a [`Mailbox`].
///
/// The generic type `M` defines the type of messages that can be sent to it and the type `S`
/// defines the serializer that will be used to de/serialize the messages. By default the
/// [`Bincode`] serializer is used.
///
/// A `Process` is spawned using the [`spawn`](crate::spawn) function. When spawned, the process
/// can capture some context from the parent. It will be provided to it through the
/// first argument of the entry function. The second argument is going to be the [`Mailbox`].
///
/// If the closure attempts to implicitly capture any variables from the outer context the code
/// will fail to compile. Processes don't share any memory and everything needs to be shared
/// through messages. This limits the capturing process to only types that can be de/serialized
/// with the serializer `S`.
///
/// A message can be sent to the `Process` with the [`send`](Process::send) method.
///
/// # Example
///
/// ```
/// let child = Process::spawn(1, |capture, mailbox: Mailbox<i32>| {
///    assert_eq!(capture, 1);
///    assert_eq!(mailbox.receive(), 2);
/// });
///
/// child.send(2);
/// ```
pub struct Process<M, S = Bincode>
where
    S: Serializer<M>,
{
    id: u64,
    // If set to true, the host call `lunatic::process::drop_process` will not be executed on drop.
    consumed: UnsafeCell<bool>,
    serializer_type: PhantomData<(M, S)>,
}

impl<M, S> Process<M, S>
where
    S: Serializer<M>,
{
    pub fn spawn<C>(capture: C, entry: fn(C, Mailbox<M, S>)) -> Process<M, S>
    where
        S: Serializer<C> + Serializer<M>,
    {
        Self::spawn_(capture, entry, None, None)
    }

    pub fn spawn_link<C>(capture: C, entry: fn(C, Mailbox<M, S>)) -> Process<M, S>
    where
        S: Serializer<C> + Serializer<M>,
    {
        Self::spawn_(capture, entry, Some(Tag::new()), None)
    }

    /// Spawns a linked process.
    ///
    /// Allows the caller to provide a tag for the link.
    pub fn spawn_link_tag<C>(capture: C, tag: Tag, entry: fn(C, Mailbox<M, S>)) -> Process<M, S>
    where
        S: Serializer<C> + Serializer<M>,
    {
        Self::spawn_(capture, entry, Some(tag), None)
    }

    pub fn spawn_config<C>(
        config: &ProcessConfig,
        capture: C,
        entry: fn(C, Mailbox<M, S>),
    ) -> Process<M, S>
    where
        S: Serializer<C> + Serializer<M>,
    {
        Self::spawn_(capture, entry, None, Some(config))
    }

    pub fn spawn_link_config<C>(
        config: &ProcessConfig,
        capture: C,
        entry: fn(C, Mailbox<M, S>),
    ) -> Process<M, S>
    where
        S: Serializer<C> + Serializer<M>,
    {
        Self::spawn_(capture, entry, Some(Tag::new()), Some(config))
    }

    pub fn spawn_link_config_tag<C>(
        config: &ProcessConfig,
        capture: C,
        tag: Tag,
        entry: fn(C, Mailbox<M, S>),
    ) -> Process<M, S>
    where
        S: Serializer<C> + Serializer<M>,
    {
        Self::spawn_(capture, entry, Some(tag), Some(config))
    }

    fn spawn_<C>(
        capture: C,
        entry: fn(C, Mailbox<M, S>),
        link: Option<Tag>,
        config: Option<&ProcessConfig>,
    ) -> Process<M, S>
    where
        S: Serializer<C> + Serializer<M>,
    {
        let entry = entry as usize as i32;

        // The `type_helper_wrapper` function is used here to create a pointer to a function with
        // generic types C, M & S. We can only send pointer data across porcesses and this is the
        // only way the Rust compiler will let us transfer this information into the new process.
        match host::spawn(config, link, type_helper_wrapper::<C, M, S>, entry) {
            Ok(id) => {
                // If the captured variable is of size 0, we don't need to send it to another process.
                if std::mem::size_of::<C>() == 0 {
                    Process {
                        id,
                        consumed: UnsafeCell::new(false),
                        serializer_type: PhantomData,
                    }
                } else {
                    let child = Process::<C, S> {
                        id,
                        consumed: UnsafeCell::new(false),
                        serializer_type: PhantomData,
                    };
                    child.send(capture);
                    // Processes can only receive one type of message, but to pass in the captured variable
                    // we pretend for the first message that our process is receiving messages of type `C`.
                    unsafe { std::mem::transmute(child) }
                }
            }
            Err(err) => panic!("Failed to spawn a process: {}", err),
        }
    }

    /// Returns a globally unique process ID.
    pub fn uuid(&self) -> u128 {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host::api::process::id(self.id, &mut uuid as *mut [u8; 16]) };
        u128::from_le_bytes(uuid)
    }

    /// Send a message to the process.
    pub fn send(&self, message: M) {
        // Create new message.
        unsafe { host::api::message::create_data(1, 0) };
        // During serialization resources will add themself to the message.
        S::encode(&message).unwrap();
        // Send it!
        unsafe { host::api::message::send(self.id) };
    }

    /// Send message to process with a specific tag.
    pub fn tag_send(&self, tag: Tag, message: M) {
        // Create new message.
        unsafe { host::api::message::create_data(tag.id(), 0) };
        // During serialization resources will add themself to the message.
        S::encode(&message).unwrap();
        // Send it!
        unsafe { host::api::message::send(self.id) };
    }

    /// Link process to the one currently running.
    pub fn link(&self) {
        // Don't use tags because a process' [`Mailbox`] can't differentiate between regular
        // messages and signals. Linked processes will almost always die when a link is broken.
        unsafe { host::api::process::link(0, self.id) };
    }

    /// Unlink processes from the caller.
    pub fn unlink(&self) {
        unsafe { host::api::process::unlink(self.id) };
    }

    /// Marks the process as consumed.
    ///
    /// Consumed processes don't call the `lunatic::process::drop_process` host function when they
    /// are dropped. This characteristic is useful when implementing serializers for processes.
    /// Serializers will move the process out of the local state into the message scratch buffer
    /// and they can't be dropped from the local state anymore.
    unsafe fn consume(&self) {
        *self.consumed.get() = true;
    }
}

// Wrapper functions to help transfer the generic types C, M & S into the new process.
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

impl<M, S> Resource for Process<M, S>
where
    S: Serializer<M>,
{
    fn id(&self) -> u64 {
        self.id
    }

    unsafe fn from_id(id: u64) -> Self {
        Self {
            id,

            consumed: UnsafeCell::new(false),
            serializer_type: PhantomData,
        }
    }
}

// Processes are equal if their UUID is equal.
impl<M, S> PartialEq for Process<M, S>
where
    S: Serializer<M>,
{
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

impl<M, S> std::fmt::Debug for Process<M, S>
where
    S: Serializer<M>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("uuid", &self.uuid())
            .finish()
    }
}

impl<M, S> Clone for Process<M, S>
where
    S: Serializer<M>,
{
    fn clone(&self) -> Self {
        let id = unsafe { host::api::process::clone_process(self.id) };
        unsafe { Process::from_id(id) }
    }
}

impl<M, S> Drop for Process<M, S>
where
    S: Serializer<M>,
{
    fn drop(&mut self) {
        // Only drop a process if it's not already consumed.
        if unsafe { !*self.consumed.get() } {
            unsafe { host::api::process::drop_process(self.id) };
        }
    }
}

impl<M, S> serde::Serialize for Process<M, S>
where
    S: Serializer<M>,
{
    fn serialize<A>(&self, serializer: A) -> Result<A::Ok, A::Error>
    where
        A: serde::Serializer,
    {
        // Mark process as consumed.
        unsafe { self.consume() };

        let index = unsafe { host::api::message::push_process(self.id) };
        serializer.serialize_u64(index)
    }
}

struct ProcessVisitor<M, S> {
    _phantom: PhantomData<(M, S)>,
}

impl<'de, M, S> serde::de::Visitor<'de> for ProcessVisitor<M, S>
where
    S: Serializer<M>,
{
    type Value = Process<M, S>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an u64 index")
    }

    fn visit_u64<E>(self, index: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let id = unsafe { host::api::message::take_process(index) };
        Ok(unsafe { Process::from_id(id) })
    }
}

impl<'de, M, S> serde::de::Deserialize<'de> for Process<M, S>
where
    S: Serializer<M>,
{
    fn deserialize<D>(deserializer: D) -> Result<Process<M, S>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_u64(ProcessVisitor {
            _phantom: PhantomData {},
        })
    }
}

#[cfg(test)]
mod tests {
    use lunatic_test::test;
    use std::time::Duration;

    use super::*;
    use crate::sleep;

    #[test]
    fn spawn() {
        let child = Process::spawn(1, |capture, mailbox: Mailbox<i32>| {
            assert_eq!(capture, 1);
            assert_eq!(mailbox.receive(), 2);
        });

        child.send(2);
        sleep(Duration::from_millis(100));
    }

    #[test]
    #[should_panic]
    fn spawn_link() {
        Process::<()>::spawn_link((), |_, _| {
            panic!("fails");
        });

        // This process should fail before 100ms, because the link panics.
        sleep(Duration::from_millis(100));
    }
}
