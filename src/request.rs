use crate::{
    host_api::{self},
    mailbox::Msg,
    process::Process,
    tag::Tag,
    ReceiveError,
};

pub struct Request<T, U>
where
    T: Msg,
    U: Msg,
{
    message: T,
    tag: Tag,
    sender_process: Process<U>,
}

impl<T, U> Msg for Request<T, U>
where
    T: Msg,
    U: Msg,
{
    fn prepare_draft(&self) {
        unsafe {
            host_api::message::create_data(self.tag.id(), 0);
            host_api::message::push_process(self.sender_process.id);
        };
        self.message.prepare_draft();
    }

    fn from_message_buffer() -> Result<Self, ReceiveError> {
        let sender_process = Process::from(unsafe { host_api::message::take_process(0) });
        let tag = Tag::from(unsafe { host_api::message::get_tag() });
        let message = T::from_message_buffer()?;
        Ok(Request {
            message,
            tag,
            sender_process,
        })
    }
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
