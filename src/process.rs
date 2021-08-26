use std::{
    cell::UnsafeCell,
    fmt::{self, Debug},
    marker::PhantomData,
    mem::transmute,
};

use crate::{
    environment::{params_to_vec, Param},
    error::LunaticError,
    host_api::{self, message, process},
    mailbox::{LinkMailbox, Mailbox, MessageRw, TransformMailbox},
    Environment,
};

use serde::{
    de::{self, DeserializeOwned, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

/// A sandboxed computation.
///
/// Processes are fundamental building blocks of Lunatic applications. Each of them has their own
/// memory space. The only way for processes to interact is trough [`Serialize + DeserializeOwned`]
/// passing.
///
/// ### Safety:
/// It's not safe to use mutable `static` variables to share data between processes, because each
/// of them is going to see a separate heap and a unique `static` variable.
pub struct Process<T: Serialize + DeserializeOwned> {
    pub(crate) id: u64,
    // If the process handle is serialized it will be removed from our resources, so we can't call
    // `drop_process()` anymore on it.
    consumed: UnsafeCell<bool>,
    _phantom: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> Debug for Process<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host_api::process::id(self.id, &mut uuid as *mut [u8; 16]) };
        f.debug_struct("Process")
            .field("uuid", &u128::from_le_bytes(uuid))
            .finish()
    }
}

impl<T: Serialize + DeserializeOwned> Clone for Process<T> {
    fn clone(&self) -> Self {
        let id = unsafe { host_api::process::clone_process(self.id) };
        Process::from(id)
    }
}

impl<T: Serialize + DeserializeOwned> Drop for Process<T> {
    fn drop(&mut self) {
        // Only drop process if it's not already consumed
        if unsafe { !*self.consumed.get() } {
            unsafe { process::drop_process(self.id) };
        }
    }
}
impl<T: Serialize + DeserializeOwned> Serialize for Process<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Mark process as consumed
        unsafe { *self.consumed.get() = true };

        let index = unsafe { host_api::message::push_process(self.id) };
        serializer.serialize_u64(index)
    }
}
struct ProcessVisitor<T> {
    _phantom: PhantomData<T>,
}
impl<'de, T: Serialize + DeserializeOwned> Visitor<'de> for ProcessVisitor<T> {
    type Value = Process<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an u64 index")
    }

    fn visit_u64<E>(self, index: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let id = unsafe { host_api::message::take_process(index) };
        Ok(Process::from(id))
    }
}

impl<'de, T: Serialize + DeserializeOwned> Deserialize<'de> for Process<T> {
    fn deserialize<D>(deserializer: D) -> Result<Process<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(ProcessVisitor {
            _phantom: PhantomData {},
        })
    }
}

impl<T: Serialize + DeserializeOwned> Process<T> {
    pub(crate) fn from(id: u64) -> Self {
        Process {
            id,
            consumed: UnsafeCell::new(false),
            _phantom: PhantomData,
        }
    }

    /// Send message to process.
    pub fn send(&self, value: T) {
        self.send_(None, value)
    }

    fn send_(&self, tag: Option<i64>, value: T) {
        let tag = match tag {
            Some(tag) => tag,
            None => 0,
        };
        // Create new message
        unsafe { message::create_data(tag, 0) };
        // During serialization resources will add themself to the message
        rmp_serde::encode::write(&mut MessageRw {}, &value).unwrap();
        // Send it
        unsafe { message::send(self.id) };
    }

    /// Links the current process with another one.
    pub fn link(&self) {
        unsafe { process::link(0, self.id) };
    }

    /// Unlinks the current process from another one.
    pub fn unlink(&self) {
        unsafe { process::unlink(self.id) };
    }
}

/// Returns a handle to the current process.
pub fn this<T: Serialize + DeserializeOwned, U: TransformMailbox<T>>(
    mailbox: U,
) -> (Process<T>, U) {
    let id = unsafe { process::this() };
    (Process::from(id), mailbox)
}

/// Returns a handle to the current environment.
pub fn this_env() -> Environment {
    let id = unsafe { process::this_env() };
    Environment::from(id)
}

