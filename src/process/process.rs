use std::{cell::UnsafeCell, marker::PhantomData};

use super::{IntoProcess, IntoProcessLink};
use crate::{
    environment::{params_to_vec, Param},
    host_api,
    serializer::{Bincode, Serializer},
    LunaticError, Mailbox,
};

/// A handle to a process that can receive messages of type `M`, serialized by a serializer of type
/// `S`.
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
    /// Construct a process from a raw ID.
    pub unsafe fn from(id: u64) -> Self {
        Process {
            id,
            consumed: UnsafeCell::new(false),
            serializer_type: PhantomData,
        }
    }

    /// Returns a globally unique process ID.
    pub fn id(&self) -> u128 {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host_api::process::id(self.id, &mut uuid as *mut [u8; 16]) };
        u128::from_le_bytes(uuid)
    }

    pub fn send(&self, message: M) {
        // Create new message.
        unsafe { host_api::message::create_data(0, 0) };
        // During serialization resources will add themself to the message.
        S::encode(&message).unwrap();
        // Send it!
        unsafe { host_api::message::send(self.id) };
    }

    /// Link process to the one currently running.
    pub fn link(&self) {
        // Don't use tag IDs because a process' [`Mailbox`] can't differentiate between regular
        // messages and signals. Linked processes will almost always die when a link is broken.
        unsafe { host_api::process::link(0, self.id) };
    }

    /// Unlink processes.
    pub fn unlink(&self) {
        unsafe { host_api::process::unlink(self.id) };
    }

    /// Marks the process as consumed.
    ///
    /// Consumed processes don't call the `lunatic::process::drop_process` host function when they
    /// are dropped. This characteristic is useful when implementing serializers for processes.
    /// Serializers will move the process out of the local state into the message scratch buffer
    /// and they can't be dropped from the local state anymore.
    pub unsafe fn consume(&self) {
        *self.consumed.get() = true;
    }
}

impl<C, M, S> IntoProcess<C> for Process<M, S>
where
    S: Serializer<C> + Serializer<M>,
{
    type Handler = fn(capture: C, arg: Mailbox<M, S>);

    fn spawn(captured: C, handler: Self::Handler) -> Result<Process<M, S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(None, false, captured, handler)
    }
}

impl<C, M, S> IntoProcessLink<C> for Process<M, S>
where
    S: Serializer<C> + Serializer<M>,
{
    type Handler = fn(capture: C, arg: Mailbox<M, S>);

    fn spawn_link(captured: C, handler: Self::Handler) -> Result<Process<M, S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(None, true, captured, handler)
    }
}

// `spawn` performs a low level dance that will turn a high level rust function and captured
// variable into a correct lunatic process.
//
// If `module_id` is None it will use the current module & environment, if it's `Some` it will
// use the current module but spawned inside another environment. Look at [`ThisModule`] for
// more information about sending the current module to another environment.
fn spawn<C, M, S>(
    module_id: Option<u64>,
    link: bool,
    captured: C,
    entry: fn(C, Mailbox<M, S>),
) -> Result<Process<M, S>, LunaticError>
where
    S: Serializer<C> + Serializer<M>,
{
    // Spawning a new process from  the same module is a delicate undertaking. First of all, the
    // WebAssembly spec only allows us to call exported functions from a module. Therefore we
    // define a module export under the name `_lunatic_spawn_by_index`. This global function will
    // get 2 arguments:
    //
    // * A pointer to a type helper function (`type_helper_wrapper`)
    // * A pointer to the function we want to use as the entry point into the process
    //
    // It's obvious why we need the entry function, but what is a "type helper function"? The entry
    // function contains 2 generic types, one for the captured value and one for messages, but the
    // `_lunatic_spawn_by_index` function can't be generic, and we can't call the entry from it. We
    // relay here on Rust generating the right pointer to the correct generic function during
    // monomorphization and send it to the none-generic `_lunatic_spawn_by_index` export.

    let (type_helper, entry) = (type_helper_wrapper::<C, M, S> as i32, entry as i32);

    let params = params_to_vec(&[Param::I32(type_helper), Param::I32(entry)]);
    let mut id = 0;
    let func = "_lunatic_spawn_by_index";
    let link = match link {
        // TODO: Do we want to be notified with the right tag once the link dies?
        //       I assume not, because only supervisors can use this information and we can't spawn
        //       this kind of processes from supervisors.
        true => 1,
        false => 0,
    };
    let result = unsafe {
        match module_id {
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
            Ok(Process {
                id,
                consumed: UnsafeCell::new(false),
                serializer_type: PhantomData,
            })
        } else {
            let child = Process::<C, S> {
                id,
                consumed: UnsafeCell::new(false),
                serializer_type: PhantomData,
            };
            child.send(captured);
            // Processes can only receive one type of message, but to pass in the captured variable
            // we pretend for the first message that our process is receiving messages of type `C`.
            Ok(unsafe { std::mem::transmute(child) })
        }
    } else {
        Err(LunaticError::from(id))
    }
}

// Type helper
fn type_helper_wrapper<C, M, S>(function: usize)
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

#[export_name = "_lunatic_spawn_by_index"]
extern "C" fn _lunatic_spawn_by_index(type_helper: usize, function: usize) {
    let type_helper: fn(usize) = unsafe { std::mem::transmute(type_helper) };
    type_helper(function);
}

// Processes are equal if their UUID is equal.
impl<M, S> PartialEq for Process<M, S>
where
    S: Serializer<M>,
{
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<M, S> std::fmt::Debug for Process<M, S>
where
    S: Serializer<M>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process").field("uuid", &self.id()).finish()
    }
}

impl<M, S> Clone for Process<M, S>
where
    S: Serializer<M>,
{
    fn clone(&self) -> Self {
        let id = unsafe { host_api::process::clone_process(self.id) };
        unsafe { Process::from(id) }
    }
}

impl<M, S> Drop for Process<M, S>
where
    S: Serializer<M>,
{
    fn drop(&mut self) {
        // Only drop a process if it's not already consumed.
        if unsafe { !*self.consumed.get() } {
            unsafe { host_api::process::drop_process(self.id) };
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

        let index = unsafe { host_api::message::push_process(self.id) };
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
        let id = unsafe { host_api::message::take_process(index) };
        Ok(unsafe { Process::from(id) })
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
    use super::*;
    use crate::process::{sleep, spawn, spawn_link};

    #[test]
    fn spawn_test() {
        let child = spawn::<Process<i32>, _>(1, |capture, mailbox| {
            assert_eq!(capture, 1);
            assert_eq!(mailbox.receive(), 2);
        })
        .unwrap();
        child.send(2);
        sleep(100);
    }

    #[test]
    fn spawn_link_test() {
        // There is no real way of testing traps for now, at least not until this is resolved:
        // https://github.com/lunatic-solutions/rust-lib/issues/8
        // A manual log output observation is necessary her to check if both processes failed.
        spawn::<Process<i32>, _>((), |_, _| {
            spawn_link::<Process<i32>, _>((), |_, _| {
                panic!("fails");
            })
            .unwrap();
            // This process should fails too before 100ms
            sleep(100)
        })
        .unwrap();
        sleep(100);
    }
}
