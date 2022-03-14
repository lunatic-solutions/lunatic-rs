use std::marker::PhantomData;

use super::{IntoProcess, IntoProcessLink};
use crate::{
    host_api,
    module::{params_to_vec, Param, WasmModule},
    serializer::{Bincode, Serializer},
    LunaticError, Mailbox, ProcessConfig, Resource, Tag,
};

/// An one-off process spawned from a function that can capture some input from the parent.
///
/// # Example
///
/// ```
/// // Run computation in different process.
/// spawn::<BackgroundTask, _>((2, 3), |(a, b)| {a + b;}).unwrap();
/// ```
pub struct BackgroundTask<S = Bincode> {
    id: u64,
    serializer_type: PhantomData<S>,
}

impl<S> BackgroundTask<S> {
    /// Returns a globally unique process ID.
    pub fn uuid(&self) -> u128 {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host_api::process::id(self.id, &mut uuid as *mut [u8; 16]) };
        u128::from_le_bytes(uuid)
    }

    fn send<C>(&self, message: C)
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
}

impl<S> Resource for BackgroundTask<S> {
    fn id(&self) -> u64 {
        self.id
    }

    unsafe fn from_id(id: u64) -> Self {
        Self {
            id,
            serializer_type: PhantomData,
        }
    }
}

impl<C, S> IntoProcess<C> for BackgroundTask<S>
where
    S: Serializer<C>,
{
    type Handler = fn(capture: C);

    fn spawn(
        module: Option<WasmModule>,
        config: Option<&ProcessConfig>,
        capture: C,
        handler: Self::Handler,
    ) -> Result<BackgroundTask<S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, config, None, capture, handler)
    }
}

impl<C, S> IntoProcessLink<C> for BackgroundTask<S>
where
    S: Serializer<C>,
{
    type Handler = fn(capture: C);

    fn spawn_link(
        module: Option<WasmModule>,
        config: Option<&ProcessConfig>,
        tag: Tag,
        capture: C,
        handler: Self::Handler,
    ) -> Result<BackgroundTask<S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, config, Some(tag), capture, handler)
    }
}

// `spawn` performs a low level dance that will turn a high level rust function and captured
// variable into a correct lunatic task.
//
// For more info on how this function works, read the explanation inside super::process::spawn.
fn spawn<C, S>(
    module: Option<WasmModule>,
    config: Option<&ProcessConfig>,
    link: Option<Tag>,
    capture: C,
    entry: fn(C),
) -> Result<BackgroundTask<S>, LunaticError>
where
    S: Serializer<C>,
{
    let (type_helper, entry) = (
        type_helper_wrapper::<C, S> as usize as i32,
        entry as usize as i32,
    );

    let params = params_to_vec(&[Param::I32(type_helper), Param::I32(entry)]);
    let mut id = 0;
    let func = "_lunatic_spawn_task_by_index";
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
        if std::mem::size_of::<C>() == 0 {
            Ok(BackgroundTask::<S> {
                id,
                serializer_type: PhantomData,
            })
        } else {
            let child = BackgroundTask::<S> {
                id,
                serializer_type: PhantomData,
            };
            child.send(capture);
            Ok(child)
        }
    } else {
        Err(LunaticError::from(id))
    }
}

// Type helper
fn type_helper_wrapper<C, S>(function: usize)
where
    S: Serializer<C>,
{
    // If the captured variable is of size 0, don't wait on it.
    let capture = if std::mem::size_of::<C>() == 0 {
        unsafe { std::mem::MaybeUninit::<C>::zeroed().assume_init() }
    } else {
        unsafe { Mailbox::<C, S>::new() }.receive()
    };
    let function: fn(C) = unsafe { std::mem::transmute(function) };
    function(capture);
}

#[export_name = "_lunatic_spawn_task_by_index"]
extern "C" fn _lunatic_spawn_task_by_index(type_helper: usize, function: usize) {
    let type_helper: fn(usize) = unsafe { std::mem::transmute(type_helper) };
    type_helper(function);
}

// Processes are equal if their UUID is equal.
impl<S> PartialEq for BackgroundTask<S> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

impl<S> std::fmt::Debug for BackgroundTask<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("uuid", &self.uuid())
            .finish()
    }
}

impl<S> Drop for BackgroundTask<S> {
    fn drop(&mut self) {
        unsafe { host_api::process::drop_process(self.id) };
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::{
        process::{spawn, spawn_link},
        sleep,
    };

    #[test]
    fn spawn_test() {
        let _child = spawn::<BackgroundTask, _>(1, |capture| {
            assert_eq!(capture, 1);
        })
        .unwrap();
    }

    #[test]
    fn spawn_link_test() {
        // There is no real way of testing traps for now, at least not until this is resolved:
        // https://github.com/lunatic-solutions/rust-lib/issues/8
        // A manual log output observation is necessary her to check if both processes failed.
        let _child = spawn::<BackgroundTask, _>((), |_| {
            let _child = spawn_link::<BackgroundTask, _>((), |_| {
                panic!("fails");
            })
            .unwrap();
            sleep(Duration::from_millis(100));
        })
        .unwrap();
        sleep(Duration::from_millis(100));
    }
}
