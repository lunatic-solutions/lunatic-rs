use std::{cell::UnsafeCell, marker::PhantomData};
use thiserror::Error;

use super::{
    gen_server::{Message, Request},
    IntoProcess, IntoProcessLink, Process,
};
use crate::{
    host_api,
    mailbox::LinkMailbox,
    module::{params_to_vec, Param, WasmModule},
    process::gen_server::Sendable,
    serializer::{Bincode, Serializer},
    LunaticError, Mailbox, ProcessConfig, Resource, Tag,
};

pub trait HandleSupervisorMessage<M, S = Bincode>
where
    S: Serializer<M>,
{
    fn handle(&mut self, message: M, children: &mut Children);
}

pub trait HandleSupervisorRequest<M, S = Bincode>
where
    S: Serializer<M>,
{
    type Result;

    fn handle(&mut self, request: M, children: &mut Children) -> Self::Result;
}

/// Represents a supervised process.
///
/// To get a reference to the currently running instance of the process use the `process` method.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Supervised<T: Resource> {
    index: usize,
    temp: Option<T>,
    _phantom: PhantomData<T>,
}

#[derive(Error, Debug)]
#[error("The process is not spawned by the supervisor yet.")]
pub struct ProcessNotRunningYet;

impl<T: Resource> Supervised<T> {
    pub fn process(&mut self, children: &mut Children) -> Result<&T, ProcessNotRunningYet> {
        match children.children.get(self.index) {
            Some(child) => match child.pid {
                Some(pid) => {
                    let proc = unsafe { T::from_id(pid) };
                    // The old process is going be dropped by the `Children` struct holding it.
                    if let Some(old_proc) = self.temp.replace(proc) {
                        std::mem::forget(old_proc)
                    }
                    Ok(self.temp.as_ref().unwrap())
                }
                None => Err(ProcessNotRunningYet),
            },
            None => unreachable!("`Supervised<T>` can't exist without a matching index."),
        }
    }
}

struct Child {
    tag: Tag,
    pid: Option<u64>,
    re_spawner: Box<dyn Fn() -> Result<u64, LunaticError>>,
}

impl Child {
    fn spawn(&mut self) {
        let pid = (self.re_spawner)().unwrap();
        self.pid = Some(pid);
    }
}

/// A collection of processes that are supervised.
pub struct Children {
    children: Vec<Child>,
}

impl Children {
    fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    fn spawn(&mut self) {
        for child in self.children.iter_mut() {
            child.spawn();
        }
    }

    fn re_spawn(&mut self, tag: Tag) {
        let child = self.children.iter_mut().find(|child| child.tag == tag);
        match child {
            // Spawn child again
            Some(child) => child.spawn(),
            None => panic!("Supervisor received unknown link trapped tag"),
        }
    }

    /// Add child process definition to the supervisor.
    pub fn supervise<T, C>(&mut self, capture: C, handler: T::Handler) -> Supervised<T>
    where
        T: IntoProcessLink<C> + Resource,
        T::Handler: Clone + 'static,
        C: Clone + 'static,
    {
        let tag = Tag::new();
        let re_spawner = move || {
            let result = <T as IntoProcessLink<C>>::spawn_link(
                None,
                None,
                tag,
                capture.clone(),
                handler.clone(),
            );
            match result {
                Ok(process) => {
                    let pid = process.id();
                    // Don't drop the process.
                    std::mem::forget(process);
                    Ok(pid)
                }
                Err(err) => Err(err),
            }
        };

        let child = Child {
            tag,
            pid: None,
            re_spawner: Box::new(re_spawner),
        };
        self.children.push(child);
        let index = self.children.len() - 1;
        Supervised {
            index,
            temp: None,
            _phantom: PhantomData,
        }
    }
}

pub trait Supervise {
    /// Spawn and initialize children.
    fn init(&mut self, children: &mut Children);
}

/// A `Supervisor` is a [`GenericServer`](crate::GenericServer) that can supervise other processes.
///
/// Like the `GenericServer`, the `Supervisor` can implement the [`HandleSupervisorMessage`] or the
/// [`HandleSupervisorRequest`] trait to handle different messages sent to it.
///
/// It also implements the [`Supervise`] trait, allowing it to define a list of other processes
/// that are supervised.
///
/// # Example
///
/// ```
/// ```
pub struct Supervisor<T> {
    id: u64,
    // If set to true, the host call `lunatic::process::drop_process` will not be executed on drop.
    consumed: UnsafeCell<bool>,
    serializer_type: PhantomData<T>,
}

