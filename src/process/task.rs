use std::marker::PhantomData;

use super::{IntoProcess, IntoProcessLink, Process};
use crate::{
    environment::{params_to_vec, Param},
    host_api,
    serializer::{Bincode, Serializer},
    LunaticError, Mailbox, Tag,
};

/// A [`Task`] is a simple process spawned from a function that can capture some input from the
/// parent and send back a result.
///
/// When [`result`] is called it will block until the async computation is done and a result
/// available.
///
/// TODO:
///     - Add TaskIterator trait that has methods like `first`, `all` impl for everything
///       that implements Iterator over `AsyncTask`s.
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
    pub fn id(&self) -> u128 {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host_api::process::id(self.id, &mut uuid as *mut [u8; 16]) };
        u128::from_le_bytes(uuid)
    }

    pub fn result(self) -> M {
        unsafe { Mailbox::<M, S>::new() }.tag_receive(&[self.tag])
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

    fn spawn(capture: C, handler: Self::Handler) -> Result<Task<M, S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(false, capture, handler)
    }
}

impl<C, M, S> IntoProcessLink<C> for Task<M, S>
where
    S: Serializer<(Process<M, S>, Tag, C)> + Serializer<M>,
{
    type Handler = fn(capture: C) -> M;

    fn spawn_link(capture: C, handler: Self::Handler) -> Result<Task<M, S>, LunaticError>
    where
        Self: Sized,
    {
        spawn(true, capture, handler)
    }
}

// `spawn` performs a low level dance that will turn a high level rust function and captured
// variable into a correct lunatic task.
//
// For more info on how this function works, read the explanation inside super::process::spawn.
fn spawn<C, M, S>(link: bool, capture: C, entry: fn(C) -> M) -> Result<Task<M, S>, LunaticError>
where
    S: Serializer<(Process<M, S>, Tag, C)> + Serializer<M>,
{
    let (type_helper, entry) = (type_helper_wrapper::<C, M, S> as i32, entry as i32);

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
        host_api::process::inherit_spawn(
            link,
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
        let this_proc: Process<M, S> = unsafe { Process::from(this_id) };
        // Send all data to child required
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

#[export_name = "_lunatic_spawn_task_by_index"]
extern "C" fn _lunatic_spawn_task_by_index(type_helper: usize, function: usize) {
    let type_helper: fn(usize) = unsafe { std::mem::transmute(type_helper) };
    type_helper(function);
}

// Processes are equal if their UUID is equal.
impl<M, S> PartialEq for Task<M, S>
where
    S: Serializer<M>,
{
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<M, S> std::fmt::Debug for Task<M, S>
where
    S: Serializer<M>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process").field("uuid", &self.id()).finish()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::process::{sleep, spawn, spawn_link};

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
        spawn::<Task<()>, _>((), |_| {
            spawn_link::<Task<()>, _>((), |_| {
                panic!("fails");
            })
            .unwrap();
            // This process should fails too before 100ms
            sleep(Duration::from_millis(100));
        })
        .unwrap();
        sleep(Duration::from_millis(100));
    }
}
