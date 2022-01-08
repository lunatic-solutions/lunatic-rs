use std::marker::PhantomData;

use super::{IntoProcess, IntoProcessLink};
use crate::{
    environment::{params_to_vec, Param},
    host_api,
    serializer::{Bincode, Serializer},
    LunaticError, Mailbox, Resource,
};

/// An one-off process spawned from a function that can capture some input from the parent.
///
/// # Example
///
/// ```
/// // Run computation in different process.
/// spawn::<AsyncTask<_>, _>((2, 3), |(a, b)| {a + b;}).unwrap();
/// ```
pub struct AsyncTask<S = Bincode> {
    id: u64,
    serializer_type: PhantomData<S>,
}

impl<S> AsyncTask<S> {
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

impl<S> Resource for AsyncTask<S> {
    fn id(&self) -> u64 {
        self.id
    }
}

impl<C, S> IntoProcess<C> for AsyncTask<S>
where
    S: Serializer<C>,
{
    type Handler = fn(capture: C);

    fn spawn(
        module: Option<u64>,
        capture: C,
        handler: Self::Handler,
    ) -> Result<AsyncTask<S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, false, capture, handler)
    }
}

impl<C, S> IntoProcessLink<C> for AsyncTask<S>
where
    S: Serializer<C>,
{
    type Handler = fn(capture: C);

    fn spawn_link(
        module: Option<u64>,
        capture: C,
        handler: Self::Handler,
    ) -> Result<AsyncTask<S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(module, true, capture, handler)
    }
}

// `spawn` performs a low level dance that will turn a high level rust function and captured
// variable into a correct lunatic task.
//
// For more info on how this function works, read the explanation inside super::process::spawn.
fn spawn<C, S>(
    module: Option<u64>,
    link: bool,
    capture: C,
    entry: fn(C),
) -> Result<AsyncTask<S>, LunaticError>
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
            Ok(AsyncTask::<S> {
                id,
                serializer_type: PhantomData,
            })
        } else {
            let child = AsyncTask::<S> {
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
impl<S> PartialEq for AsyncTask<S> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

impl<S> std::fmt::Debug for AsyncTask<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("uuid", &self.uuid())
            .finish()
    }
}

impl<S> Drop for AsyncTask<S> {
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
        let _child = spawn::<AsyncTask, _>(1, |capture| {
            assert_eq!(capture, 1);
        })
        .unwrap();
    }

    #[test]
    fn spawn_link_test() {
        // There is no real way of testing traps for now, at least not until this is resolved:
        // https://github.com/lunatic-solutions/rust-lib/issues/8
        // A manual log output observation is necessary her to check if both processes failed.
        let _child = spawn::<AsyncTask, _>((), |_| {
            let _child = spawn_link::<AsyncTask, _>((), |_| {
                panic!("fails");
            })
            .unwrap();
            sleep(Duration::from_millis(100));
        })
        .unwrap();
        sleep(Duration::from_millis(100));
    }
}