impl<M, S, T> Message<M, S> for Supervisor<T>
where
    T: HandleSupervisorMessage<M, S>,
    S: Serializer<M>,
{
    /// Send message to the server.
    fn send(&self, message: M) {
        fn unpacker<TU, MU, SU>(this: &mut TU, children: &mut Children)
        where
            TU: HandleSupervisorMessage<MU, SU>,
            SU: Serializer<MU>,
        {
            let message: MU = SU::decode().unwrap();
            <TU as HandleSupervisorMessage<MU, SU>>::handle(this, message, children);
        }

        // Create new message buffer.
        unsafe { host_api::message::create_data(1, 0) };
        // First encode the handler inside the message buffer.
        let handler = unpacker::<T, M, S> as usize as i32;
        let handler_message = Sendable::Message(handler);
        Bincode::encode(&handler_message).unwrap();
        // Then the message itself.
        S::encode(&message).unwrap();
        // Send the message
        unsafe { host_api::message::send(self.id) };
    }
}

impl<T> Resource for Supervisor<T> {
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

impl<M, S, T> Request<M, S> for Supervisor<T>
where
    T: HandleSupervisorRequest<M, S>,
    S: Serializer<M> + Serializer<<T as HandleSupervisorRequest<M, S>>::Result>,
{
    type Result = <T as HandleSupervisorRequest<M, S>>::Result;

    /// Send request to the server and block until an answer is received.
    fn request(&self, request: M) -> Self::Result {
        fn unpacker<TU, MU, SU>(
            this: &mut TU,
            sender: Process<<TU as HandleSupervisorRequest<MU, SU>>::Result, SU>,
            children: &mut Children,
        ) where
            TU: HandleSupervisorRequest<MU, SU>,
            SU: Serializer<MU> + Serializer<<TU as HandleSupervisorRequest<MU, SU>>::Result>,
        {
            // Get content out of message
            let message: MU = SU::decode().unwrap();
            // Get tag out of message before the handler function maybe manipulates it.
            let tag = unsafe { host_api::message::get_tag() };
            let tag = Tag::from(tag);
            let result = <TU as HandleSupervisorRequest<MU, SU>>::handle(this, message, children);
            sender.tag_send(tag, result);
        }

        let tag = Tag::new();
        // Create new message buffer.
        unsafe { host_api::message::create_data(tag.id(), 0) };
        // Create reference to self
        let this_id = unsafe { host_api::process::this() };
        let this_proc: Process<()> = unsafe { Process::from_id(this_id) };
        // First encode the handler inside the message buffer.
        let handler = unpacker::<T, M, S> as usize as i32;
        let handler_message = Sendable::Request(handler, this_proc);
        Bincode::encode(&handler_message).unwrap();
        // Then the message itself.
        S::encode(&request).unwrap();
        // Send it & wait on a response!
        unsafe { host_api::message::send_receive_skip_search(self.id, 0) };
        S::decode().unwrap()
    }
}

impl<T> Supervisor<T> {
    /// Construct a process from a raw ID.
    unsafe fn from(id: u64) -> Self {
        Supervisor {
            id,
            consumed: UnsafeCell::new(false),
            serializer_type: PhantomData,
        }
    }

    /// Returns a globally unique process ID.
    pub fn uuid(&self) -> u128 {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host_api::process::id(self.id, &mut uuid as *mut [u8; 16]) };
        u128::from_le_bytes(uuid)
    }

    fn send_init<C>(&self, message: C)
    where
        C: serde::Serialize + serde::de::DeserializeOwned,
    {
        // Create new message.
        unsafe { host_api::message::create_data(1, 0) };
        // During serialization resources will add themself to the message.
        Bincode::encode(&message).unwrap();
        // Send it!
        unsafe { host_api::message::send(self.id) };
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

impl<T> IntoProcess<T> for Supervisor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Supervise,
{
    type Handler = fn(state: &mut T);

    fn spawn(
        module: Option<WasmModule>,
        config: Option<&ProcessConfig>,
        state: T,
        init: Self::Handler,
    ) -> Result<Supervisor<T>, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, config, None, state, init)
    }
}

