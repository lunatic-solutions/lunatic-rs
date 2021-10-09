use std::{fmt::Display, u128};

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

use crate::{
    error::LunaticError,
    host_api,
    mailbox::{LinkMailbox, Mailbox, TransformMailbox},
    process::{spawn_, Context, Process},
    tag::Tag,
};

/// Environment configuration
pub struct Config {
    id: u64,
}

impl Drop for Config {
    fn drop(&mut self) {
        unsafe { host_api::process::drop_config(self.id) };
    }
}

impl Config {
    /// Create a new configuration
    pub fn new(max_memory: u64, max_fuel: Option<u64>) -> Self {
        let max_fuel = max_fuel.unwrap_or(0);
        let id = unsafe { host_api::process::create_config(max_memory, max_fuel) };
        Self { id }
    }

    /// Allow a host function namespace to be used by processes spawned with this configuration.
    ///
    /// Namespaces can be exact function matches (e.g. `lunatic::error::string_size`) or just a
    /// prefix (e.g. `lunatic::error::`) matching all functions inside of the namespace.
    ///
    /// An empty string ("") is considered a prefix of **all** namespaces.
    pub fn allow_namespace(&mut self, namespace: &str) {
        unsafe { host_api::process::allow_namespace(self.id, namespace.as_ptr(), namespace.len()) };
    }

    /// Grant access to the given host directory.
    /// Returns error if host does not have access to directory.
    pub fn preopen_dir(&mut self, dir: &str) -> Result<(), LunaticError> {
        let mut error_id = 0;
        let result = unsafe {
            host_api::process::preopen_dir(
                self.id,
                dir.as_ptr(),
                dir.len(),
                &mut error_id as *mut u64,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(LunaticError::from(error_id))
        }
    }

    /// Add a WebAssembly module as a plugin to this configuration.
    pub fn add_plugin(&mut self, plugin: &[u8]) -> Result<(), LunaticError> {
        let mut error_id = 0;
        let result = unsafe {
            host_api::process::add_plugin(
                self.id,
                plugin.as_ptr(),
                plugin.len(),
                &mut error_id as *mut u64,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(LunaticError::from(error_id))
        }
    }
}

#[derive(Error, Debug)]
pub enum RegistryError {
    IncorrectSemver,
    IncorrectQuery,
    NotFound,
}

impl Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Environments can define characteristics of processes that are spawned into it.
pub struct Environment {
    id: u64,
}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe { host_api::process::drop_environment(self.id) };
    }
}

impl Environment {
    pub(crate) fn from(id: u64) -> Self {
        Environment { id }
    }

    /// Create a new environment from a configurationS
    pub fn new(config: Config) -> Result<Self, LunaticError> {
        let mut env_or_error_id = 0;
        let result = unsafe {
            host_api::process::create_environment(config.id, &mut env_or_error_id as *mut u64)
        };
        if result == 0 {
            Ok(Self {
                id: env_or_error_id,
            })
        } else {
            Err(LunaticError::from(env_or_error_id))
        }
    }

    /// Add a WebAssembly module to the environment.
    pub fn add_module(&mut self, module: &[u8]) -> Result<Module, LunaticError> {
        let mut module_or_error_id = 0;
        let result = unsafe {
            host_api::process::add_module(
                self.id,
                module.as_ptr(),
                module.len(),
                &mut module_or_error_id as *mut u64,
            )
        };
        if result == 0 {
            Ok(Module {
                id: module_or_error_id,
            })
        } else {
            Err(LunaticError::from(module_or_error_id))
        }
    }

    /// Add the module that is being currently executed to the environment.
    pub fn add_this_module(&mut self) -> Result<ThisModule, LunaticError> {
        let mut module_or_error_id = 0;
        let result = unsafe {
            host_api::process::add_this_module(self.id, &mut module_or_error_id as *mut u64)
        };
        if result == 0 {
            Ok(ThisModule {
                id: module_or_error_id,
            })
        } else {
            Err(LunaticError::from(module_or_error_id))
        }
    }

