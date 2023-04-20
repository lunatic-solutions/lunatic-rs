use crate::serializer::CanSerialize;
use crate::{Process, Tag};

/// Contains information about the request sender, so that a response can be
/// sent back to the correct process.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub(crate) struct ReturnAddress<Response, Serializer> {
    process: Process<Response, Serializer>,
}

impl<Response, Serializer> ReturnAddress<Response, Serializer>
where
    Serializer: CanSerialize<Response>,
{
    pub(crate) fn from_self() -> Self {
        let process = unsafe { Process::this() };
        ReturnAddress { process }
    }

    /// Sends response back to a process.
    ///
    /// The tag should be provided by the sender and should be extracted from
    /// the incoming message.
    pub(crate) fn send_response(self, response: Response, tag: Tag) {
        self.process.tag_send(tag, response);
    }
}

/// Value identifying the shutdown handler.
///
/// All other handlers have a value from 0-16.
pub(crate) const SHUTDOWN_HANDLER: u8 = 32;

/// An incoming message indicating a shutdown for the [`AbstractProcess`].
///
/// The message combined with the `SHUTDOWN_HANDLER` data inside the tag.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub struct ShutdownMessage<Serializer>(pub(crate) ReturnAddress<(), Serializer>);

/// An incoming message indicating a request for the [`AbstractProcess`].
#[derive(serde::Serialize, serde::Deserialize)]
pub struct RequestMessage<T, Response, Serializer>(
    pub(crate) T,
    #[serde(bound = "")] pub(crate) ReturnAddress<Response, Serializer>,
);