impl<T> IntoProcessLink<T> for Supervisor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Supervise,
{
    type Handler = fn(state: &mut T);

    fn spawn_link(
        module: Option<WasmModule>,
        config: Option<&ProcessConfig>,
        tag: Tag,
        state: T,
        init: Self::Handler,
    ) -> Result<Supervisor<T>, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, config, Some(tag), state, init)
    }
}

// `spawn` performs a low level dance that will turn a high level rust function and state into a
// correct lunatic server.
//
// For more info on how this function works, read the explanation inside super::process::spawn.
fn spawn<T>(
    module: Option<WasmModule>,
    config: Option<&ProcessConfig>,
    link: Option<Tag>,
    state: T,
    init: fn(state: &mut T),
) -> Result<Supervisor<T>, LunaticError>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Supervise,
{
    let (type_helper, init) = (
        type_helper_wrapper::<T> as usize as i32,
        init as usize as i32,
    );

    let params = params_to_vec(&[Param::I32(type_helper), Param::I32(init)]);
    let mut id = 0;
    let func = "_lunatic_spawn_supervisor_by_index";
    let link = match link {
        Some(tag) => tag.id(),
        None => 0,
    };
    let module_id = module.unwrap_or_else(WasmModule::inherit).id();
    let config_id = config.map_or_else(|| ProcessConfig::inherit().id(), |config| config.id());
    let result = unsafe {
        host_api::process::spawn(
            link,
            config_id,
            module_id,
            func.as_ptr(),
            func.len(),
            params.as_ptr(),
            params.len(),
            &mut id,
        )
    };
    if result == 0 {
        // If the captured variable is of size 0, we don't need to send it to another process.
        if std::mem::size_of::<T>() == 0 {
            Ok(Supervisor {
                id,
                consumed: UnsafeCell::new(false),
                serializer_type: PhantomData,
            })
        } else {
            let child = Supervisor::<T> {
                id,
                consumed: UnsafeCell::new(false),
                serializer_type: PhantomData,
            };
            child.send_init(state);
            Ok(child)
        }
    } else {
        Err(LunaticError::from(id))
    }
}

// Type helper
fn type_helper_wrapper<T>(function: usize)
where
    T: serde::Serialize + serde::de::DeserializeOwned + Supervise,
{
    // If the captured variable is of size 0, don't wait on it.
    let mut state = if std::mem::size_of::<T>() == 0 {
        unsafe { std::mem::MaybeUninit::<T>::zeroed().assume_init() }
    } else {
        unsafe { Mailbox::<T, Bincode>::new() }.receive()
    };

    // Run the init function before anything else
    let init: fn(state: &mut T) = unsafe { std::mem::transmute(function) };
    init(&mut state);

    let mailbox: LinkMailbox<Sendable, Bincode> = unsafe { LinkMailbox::new() };

    let mut children = Children::new();
    // Collect information on the children.
    state.init(&mut children);
    // Spawn all children
    children.spawn();

    // Run server forever and respond to requests.
    loop {
        let result = mailbox.tag_receive(None);
        match result {
            Ok(dispatcher) => match dispatcher {
                Sendable::Message(handler) => {
                    let handler: fn(state: &mut T, children: &mut Children) =
                        unsafe { std::mem::transmute(handler) };
                    handler(&mut state, &mut children);
                }
                Sendable::Request(handler, sender) => {
                    let handler: fn(state: &mut T, sender: Process<()>, children: &mut Children) =
                        unsafe { std::mem::transmute(handler) };
                    handler(&mut state, sender, &mut children);
                }
            },
            Err(trap) => children.re_spawn(trap.tag()),
        }
    }
}

#[export_name = "_lunatic_spawn_supervisor_by_index"]
extern "C" fn _lunatic_spawn_supervisor_by_index(type_helper: usize, function: usize) {
    let type_helper: fn(usize) = unsafe { std::mem::transmute(type_helper) };
    type_helper(function);
}

// Processes are equal if their UUID is equal.
impl<T> PartialEq for Supervisor<T> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

