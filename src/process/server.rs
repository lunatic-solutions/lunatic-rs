use std::{cell::UnsafeCell, marker::PhantomData};

use super::{IntoProcess, IntoProcessLink, Process};
use crate::{
    environment::{params_to_vec, Param},
    host_api,
    serializer::{Bincode, Serializer},
    LunaticError, Mailbox, Resource, Tag,
};

/// A [`Server`] is a simple process spawned from a function that can maintain a state, runs in a
/// loop and answers requests sent to it.
pub struct Server<M, R, S = Bincode>
where
    S: Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    id: u64,
    // If set to true, the host call `lunatic::process::drop_process` will not be executed on drop.
    consumed: UnsafeCell<bool>,
    serializer_type: PhantomData<(M, R, S)>,
}

impl<M, R, S> Server<M, R, S>
where
    S: Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    /// Construct a process from a raw ID.
    unsafe fn from(id: u64) -> Self {
        Server {
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

    pub fn request(&self, message: M) -> R {
        let tag = Tag::new();
        // Create new message.
        unsafe { host_api::message::create_data(tag.id(), 0) };
        // Create reference to self
        let this_id = unsafe { host_api::process::this() };
        let this_proc: Process<R, S> = unsafe { Process::from(this_id) };
        // During serialization resources will add themself to the message.
        S::encode(&(this_proc, message)).unwrap();
        // Send it & wait on a response!
        unsafe { host_api::message::send_receive_skip_search(self.id, 0) };
        S::decode().unwrap()
    }

    fn send_init<C>(&self, message: C)
    where
        S: Serializer<C>,
    {
        // Create new message.
        unsafe { host_api::message::create_data(1, 0) };
        // During serialization resources will add themself to the message.
        S::encode(&message).unwrap();
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

impl<M, R, S> Resource for Server<M, R, S>
where
    S: Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    fn id(&self) -> u64 {
        self.id
    }
}

impl<C, M, R, S> IntoProcess<C> for Server<M, R, S>
where
    S: Serializer<C> + Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    type Handler = fn(state: &mut C, request: M) -> R;

    fn spawn(
        module: Option<u64>,
        state: C,
        handler: Self::Handler,
    ) -> Result<Server<M, R, S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, false, state, handler)
    }
}

impl<C, M, R, S> IntoProcessLink<C> for Server<M, R, S>
where
    S: Serializer<C> + Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    type Handler = fn(state: &mut C, request: M) -> R;

    fn spawn_link(
        module: Option<u64>,
        state: C,
        handler: Self::Handler,
    ) -> Result<Server<M, R, S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, true, state, handler)
    }
}