    /// Register a process under a specific name & version in the environment.
    ///
    /// The version must be in a correct semver format. If a process was registered under the same
    /// name and exactly same version, it will be overwritten.
    pub fn register<T: Serialize + DeserializeOwned>(
        &self,
        name: &str,
        version: &str,
        process: Process<T>,
    ) -> Result<(), RegistryError> {
        match unsafe {
            host_api::process::register(
                name.as_ptr(),
                name.len(),
                version.as_ptr(),
                version.len(),
                self.id,
                process.id,
            )
        } {
            0 => Ok(()),
            1 => Err(RegistryError::IncorrectSemver),
            _ => unreachable!(),
        }
    }

    /// Unregister a process from the environment
    pub fn unregister<T: Serialize + DeserializeOwned>(
        &self,
        name: &str,
        version: &str,
    ) -> Result<(), RegistryError> {
        match unsafe {
            host_api::process::unregister(
                name.as_ptr(),
                name.len(),
                version.as_ptr(),
                version.len(),
                self.id,
            )
        } {
            0 => Ok(()),
            1 => Err(RegistryError::IncorrectSemver),
            2 => Err(RegistryError::NotFound),
            _ => unreachable!(),
        }
    }
}

/// Returns a process that was registered inside the environment that the caller belongs to.
///
/// The query can be be an exact version or follow semver query rules (e.g. "^1.1").
pub fn lookup<T: Serialize + DeserializeOwned>(
    name: &str,
    query: &str,
) -> Result<Option<Process<T>>, RegistryError> {
    let mut process_id: u64 = 0;
    match unsafe {
        host_api::process::lookup(
            name.as_ptr(),
            name.len(),
            query.as_ptr(),
            query.len(),
            &mut process_id as *mut u64,
        )
    } {
        0 => Ok(Some(Process::from(process_id))),
        1 => Err(RegistryError::IncorrectSemver),
        2 => Ok(None),
        _ => unreachable!(),
    }
}

/// A compiled instance of a WebAssembly module.
///
/// Modules belong to [`Environments`](Environment) and processes spawned from the modules will
/// have characteristics defined by the [`Environment`].
///
/// Creating a module will also JIT compile it, this can be a compute intensive tasks.
pub struct Module {
    id: u64,
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe { host_api::process::drop_module(self.id) };
    }
}

impl Module {
    /// Spawn a new process and use `function` as the entry point. If the function takes arguments
    /// the passed in `params` need to exactly match their types.
    pub fn spawn<T: Serialize + DeserializeOwned>(
        &self,
        function: &str,
        params: &[Param],
    ) -> Result<Process<T>, LunaticError> {
        let mut process_or_error_id = 0;
        let params: Vec<u8> = params_to_vec(params);
        let result = unsafe {
            host_api::process::spawn(
                0,
                self.id,
                function.as_ptr(),
                function.len(),
                params.as_ptr(),
                params.len(),
                &mut process_or_error_id as *mut u64,
            )
        };

        if result == 0 {
            Ok(Process::from(process_or_error_id))
        } else {
            Err(LunaticError::from(process_or_error_id))
        }
    }

    /// Spawn a new process and link it to the current one.
    pub fn spawn_link<T, P, M>(
        &self,
        mailbox: M,
        function: &str,
        params: &[Param],
    ) -> Result<(Process<T>, LinkMailbox<P>), LunaticError>
    where
        T: Serialize + DeserializeOwned,
        P: Serialize + DeserializeOwned,
        M: TransformMailbox<P>,
    {
        let mailbox = mailbox.catch_link_panic();
        let mut process_or_error_id = 0;
        let params: Vec<u8> = params_to_vec(params);
        let result = unsafe {
            host_api::process::spawn(
                1,
                self.id,
                function.as_ptr(),
                function.len(),
                params.as_ptr(),
                params.len(),
                &mut process_or_error_id as *mut u64,
            )
        };

        if result == 0 {
            Ok((Process::from(process_or_error_id), mailbox))
        } else {
            Err(LunaticError::from(process_or_error_id))
        }
    }
}

/// A pointer to the current module.
///
/// This type is useful because it allows us to spawn existing functions by reference into a new
/// environment. This is only possible if we are running inside the module we are spawning the
/// processes from, otherwise we could not reference them by table id.
pub struct ThisModule {
    id: u64,
}

impl Drop for ThisModule {
    fn drop(&mut self) {
        unsafe { host_api::process::drop_module(self.id) };
    }
}

