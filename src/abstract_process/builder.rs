use std::marker::PhantomData;

use super::messages::ShutdownMessage;
use super::{lifecycles, AbstractProcess, ProcessRef, StartupError};
use crate::protocol::ProtocolCapture;
use crate::serializer::Serializer;
use crate::{host, Mailbox, Process, ProcessConfig, Tag};

trait IntoAbstractProcessBuilder<T> {}

/// Holds additional information about [`AbstractProcess`] spawning.
///
/// This information can include data about the process configuration, what node
/// it should be spawned on, if the process should be linked and with which tag.
///
/// It implements the same public interface as [`AbstractProcess`], so that the
/// builder pattern can simply start with the [`AbstractProcess`] and transition
/// to the [`AbstractProcessBuilder`].
pub struct AbstractProcessBuilder<T: ?Sized> {
    link: Option<Tag>,
    config: Option<ProcessConfig>,
    node: Option<u64>,
    phantom: PhantomData<T>,
}

impl<T> AbstractProcessBuilder<T>
where
    T: AbstractProcess,
{
    pub(crate) fn new() -> AbstractProcessBuilder<T> {
        AbstractProcessBuilder {
            link: None,
            config: None,
            node: None,
            phantom: PhantomData,
        }
    }

    pub fn link(self) -> AbstractProcessBuilder<T> {
        AbstractProcessBuilder {
            link: Some(Tag::new()),
            config: self.config,
            node: self.node,
            phantom: PhantomData,
        }
    }

    pub fn link_with(self, tag: Tag) -> AbstractProcessBuilder<T> {
        AbstractProcessBuilder {
            link: Some(tag),
            config: self.config,
            node: self.node,
            phantom: PhantomData,
        }
    }

    pub fn configure(self, config: ProcessConfig) -> AbstractProcessBuilder<T> {
        AbstractProcessBuilder {
            link: self.link,
            config: Some(config),
            node: self.node,
            phantom: PhantomData,
        }
    }

    pub fn on_node(self, node: u64) -> AbstractProcessBuilder<T> {
        AbstractProcessBuilder {
            link: self.link,
            config: self.config,
            node: Some(node),
            phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn start(&self, arg: T::Arg) -> Result<ProcessRef<T>, StartupError<T>>
    where
        T::Serializer: Serializer<()>,
        T::Serializer: Serializer<ShutdownMessage<(), T::Serializer>>,
        T::Serializer: Serializer<(
            Process<Result<(), StartupError<T>>, T::Serializer>,
            Tag,
            T::Arg,
        )>,
        // TODO: Remove this constraints once Processes/Protocols are refactored.
        T::Serializer: Serializer<ProtocolCapture<T::Arg>>,
        T::Serializer: Serializer<
            ProtocolCapture<(
                Process<Result<(), StartupError<T>>, T::Serializer>,
                Tag,
                T::Arg,
            )>,
        >,
        T::Serializer: Serializer<ProtocolCapture<ProtocolCapture<T::Arg>>>,
    {
        let this = Process::<Result<(), StartupError<T>>, T::Serializer>::this();
        let init_tag = Tag::new();
        let entry_data = (this, init_tag, arg);
        let process = match (self.link, &self.config, self.node) {
            (Some(_), _, Some(_node)) => {
                unimplemented!("Linking across nodes is not supported yet");
            }
            (Some(tag), Some(config), None) => Process::<(), T::Serializer>::spawn_link_config_tag(
                config,
                entry_data,
                tag,
                lifecycles::entry::<T>,
            ),
            (Some(tag), None, None) => Process::<(), T::Serializer>::spawn_link_tag(
                entry_data,
                tag,
                lifecycles::entry::<T>,
            ),
            (None, Some(config), Some(node)) => Process::<(), T::Serializer>::spawn_node_config(
                node,
                config,
                entry_data,
                lifecycles::entry::<T>,
            ),
            (None, None, Some(node)) => {
                Process::<(), T::Serializer>::spawn_node(node, entry_data, lifecycles::entry::<T>)
            }
            (None, Some(config), None) => Process::<(), T::Serializer>::spawn_config(
                config,
                entry_data,
                lifecycles::entry::<T>,
            ),
            (None, None, None) => {
                Process::<(), T::Serializer>::spawn(entry_data, lifecycles::entry::<T>)
            }
        };

        // Don't return until `init()` finishes
        let mailbox: Mailbox<Result<(), StartupError<T>>, T::Serializer> =
            unsafe { Mailbox::new() };
        match mailbox.tag_receive(&[init_tag]) {
            Ok(()) => Ok(ProcessRef { process }),
            Err(err) => Err(err),
        }
    }

    /// Starts the process and registers it under `name`. If another process is
    /// already registered under the same name, it will return a
    /// `Err(StartupError::NameAlreadyRegistered(proc))` with a reference to the
    /// existing process.
    ///
    /// If used in combination with the [`on_node`](Self::on_node) option, the
    /// name registration will be performed on the local and not the remote
    /// node.
    #[track_caller]
    pub fn start_as<S: AsRef<str>>(
        &self,
        name: S,
        arg: T::Arg,
    ) -> Result<ProcessRef<T>, StartupError<T>>
    where
        T::Serializer: Serializer<()>,
        T::Serializer: Serializer<ShutdownMessage<(), T::Serializer>>,
        T::Serializer: Serializer<(
            Process<Result<(), StartupError<T>>, T::Serializer>,
            Tag,
            T::Arg,
        )>,
        // TODO: Remove this constraints once Processes/Protocols are refactored.
        T::Serializer: Serializer<ProtocolCapture<T::Arg>>,
        T::Serializer: Serializer<
            ProtocolCapture<(
                Process<Result<(), StartupError<T>>, T::Serializer>,
                Tag,
                T::Arg,
            )>,
        >,
        T::Serializer: Serializer<ProtocolCapture<ProtocolCapture<T::Arg>>>,
    {
        let name: &str = name.as_ref();
        let name = format!("{} + ProcessRef + {}", name, std::any::type_name::<T>());
        let mut node_id: u64 = 0;
        let mut process_id: u64 = 0;
        unsafe {
            match host::api::registry::get_or_put_later(
                name.as_ptr(),
                name.len(),
                &mut node_id,
                &mut process_id,
            ) {
                0 => Err(StartupError::NameAlreadyRegistered(ProcessRef::new(
                    node_id, process_id,
                ))),
                _ => match self.start(arg) {
                    Ok(proc) => {
                        host::api::registry::put(
                            name.as_ptr(),
                            name.len(),
                            proc.node_id(),
                            proc.id(),
                        );
                        Ok(proc)
                    }
                    err => {
                        // In case of error, we also need to call `put` or the
                        // previous call to `get_or_put_later` will forever
                        // lock the registry.
                        host::api::registry::put(name.as_ptr(), name.len(), 0, 0);
                        // Then we immediately remove the registration
                        host::api::registry::remove(name.as_ptr(), name.len());
                        err
                    }
                },
            }
        }
    }
}