impl<T> std::fmt::Debug for Supervisor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("uuid", &self.uuid())
            .finish()
    }
}

impl<T> Clone for Supervisor<T> {
    fn clone(&self) -> Self {
        let id = unsafe { host_api::process::clone_process(self.id) };
        unsafe { Supervisor::from(id) }
    }
}

impl<T> Drop for Supervisor<T> {
    fn drop(&mut self) {
        // Only drop a process if it's not already consumed.
        if unsafe { !*self.consumed.get() } {
            unsafe { host_api::process::drop_process(self.id) };
        }
    }
}

impl<T> serde::Serialize for Supervisor<T> {
    fn serialize<A>(&self, serializer: A) -> Result<A::Ok, A::Error>
    where
        A: serde::Serializer,
    {
        // Mark process as consumed.
        unsafe { self.consume() };

        let index = unsafe { host_api::message::push_process(self.id) };
        serializer.serialize_u64(index)
    }
}

struct SupervisorVisitor<T> {
    _phantom: PhantomData<T>,
}

impl<'de, T> serde::de::Visitor<'de> for SupervisorVisitor<T> {
    type Value = Supervisor<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an u64 index")
    }

    fn visit_u64<E>(self, index: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let id = unsafe { host_api::message::take_process(index) };
        Ok(unsafe { Supervisor::from(id) })
    }
}

impl<'de, T> serde::de::Deserialize<'de> for Supervisor<T> {
    fn deserialize<D>(deserializer: D) -> Result<Supervisor<T>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_u64(SupervisorVisitor {
            _phantom: PhantomData {},
        })
    }
}

#[cfg(test)]
mod tests {
    use lunatic_test::test;
    use std::time::Duration;

    use super::*;
    use crate::{
        process::{sleep, spawn, spawn_link},
        BackgroundTask, Server,
    };

    #[derive(serde::Serialize, serde::Deserialize, Default)]
    struct TestSupervisor {
        child: Option<Supervised<Server<i32, i32>>>,
    }

    impl Supervise for TestSupervisor {
        fn init(&mut self, children: &mut Children) {
            // Supervise a server
            let supervised = children.supervise::<Server<i32, i32>, _>(0, |state, message| {
                *state += message;
                // If we reach a state of 3 fail after answering request
                if *state == 3 {
                    spawn_link::<BackgroundTask, _>((), |_| panic!("kill parent")).unwrap();
                }
                *state
            });
            self.child = Some(supervised);
        }
    }

    impl HandleSupervisorRequest<i32> for TestSupervisor {
        type Result = i32;

        fn handle(&mut self, req: i32, children: &mut Children) -> Self::Result {
            // Just forward the request to the supervised server.
            let proc = self.child.as_mut().unwrap().process(children).unwrap();
            proc.request(req)
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Panic;

    impl HandleSupervisorMessage<Panic> for TestSupervisor {
        fn handle(&mut self, _: Panic, _children: &mut Children) {
            panic!("fail");
        }
    }

    #[test]
    fn spawn_test() {
        let supervisor = spawn::<Supervisor<_>, _>(TestSupervisor::default(), |_state| {}).unwrap();

        assert_eq!(supervisor.request(1), 1);
        assert_eq!(supervisor.request(1), 2);
        assert_eq!(supervisor.request(1), 3);
        // Child server dies at this point.
        // New child starts from fresh state
        sleep(Duration::from_millis(100));
        assert_eq!(supervisor.request(1), 1);
        assert_eq!(supervisor.request(1), 2);
        assert_eq!(supervisor.request(1), 3);

        sleep(Duration::from_millis(100));
    }

    #[test]
    fn spawn_link_test() {
        // There is no real way of testing traps for now, at least not until this is resolved:
        // https://github.com/lunatic-solutions/rust-lib/issues/8
        // A manual log output observation is necessary her to check if both processes failed.
        spawn::<BackgroundTask, _>((), |_| {
            let child = spawn_link::<Supervisor<_>, _>(TestSupervisor::default(), |_| {}).unwrap();
            // Trigger failure
            child.send(Panic);
            // This process should fails too before 100ms
            sleep(Duration::from_millis(100));
        })
        .unwrap();
        sleep(Duration::from_millis(100));
    }
}
