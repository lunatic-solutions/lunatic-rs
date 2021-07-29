use std::marker::PhantomData;

use crate::{
    host_api::{message, process},
    message::Message,
};

/// Mailbox for processes that are not linked, or linked and set to trap on notify signals.
pub struct Mailbox<T: Message> {
    _phantom: PhantomData<T>,
}

impl<T: Message> Mailbox<T> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData {},
        }
    }

    /// Gets next message from process' mailbox.
    ///
    /// If the mailbox is empty, this function will block until a new message arrives.
    pub fn receive(&self) -> T {
        let mut data_size = 0;
        let mut resource_size = 0;
        let message_type = unsafe {
            message::prepare_receive(
                &mut data_size as *mut usize,
                &mut resource_size as *mut usize,
            )
        };
        // Mailbox can't receive Signal messages, only
        assert_eq!(message_type, 0);

        let mut data = vec![0; data_size];
        let mut resources = vec![0; resource_size];

        unsafe { message::receive(data.as_mut_ptr(), resources.as_mut_ptr()) };
        let (_bytes_read, value) = T::from_bincode(&data, &resources);
        value
    }
}

impl<T: Message> TransformMailbox<T> for Mailbox<T> {
    fn catch_child_panic(self) -> LinkMailbox<T> {
        unsafe { process::die_when_link_dies(0) };
        LinkMailbox::new()
    }
    fn panic_if_child_panics(self) -> Mailbox<T> {
        self
    }
}

/// Mailbox for linked processes.
///
/// When a process is linked to others it will also receive messages if one of the others dies.
pub struct LinkMailbox<T: Message> {
    _phantom: PhantomData<T>,
}

impl<T: Message> LinkMailbox<T> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData {},
        }
    }

    /// Gets next message from process' mailbox.
    ///
    /// If the mailbox is empty, this function will block until a new message arrives.
    pub fn receive(&self) -> Result<T, Signal> {
        let mut data_size = 0;
        let mut resource_size = 0;
        let message_type = unsafe {
            message::prepare_receive(
                &mut data_size as *mut usize,
                &mut resource_size as *mut usize,
            )
        };
        if message_type == 1 {
            return Err(Signal {});
        }

        let mut data = vec![0; data_size];
        let mut resources = vec![0; resource_size];

        unsafe { message::receive(data.as_mut_ptr(), resources.as_mut_ptr()) };
        let (_bytes_read, value) = T::from_bincode(&data, &resources);
        Ok(value)
    }
}

impl<T: Message> TransformMailbox<T> for LinkMailbox<T> {
    fn catch_child_panic(self) -> LinkMailbox<T> {
        self
    }
    fn panic_if_child_panics(self) -> Mailbox<T> {
        unsafe { process::die_when_link_dies(1) };
        Mailbox::new()
    }
}

/// A Signal that was turned into a message.
pub struct Signal {}

pub trait TransformMailbox<T: Message> {
    fn catch_child_panic(self) -> LinkMailbox<T>;
    fn panic_if_child_panics(self) -> Mailbox<T>;
}
