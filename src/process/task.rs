use std::marker::PhantomData;

use super::{IntoProcess, IntoProcessLink, Process};
use crate::{
    host_api,
    module::{params_to_vec, Param, WasmModule},
    serializer::{Bincode, Serializer},
    LunaticError, Mailbox, ProcessConfig, Resource, Tag,
};

/// An one-off process spawned from a function that can capture some input from the parent and send
/// back a result.
///
/// When [`result`](Task::result) is called it will block until the async computation is done and
/// a result available. If you don't want to wait on the result use an
/// [`BackgroundTask`](crate::BackgroundTask).
///
/// # Example
///
/// ```
/// // Run computation in different process.
/// let child = spawn::<Task<_>, _>((2, 3), |(a, b)| a + b).unwrap();
/// // Wait for process to finish and get result.
/// assert_eq!(child.result(), 5);
/// ```
#[must_use = "If `result` is not called on `Task` it will leak memory. Use `BackgroundTask` instead."]
pub struct Task<M, S = Bincode>
where
    S: Serializer<M>,
{
    id: u64,
    // A tag is used to match the return message to the correct task.
    tag: Tag,
    serializer_type: PhantomData<(M, S)>,
}

impl<M, S> Task<M, S>
where
    S: Serializer<M>,
{
    /// Returns a globally unique process ID.
    pub fn uuid(&self) -> u128 {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host_api::process::id(self.id, &mut uuid as *mut [u8; 16]) };
        u128::from_le_bytes(uuid)
    }

    /// Wait for the result of the task.
    ///
    /// This function will block until the task returns a result. It must be called on all tasks
    /// or the returned result will stay forever inside the mailbox.
    pub fn result(self) -> M {
        unsafe { Mailbox::<M, S>::new() }.tag_receive(Some(&[self.tag]))
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

impl<C, M, S> IntoProcess<C> for Task<M, S>
where
    S: Serializer<(Process<M, S>, Tag, C)> + Serializer<M>,
{
    type Handler = fn(capture: C) -> M;

    fn spawn(
        module: Option<WasmModule>,
        config: Option<&ProcessConfig>,
        capture: C,
        handler: Self::Handler,
    ) -> Result<Task<M, S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, config, None, capture, handler)
    }
}

impl<C, M, S> IntoProcessLink<C> for Task<M, S>
where
    S: Serializer<(Process<M, S>, Tag, C)> + Serializer<M>,
{
    type Handler = fn(capture: C) -> M;

    fn spawn_link(
        module: Option<WasmModule>,
        config: Option<&ProcessConfig>,
        tag: Tag,
        capture: C,
        handler: Self::Handler,
    ) -> Result<Task<M, S>, LunaticError>
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
fn spawn<C, M, S>(
    module: Option<WasmModule>,
    config: Option<&ProcessConfig>,
    link: Option<Tag>,
    capture: C,
    entry: fn(C) -> M,
) -> Result<Task<M, S>, LunaticError>
where
    S: Serializer<(Process<M, S>, Tag, C)> + Serializer<M>,
{
    let (type_helper, entry) = (
        type_helper_wrapper::<C, M, S> as usize as i32,
        entry as usize as i32,
    );

    let params = params_to_vec(&[Param::I32(type_helper), Param::I32(entry)]);
    let mut id = 0;
    let func = "_lunatic_spawn_async_task_by_index";
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
        let child = Task::<M, S> {
            id,
            tag,
            serializer_type: PhantomData,
        };
        // Create reference to self
        let this_id = unsafe { host_api::process::this() };
        let this_proc: Process<M, S> = unsafe { Process::from_id(this_id) };
        // Send all data to child
        child.send((this_proc, tag, capture));
        Ok(child)
    } else {
        Err(LunaticError::from(id))
    }
}

// Type helper
fn type_helper_wrapper<C, M, S>(function: usize)
where
    S: Serializer<(Process<M, S>, Tag, C)> + Serializer<M>,
{
    let (parent, tag, capture) = unsafe { Mailbox::<(Process<M, S>, Tag, C), S>::new() }.receive();
    let function: fn(C) -> M = unsafe { std::mem::transmute(function) };
    let result = function(capture);
    parent.tag_send(tag, result);
}

#[export_name = "_lunatic_spawn_async_task_by_index"]
extern "C" fn _lunatic_spawn_async_task_by_index(type_helper: usize, function: usize) {
    let type_helper: fn(usize) = unsafe { std::mem::transmute(type_helper) };
    type_helper(function);
}

// Processes are equal if their UUID is equal.
impl<M, S> PartialEq for Task<M, S>
where
    S: Serializer<M>,
{
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

impl<M, S> std::fmt::Debug for Task<M, S>
where
    S: Serializer<M>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("uuid", &self.uuid())
            .finish()
    }
}

impl<M, S> Drop for Task<M, S>
where
    S: Serializer<M>,
{
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
        let child = spawn::<Task<i32>, _>(1, |capture| {
            assert_eq!(capture, 1);
            2
        })
        .unwrap();
        assert_eq!(child.result(), 2);
    }

    #[test]
    fn spawn_link_test() {
        // There is no real way of testing traps for now, at least not until this is resolved:
        // https://github.com/lunatic-solutions/rust-lib/issues/8
        // A manual log output observation is necessary her to check if both processes failed.
        let _ = spawn::<Task<()>, _>((), |_| {
            let child = spawn_link::<Task<()>, _>((), |_| {
                panic!("fails");
            })
            .unwrap();
            child.result();
        })
        .unwrap();
        sleep(Duration::from_millis(100));
    }
}
