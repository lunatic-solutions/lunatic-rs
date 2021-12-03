use crate::{
    host_api::{self},
    message::{DeserializeError, Msg, Serializer},
    process::Process,
    tag::Tag,
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

impl<T, U> Serializer for Request<T, U>
where
    T: Msg,
    U: Msg,
{
    type Data = Self;

    fn serialize(data: &Self::Data, _writer: &mut dyn std::io::Write) {
        unsafe {
            *data.sender_process.consumed.get() = true;
            host_api::message::push_process(data.sender_process.id);
        };
        data.message.prepare_draft();
    }

    fn deserialize(reader: &mut dyn std::io::Read) -> Result<Self::Data, DeserializeError> {
        let sender_process = Process::from(unsafe { host_api::message::take_process(0) });
        let tag = Tag::from(unsafe { host_api::message::get_tag() });
        let message = T::Serializer::deserialize(reader)?;
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