impl ThisModule {
    /// Spawns a new process from a function.
    ///
    /// - `function` is the starting point of the new process. The new process doesn't share
    ///   memory with its parent, because of this the function can't capture anything from parents.
    pub fn spawn<T: Serialize + DeserializeOwned>(
        &self,
        function: fn(Mailbox<T>),
    ) -> Result<Process<T>, LunaticError> {
        // LinkMailbox<T> & Mailbox<T> are marker types and it's safe to cast to Mailbox<T> here if we
        //  set the `link` argument to `false`.
        let function = unsafe { std::mem::transmute(function) };
        spawn_(Some(self.id), None, Context::<(), _>::Without(function))
    }

    /// Spawns a new process from a function and links it to the parent.
    ///
    /// - `function` is the starting point of the new process. The new process doesn't share
    ///   memory with its parent, because of this the function can't capture anything from parents.
    pub fn spawn_link<T, P, M>(
        &self,
        mailbox: M,
        function: fn(Mailbox<T>),
    ) -> Result<(Process<T>, Tag, LinkMailbox<P>), LunaticError>
    where
        T: Serialize + DeserializeOwned,
        P: Serialize + DeserializeOwned,
        M: TransformMailbox<P>,
    {
        let mailbox = mailbox.catch_link_panic();
        let tag = Tag::new();
        let proc = spawn_(
            Some(self.id),
            Some(tag),
            Context::<(), _>::Without(function),
        )?;
        Ok((proc, tag, mailbox))
    }

    /// Spawns a new process from a function and links it to the parent.
    ///
    /// - `function` is the starting point of the new process. The new process doesn't share
    ///   memory with its parent, because of this the function can't capture anything from parents.
    ///
    /// If the linked process dies, the parent is going to die too.
    pub fn spawn_link_unwrap<T, P, M>(
        &self,
        mailbox: M,
        function: fn(Mailbox<T>),
    ) -> Result<(Process<T>, Mailbox<P>), LunaticError>
    where
        T: Serialize + DeserializeOwned,
        P: Serialize + DeserializeOwned,
        M: TransformMailbox<P>,
    {
        let mailbox = mailbox.panic_if_link_panics();
        let proc = spawn_(
            Some(self.id),
            Some(Tag::new()),
            Context::<(), _>::Without(function),
        )?;
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
        &self,
        context: C,
        function: fn(C, Mailbox<T>),
    ) -> Result<Process<T>, LunaticError> {
        // LinkMailbox<T> & Mailbox<T> are marker types and it's safe to cast to Mailbox<T> here if we
        //  set the `link` argument to `false`.
        let function = unsafe { std::mem::transmute(function) };
        spawn_(Some(self.id), None, Context::With(function, context))
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
        &self,
        mailbox: M,
        context: C,
        function: fn(C, Mailbox<T>),
    ) -> Result<(Process<T>, Tag, LinkMailbox<P>), LunaticError>
    where
        C: Serialize + DeserializeOwned,
        T: Serialize + DeserializeOwned,
        P: Serialize + DeserializeOwned,
        M: TransformMailbox<P>,
    {
        let mailbox = mailbox.catch_link_panic();
        let tag = Tag::new();
        let proc = spawn_(Some(self.id), Some(tag), Context::With(function, context))?;
        Ok((proc, tag, mailbox))
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
        &self,
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
        let proc = spawn_(
            Some(self.id),
            Some(Tag::new()),
            Context::With(function, context),
        )?;
        Ok((proc, mailbox))
    }
}

pub enum Param {
    I32(i32),
    I64(i64),
    V128(u128),
}

pub(crate) fn params_to_vec(params: &[Param]) -> Vec<u8> {
    let mut result = Vec::with_capacity(params.len() * 17);
    params.iter().for_each(|param| match param {
        Param::I32(value) => {
            result.push(0x7F);
            result.extend((*value as u128).to_le_bytes())
        }
        Param::I64(value) => {
            result.push(0x7E);
            result.extend((*value as u128).to_le_bytes())
        }
        Param::V128(value) => {
            result.push(0x7B);
            result.extend((*value).to_le_bytes())
        }
    });
    result
}