/// Spawns a new process from a function.
///
/// - `function` is the starting point of the new process. The new process doesn't share
///   memory with its parent, because of this the function can't capture anything from parents.
pub fn spawn<T: Serialize + DeserializeOwned>(
    function: fn(Mailbox<T>),
) -> Result<Process<T>, LunaticError> {
    // LinkMailbox<T> & Mailbox<T> are marker types and it's safe to cast to Mailbox<T> here if we
    // set the `link` argument to `false`.
    let function = unsafe { transmute(function) };
    spawn_(None, false, Context::<(), _>::Without(function))
}

/// Spawns a new process from a function and links it to the parent.
///
/// - `function` is the starting point of the new process. The new process doesn't share
///   memory with its parent, because of this the function can't capture anything from parents.
pub fn spawn_link<T, P, M>(
    mailbox: M,
    function: fn(Mailbox<T>),
) -> Result<(Process<T>, LinkMailbox<P>), LunaticError>
where
    T: Serialize + DeserializeOwned,
    P: Serialize + DeserializeOwned,
    M: TransformMailbox<P>,
{
    let mailbox = mailbox.catch_link_panic();
    let proc = spawn_(None, true, Context::<(), _>::Without(function))?;
    Ok((proc, mailbox))
}

/// Spawns a new process from a function and links it to the parent.
///
/// - `function` is the starting point of the new process. The new process doesn't share
///   memory with its parent, because of this the function can't capture anything from parents.
///
/// If the linked process dies, the parent is going to die too.
pub fn spawn_link_unwrap<T, P, M>(
    mailbox: M,
    function: fn(Mailbox<T>),
) -> Result<(Process<T>, Mailbox<P>), LunaticError>
where
    T: Serialize + DeserializeOwned,
    P: Serialize + DeserializeOwned,
    M: TransformMailbox<P>,
{
    let mailbox = mailbox.panic_if_link_panics();
    let proc = spawn_(None, true, Context::<(), _>::Without(function))?;
    Ok((proc, mailbox))
}

/// Spawns a new process from a function and context.
///
/// - `context` is  data that we want to pass to the newly spawned process. It needs to impl.
///    the [`Serialize + DeserializeOwned`] trait.
///
/// - `function` is the starting point of the new process. The new process doesn't share
///   memory with its parent, because of this the function can't capture anything from parents.
///   The first argument of this function is going to be the received `context`.
pub fn spawn_with<C: Serialize + DeserializeOwned, T: Serialize + DeserializeOwned>(
    context: C,
    function: fn(C, Mailbox<T>),
) -> Result<Process<T>, LunaticError> {
    // LinkMailbox<T> & Mailbox<T> are marker types and it's safe to cast to Mailbox<T> here if we
    //  set the `link` argument to `false`.
    let function = unsafe { transmute(function) };
    spawn_(None, false, Context::With(function, context))
}

/// Spawns a new process from a function and context, and links it to the parent.
///
/// - `context` is  data that we want to pass to the newly spawned process. It needs to impl.
///    the [`Serialize + DeserializeOwned`] trait.
///
/// - `function` is the starting point of the new process. The new process doesn't share
///   memory with its parent, because of this the function can't capture anything from parents.
///   The first argument of this function is going to be the received `context`.
pub fn spawn_link_with<C, T, P, M>(
    mailbox: M,
    context: C,
    function: fn(C, Mailbox<T>),
) -> Result<(Process<T>, LinkMailbox<P>), LunaticError>
where
    C: Serialize + DeserializeOwned,
    T: Serialize + DeserializeOwned,
    P: Serialize + DeserializeOwned,
    M: TransformMailbox<P>,
{
    let mailbox = mailbox.catch_link_panic();
    let proc = spawn_(None, true, Context::With(function, context))?;
    Ok((proc, mailbox))
}

/// Spawns a new process from a function and context, and links it to the parent.
///
/// - `context` is  data that we want to pass to the newly spawned process. It needs to impl.
///    the [`Serialize + DeserializeOwned`] trait.
///
/// - `function` is the starting point of the new process. The new process doesn't share
///   memory with its parent, because of this the function can't capture anything from parents.
///   The first argument of this function is going to be the received `context`.
///
/// If the linked process dies, the parent is going to die too.
pub fn spawn_link_unwrap_with<C, T, P, M>(
    mailbox: M,
    context: C,
    function: fn(C, Mailbox<T>),
) -> Result<(Process<T>, Mailbox<P>), LunaticError>
where
    C: Serialize + DeserializeOwned,
    T: Serialize + DeserializeOwned,
    P: Serialize + DeserializeOwned,
    M: TransformMailbox<P>,
{
    let mailbox = mailbox.panic_if_link_panics();
    let proc = spawn_(None, true, Context::With(function, context))?;
    Ok((proc, mailbox))
}

