use std::marker::PhantomData;

use super::{lifecycles, AbstractProcess, ProcessRef, StartupError};
use crate::function::process::{process_name, ProcessType};
use crate::{host, Mailbox, Process, ProcessConfig, Tag};

trait IntoAbstractProcessBuilder<T> {}

/// Holds additional information about [`AbstractProcess`] spawning.
///
/// This information can include data about the process configuration, what node
/// it should be spawned on, if the process should be linked and with which tag.
///
/// It implements the same public interface as [`AbstractProcess`], so that the
/// builder pattern can start with the [`AbstractProcess`] and transition to the
/// [`AbstractProcessBuilder`].
pub struct AbstractProcessBuilder<'a, T: ?Sized> {
    link: Option<Tag>,
    config: Option<&'a ProcessConfig>,
    node: Option<u64>,
    phantom: PhantomData<T>,
}

impl<'a, T> AbstractProcessBuilder<'a, T>
where
    T: AbstractProcess,
{
    pub(crate) fn new() -> AbstractProcessBuilder<'a, T> {
        AbstractProcessBuilder {
            link: None,
            config: None,
            node: None,
            phantom: PhantomData,
        }
    }

    /// Links the to be spawned process to the parent.
    pub fn link(self) -> AbstractProcessBuilder<'a, T> {
        AbstractProcessBuilder {
            link: Some(Tag::new()),
            config: self.config,
            node: self.node,
            phantom: PhantomData,
        }
    }

    /// Links the to be spawned process to the parent with a specific [`Tag`].
    pub fn link_with(self, tag: Tag) -> AbstractProcessBuilder<'a, T> {
        AbstractProcessBuilder {
            link: Some(tag),
            config: self.config,
            node: self.node,
            phantom: PhantomData,
        }
    }

    /// Allows for spawning the process with a specific configuration.
    pub fn configure(self, config: &'a ProcessConfig) -> AbstractProcessBuilder<'a, T> {
        AbstractProcessBuilder {
            link: self.link,
            config: Some(config),
            node: self.node,
            phantom: PhantomData,
        }
    }

    /// Sets the node on which the process will be spawned.
    pub fn on_node(self, node: u64) -> AbstractProcessBuilder<'a, T> {
        AbstractProcessBuilder {
            link: self.link,
            config: self.config,
            node: Some(node),
            phantom: PhantomData,
        }
    }

    /// Starts a new `AbstractProcess` and returns a reference to it.
    ///
    /// This call will block until the `init` function finishes. If the `init`
    /// function returns an error, it will be returned as
    /// `StartupError::Custom(error)`. If the `init` function panics during
    /// execution, it will return [`StartupError::InitPanicked`].
    #[track_caller]
    pub fn start(&self, arg: T::Arg) -> Result<ProcessRef<T>, StartupError<T>> {
        let init_tag = Tag::new();
        let process = self.start_without_wait_on_init(arg, init_tag);

        // Wait on `init()`
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
    /// This call will block until the `init` function finishes. If the `init`
    /// function returns an error, it will be returned as
    /// `StartupError::Custom(error)`. If the `init` function panics during
    /// execution, it will return [`StartupError::InitPanicked`].
    ///
    /// If used in combination with the [`on_node`](Self::on_node) option, the
    /// name registration will be performed on the local node and not the remote
    /// one.
    #[track_caller]
    pub fn start_as<S: AsRef<str>>(
        &self,
        name: S,
        arg: T::Arg,
    ) -> Result<ProcessRef<T>, StartupError<T>> {
        let name: &str = name.as_ref();
        let name = process_name::<T, T::Serializer>(ProcessType::ProcessRef, name);
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
                _ => {
                    let init_tag = Tag::new();
                    let process = self.start_without_wait_on_init(arg, init_tag);
                    // Register the name
                    host::api::registry::put(
                        name.as_ptr(),
                        name.len(),
                        process.node_id(),
                        process.id(),
                    );
                    // Wait on `init()`
                    let mailbox: Mailbox<Result<(), StartupError<T>>, T::Serializer> =
                        Mailbox::new();
                    match mailbox.tag_receive(&[init_tag]) {
                        Ok(()) => Ok(ProcessRef { process }),
                        Err(err) => {
                            // In case of an error during `init`, unregister the process.
                            host::api::registry::remove(name.as_ptr(), name.len());
                            Err(err)
                        }
                    }
                }
            }
        }
    }

    /// The startup code is the same for `start` and `start_as`, but can't wait
    /// on the result in both cases to avoid deadlocks with the registry.
    #[track_caller]
    fn start_without_wait_on_init(&self, arg: T::Arg, init_tag: Tag) -> Process<(), T::Serializer> {
        let this = unsafe { Process::<Result<(), StartupError<T>>, T::Serializer>::this() };
        let entry_data = (this, init_tag, arg);
        match (self.link, &self.config, self.node) {
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
        }
    }
}