// `spawn` performs a low level dance that will turn a high level rust function and state into a
// correct lunatic server.
//
// For more info on how this function works, read the explanation inside super::process::spawn.
fn spawn<C, M, R, S>(
    module: Option<u64>,
    link: bool,
    state: C,
    handler: fn(state: &mut C, request: M) -> R,
) -> Result<Server<M, R, S>, LunaticError>
where
    S: Serializer<C> + Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    let (type_helper, handler) = (
        type_helper_wrapper::<C, M, R, S> as usize as i32,
        handler as usize as i32,
    );

    let params = params_to_vec(&[Param::I32(type_helper), Param::I32(handler)]);
    let mut id = 0;
    let func = "_lunatic_spawn_server_by_index";
    let link = match link {
        // TODO: Do we want to be notified with the right tag once the link dies?
        //       I assume not, because only supervisors can use this information and we can't spawn
        //       this kind of processes from supervisors.
        true => 1,
        false => 0,
    };
    let result = unsafe {
        match module {
            Some(module_id) => host_api::process::spawn(
                link,
                module_id,
                func.as_ptr(),
                func.len(),
                params.as_ptr(),
                params.len(),
                &mut id,
            ),
            None => host_api::process::inherit_spawn(
                link,
                func.as_ptr(),
                func.len(),
                params.as_ptr(),
                params.len(),
                &mut id,
            ),
        }
    };
    if result == 0 {
        // If the captured variable is of size 0, we don't need to send it to another process.
        if std::mem::size_of::<C>() == 0 {
            Ok(Server {
                id,
                consumed: UnsafeCell::new(false),
                serializer_type: PhantomData,
            })
        } else {
            let child = Server::<M, R, S> {
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
fn type_helper_wrapper<C, M, R, S>(function: usize)
where
    S: Serializer<C> + Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    // If the captured variable is of size 0, don't wait on it.
    let mut state = if std::mem::size_of::<C>() == 0 {
        unsafe { std::mem::MaybeUninit::<C>::zeroed().assume_init() }
    } else {
        unsafe { Mailbox::<C, S>::new() }.receive()
    };
    let mailbox: Mailbox<(Process<R, S>, M), S> = unsafe { Mailbox::new() };
    let handler: fn(state: &mut C, request: M) -> R = unsafe { std::mem::transmute(function) };

    // Run server forever and respond to requests.
    loop {
        let (sender, message) = mailbox.tag_receive(None);
        let tag = unsafe { host_api::message::get_tag() };
        let tag = Tag::from(tag);
        let response = handler(&mut state, message);
        sender.tag_send(tag, response);
    }
}

#[export_name = "_lunatic_spawn_server_by_index"]
extern "C" fn _lunatic_spawn_server_by_index(type_helper: usize, function: usize) {
    let type_helper: fn(usize) = unsafe { std::mem::transmute(type_helper) };
    type_helper(function);
}

// Processes are equal if their UUID is equal.
impl<M, R, S> PartialEq for Server<M, R, S>
where
    S: Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

impl<M, R, S> std::fmt::Debug for Server<M, R, S>
where
    S: Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("uuid", &self.uuid())
            .finish()
    }
}

impl<M, R, S> Clone for Server<M, R, S>
where
    S: Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    fn clone(&self) -> Self {
        let id = unsafe { host_api::process::clone_process(self.id) };
        unsafe { Server::from(id) }
    }
}

impl<M, R, S> Drop for Server<M, R, S>
where
    S: Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    fn drop(&mut self) {
        // Only drop a process if it's not already consumed.
        if unsafe { !*self.consumed.get() } {
            unsafe { host_api::process::drop_process(self.id) };
        }
    }
}

impl<M, R, S> serde::Serialize for Server<M, R, S>
where
    S: Serializer<(Process<R, S>, M)> + Serializer<R>,
{
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

struct ServerVisitor<M, R, S> {
    _phantom: PhantomData<(M, R, S)>,
}

impl<'de, M, R, S> serde::de::Visitor<'de> for ServerVisitor<M, R, S>
where
    S: Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    type Value = Server<M, R, S>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an u64 index")
    }

    fn visit_u64<E>(self, index: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let id = unsafe { host_api::message::take_process(index) };
        Ok(unsafe { Server::from(id) })
    }
}

impl<'de, M, R, S> serde::de::Deserialize<'de> for Server<M, R, S>
where
    S: Serializer<(Process<R, S>, M)> + Serializer<R>,
{
    fn deserialize<D>(deserializer: D) -> Result<Server<M, R, S>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_u64(ServerVisitor {
            _phantom: PhantomData {},
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::process::{sleep, spawn, spawn_link};

    #[test]
    fn spawn_test() {
        let child = spawn::<Server<i32, i32>, _>(0, |state, message| {
            *state += message;
            *state
        })
        .unwrap();
        assert_eq!(child.request(1), 1);
        assert_eq!(child.request(2), 3);
        assert_eq!(child.request(3), 6);
    }

    #[test]
    fn spawn_link_test() {
        // There is no real way of testing traps for now, at least not until this is resolved:
        // https://github.com/lunatic-solutions/rust-lib/issues/8
        // A manual log output observation is necessary her to check if both processes failed.
        spawn::<Server<(), _>, _>((), |_, _| {
            let child = spawn_link::<Server<(), _>, _>((), |_, _| {
                panic!("fails");
            })
            .unwrap();
            // Trigger failure
            child.request(());
            // This process should fails too before 100ms
            sleep(Duration::from_millis(100));
        })
        .unwrap();
        sleep(Duration::from_millis(100));
    }
}