pub(crate) enum Context<C: Serialize + DeserializeOwned, T: Serialize + DeserializeOwned> {
    With(fn(C, Mailbox<T>), C),
    Without(fn(Mailbox<T>)),
}

// If `module_id` is None it will use the current module & environment.
pub(crate) fn spawn_<C: Serialize + DeserializeOwned, T: Serialize + DeserializeOwned>(
    module_id: Option<u64>,
    link: bool,
    context: Context<C, T>,
) -> Result<Process<T>, LunaticError> {
    // Spawning a new process from  the same module is a delicate undertaking.
    // First of all, the WebAssembly spec only allows us to call exported functions from a module
    // Therefore we define a module export under the name `_lunatic_spawn_by_index`. This global
    // function will get 2 arguments:
    // * A type helper function: `type_helper_wrapper_*`
    // * The function we want to use as an entry point: `function`
    // It's obvious why we need the entry function, but what is a type helper function? The entry
    // entry function contains 2 generic types, one for the context and one for messages, but the
    // `_lunatic_spawn_by_index` one can't be generic. That's why we use the type helper, to let
    // us wrap the call to the entry function into the right type signature.

    let (type_helper, func) = match context {
        Context::With(func, _) => (type_helper_wrapper_context::<C, T> as usize, func as usize),
        Context::Without(func) => (type_helper_wrapper::<T> as usize, func as usize),
    };
    let params = params_to_vec(&[Param::I32(type_helper as i32), Param::I32(func as i32)]);
    let mut id = 0;
    let func = "_lunatic_spawn_by_index";
    let result = unsafe {
        match module_id {
            Some(module_id) => host_api::process::spawn(
                if link { 1 } else { 0 },
                module_id,
                func.as_ptr(),
                func.len(),
                params.as_ptr(),
                params.len(),
                &mut id,
            ),
            None => host_api::process::inherit_spawn(
                if link { 1 } else { 0 },
                func.as_ptr(),
                func.len(),
                params.as_ptr(),
                params.len(),
                &mut id,
            ),
        }
    };
    if result == 0 {
        match context {
            // If context exists, send it as first message to the new process
            Context::With(_, context) => {
                let self_ = Process {
                    id,
                    consumed: UnsafeCell::new(false),
                    _phantom: PhantomData,
                };
                self_.send(context);
                // Processes can only receive one type of messages, but to pass in the context we pretend
                // for the first message that our process is receiving messages of type `C`.
                Ok(unsafe { transmute(self_) })
            }
            Context::Without(_) => Ok(Process {
                id,
                consumed: UnsafeCell::new(false),
                _phantom: PhantomData,
            }),
        }
    } else {
        Err(LunaticError::from(id))
    }
}

/// Suspends the current process for `milliseconds`.
pub fn sleep(milliseconds: u64) {
    unsafe { host_api::process::sleep_ms(milliseconds) };
}

// Type helper
fn type_helper_wrapper<T: Serialize + DeserializeOwned>(function: usize) {
    let mailbox = unsafe { Mailbox::new() };
    let function: fn(Mailbox<T>) = unsafe { transmute(function) };
    function(mailbox);
}

// Type helper with context
fn type_helper_wrapper_context<C: Serialize + DeserializeOwned, T: Serialize + DeserializeOwned>(
    function: usize,
) {
    let context = unsafe { Mailbox::new() }.receive().unwrap();
    let mailbox = unsafe { Mailbox::new() };
    let function: fn(C, Mailbox<T>) = unsafe { transmute(function) };
    function(context, mailbox);
}

#[export_name = "_lunatic_spawn_by_index"]
extern "C" fn _lunatic_spawn_by_index(type_helper: usize, function: usize) {
    let type_helper: fn(usize) = unsafe { transmute(type_helper) };
    type_helper(function);
}
