pub mod session;

use std::{any::TypeId, marker::PhantomData};

use self::session::{End, HasDual};

use super::{IntoProcess, IntoProcessLink};
use crate::{
    host_api,
    module::{params_to_vec, Param, WasmModule},
    serializer::{Bincode, Serializer},
    LunaticError, Mailbox, Process, ProcessConfig, Resource, Tag,
};

pub struct Protocol<P, S = Bincode>
where
    P: 'static,
{
    id: u64,
    tag: Tag,
    phantom: PhantomData<(P, S)>,
}

impl<P, S> Protocol<P, S>
where
    P: 'static,
{
    /// Returns a globally unique process ID.
    pub fn uuid(&self) -> u128 {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host_api::process::id(self.id, &mut uuid as *mut [u8; 16]) };
        u128::from_le_bytes(uuid)
    }

    /// Send a message to the process.
    ///
    /// # Panics
    ///
    /// The operation will panic if `message` can't be serialized using serializer `S`.
    fn send_init<C>(&self, message: (Process<()>, Tag, C))
    where
        S: Serializer<(Process<()>, Tag, C)>,
    {
        // Create new message.
        unsafe { host_api::message::create_data(1, 0) };
        // During serialization resources will add themself to the message.
        S::encode(&message).unwrap();
        // Send it!
        unsafe { host_api::message::send(self.id) };
    }
}

impl<P, S> Resource for Protocol<P, S> {
    fn id(&self) -> u64 {
        self.id
    }

    unsafe fn from_id(_id: u64) -> Self {
        unreachable!("Can't create a Protocol from id");
    }
}

impl<C, P, S> IntoProcess<C> for Protocol<P, S>
where
    S: Serializer<(Process<()>, Tag, C)>,
    P: HasDual,
{
    type Handler = fn(capture: C, arg: Protocol<<P as HasDual>::Dual, S>);

    fn spawn(
        module: Option<WasmModule>,
        config: Option<&ProcessConfig>,
        captured: C,
        handler: Self::Handler,
    ) -> Result<Self, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, config, None, captured, handler)
    }
}

impl<C, P, S> IntoProcessLink<C> for Protocol<P, S>
where
    S: Serializer<(Process<()>, Tag, C)>,
    P: HasDual,
{
    type Handler = fn(capture: C, arg: Protocol<<P as HasDual>::Dual, S>);

    fn spawn_link(
        module: Option<WasmModule>,
        config: Option<&ProcessConfig>,
        tag: Tag,
        captured: C,
        handler: Self::Handler,
    ) -> Result<Self, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, config, Some(tag), captured, handler)
    }
}

// `spawn` performs a low level dance that will turn a high level rust function and captured
// variable into a correct lunatic process.
//
// If `module_id` is None it will use the current module & environment, if it's `Some` it will
// use the current module but spawned inside another environment. Look at [`ThisModule`] for
// more information about sending the current module to another environment.
fn spawn<C, P, S>(
    module: Option<WasmModule>,
    config: Option<&ProcessConfig>,
    link: Option<Tag>,
    captured: C,
    entry: fn(C, Protocol<<P as HasDual>::Dual, S>),
) -> Result<Protocol<P, S>, LunaticError>
where
    S: Serializer<(Process<()>, Tag, C)>,

    P: HasDual,
{
    let (type_helper, entry) = (
        type_helper_wrapper::<C, P, S> as usize as i32,
        entry as usize as i32,
    );

    let params = params_to_vec(&[Param::I32(type_helper), Param::I32(entry)]);
    let mut id = 0;
    let func = "_lunatic_spawn_session_by_index";
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
        let tag = Tag::new();
        let child = Protocol {
            id,
            tag,
            phantom: PhantomData,
        };
        // Create reference to self
        let this_id = unsafe { host_api::process::this() };
        let this_proc: Process<()> = unsafe { Process::from_id(this_id) };
        // Send all data to child
        child.send_init((this_proc, tag, captured));
        Ok(child)
    } else {
        Err(LunaticError::from(id))
    }
}

// Type helper
fn type_helper_wrapper<C, P, S>(function: usize)
where
    S: Serializer<(Process<()>, Tag, C)>,
    P: HasDual + 'static,
{
    let (connection, tag, captured) =
        unsafe { Mailbox::<(Process<()>, Tag, C), S>::new() }.receive();
    let session = unsafe { Protocol::new(connection, tag) };
    let function: fn(C, Protocol<<P as HasDual>::Dual, S>) =
        unsafe { std::mem::transmute(function) };
    function(captured, session);
}

#[export_name = "_lunatic_spawn_session_by_index"]
extern "C" fn _lunatic_spawn_session_by_index(type_helper: usize, function: usize) {
    let type_helper: fn(usize) = unsafe { std::mem::transmute(type_helper) };
    type_helper(function);
}

impl<M, S> std::fmt::Debug for Protocol<M, S>
where
    S: Serializer<M>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Protocol")
            .field("uuid", &self.uuid())
            .finish()
    }
}

// Processes are equal if their UUID is equal.
impl<P, S> PartialEq for Protocol<P, S> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

impl<P, S> Drop for Protocol<P, S>
where
    P: 'static,
{
    fn drop(&mut self) {
        if TypeId::of::<P>() != TypeId::of::<End>() {
            panic!("Protocol prematurely dropped, before reaching the `End` state.");
        }
        unsafe { host_api::process::drop_process(self.id) };
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::session::{End, HasDual, Recv, Send};
    use crate::{
        process::{sleep, spawn, spawn_link},
        BackgroundTask, Protocol,
    };

    type AddProtocol = Recv<i32, Recv<i32, Send<i32, End>>>;

    #[test]
    fn spawn_test() {
        let child =
            spawn::<Protocol<<AddProtocol as HasDual>::Dual>, _>(1, |capture: i32, protocol| {
                assert_eq!(capture, 1);
                let (protocol, a) = protocol.receive();
                let (protocol, b) = protocol.receive();
                let _ = protocol.send(capture + a + b);
            })
            .unwrap();
        assert_eq!(child.tag.id(), 130);
        let child = child.send(2);
        let child = child.send(2);
        let (_, result) = child.receive();
        assert_eq!(result, 5);
    }

    #[test]
    fn spawn_link_test() {
        // There is no real way of testing traps for now, at least not until this is resolved:
        // https://github.com/lunatic-solutions/rust-lib/issues/8
        // A manual log output observation is necessary her to check if both processes failed.
        spawn::<BackgroundTask, _>((), |_| {
            let _dont_drop = spawn_link::<Protocol<AddProtocol>, _>((), |_, _| {
                // Will panic because protocol is dropped without finishing.
            })
            .unwrap();
            // This process should fails too before 100ms
            sleep(Duration::from_millis(100));
        })
        .unwrap();
        sleep(Duration::from_millis(100));
    }
}
