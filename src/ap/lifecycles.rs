//! The [`AbstractProcess`] has well defined lifecycles, from startup to
//! termination. This file contains the implementation of each lifecycle.

use std::ptr::null;

use super::handlers::Handlers;
use super::messages::{ShutdownMessage, SHUTDOWN_HANDLER};
use super::tag::AbstractProcessTag;
use super::{AbstractProcess, Config, StartupError};
use crate::mailbox::LINK_DIED;
use crate::panic::{catch_panic, Panicked};
use crate::serializer::CanSerialize;
use crate::{host, Mailbox, Process, Tag};

type ParentProcessRef<AP> =
    Process<Result<(), StartupError<AP>>, <AP as AbstractProcess>::Serializer>;

/// This is the entry point into the [`AbstractProcess`].
///
/// The entry point will get a reference to the parent, so that it can notify it
/// when the initialization finishes. A tag is used so that the parent can wait
/// for the right "`init` finished" message. It will also get the arguments for
/// the `init` function.
///
/// After the initialization finishes, it will spin in a loop waiting for
/// commands, until the `Shutdown` command is received.
pub(crate) fn entry<AP: AbstractProcess>(
    (parent, init_tag, arg): (ParentProcessRef<AP>, Tag, AP::Arg),
    _: Mailbox<(), AP::Serializer>, // Can't be used for the `AbstractProcess` special case.
) where
    AP::Serializer: CanSerialize<()>,
    AP::Serializer: CanSerialize<ShutdownMessage<AP::Serializer>>,
{
    // Catch errors during startup and notify parent. Panics will also be caught.
    let mut state = match startup::<AP>(arg) {
        Ok(state) => {
            // Notify spawner that startup succeeded & continue.
            parent.tag_send(init_tag, Ok(()));
            state
        }
        Err(err) => {
            // Notify spawner that startup failed with the reason why it failed.
            parent.tag_send(init_tag, Err(err));
            return;
        }
    };

    let shutdown_tag = loop_and_handle::<AP>(&mut state);
    shutdown::<AP>(shutdown_tag, state);
}

/// This code is executed during the [`AbstractProcess::start`] call.
fn startup<AP: AbstractProcess>(arg: AP::Arg) -> Result<AP::State, StartupError<AP>> {
    let config = Config::new();
    match catch_panic(|| AP::init(config, arg)) {
        Ok(Ok(state)) => Ok(state),
        Ok(Err(custom)) => Err(StartupError::Custom(custom)),
        Err(Panicked) => Err(StartupError::InitPanicked),
    }
}

/// Extracts the handler out of the tag for each incoming message, until
/// shutdown message is received.
fn loop_and_handle<AP: AbstractProcess>(state: &mut AP::State) -> Tag {
    loop {
        // Wait for next message & handle link died if result matches constant.
        if unsafe { host::api::message::receive(null(), 0, u64::MAX) } == LINK_DIED {
            let tag = unsafe { host::api::message::get_tag() };
            let tag = Tag::from(tag);
            AP::handle_link_death(super::State { state }, tag);
            continue;
        }

        // Extract `data` from tag
        let tag = unsafe { host::api::message::get_tag() };
        let tag = Tag::from(tag);
        let (response_tag, data) = AbstractProcessTag::extract_u6_data(tag);

        // Check if `data` matches the shutdown message
        if data == SHUTDOWN_HANDLER {
            break response_tag;
        }

        // Use `data` to look up the right handler function
        AP::Handlers::handle(response_tag, data, state);
    }
}

/// Is executed if the [`AbstractProcess`] receives a `shutdown` command.
fn shutdown<AP>(shutdown_tag: Tag, state: AP::State)
where
    AP: AbstractProcess,
    AP::Serializer: CanSerialize<()>,
    AP::Serializer: CanSerialize<ShutdownMessage<AP::Serializer>>,
{
    // The shutdown message needs to deserialize before `terminate` is called.
    // After `terminate` we could have another message in the buffer.
    let shutdown_message: ShutdownMessage<AP::Serializer> = AP::Serializer::decode().unwrap();
    AP::terminate(state);
    shutdown_message.0.send_response((), shutdown_tag);
}
