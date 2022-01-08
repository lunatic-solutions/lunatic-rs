use std::{fmt::Display, u128};

use thiserror::Error;

use crate::{
    error::LunaticError,
    host_api,
    process::{IntoProcess, IntoProcessLink, Process},
    serializer::Serializer,
    Resource,
};

/// Environment configuration.
pub struct Config {
    id: u64,
}

impl Drop for Config {
    fn drop(&mut self) {
        unsafe { host_api::process::drop_config(self.id) };
    }
}

impl Config {
    /// Create a new environment configuration.
    ///
    /// * **max_memory** - The maximum amount of memory in **bytes** that processes spawned into
    ///                    the environment can use. This limitation is **per process**.
    /// * **max_fuel**   - The maximum amount of fuel (expressed in units of 100k instructions)
    ///                    that processes can use. Once a process consumes all fuel it will trap.
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
    ///
    /// Returns error if the currently running process does not have access to directory.
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
///
/// The `Environment` is configured through a [`Config`] struct.
///
/// # Example
///
/// ```
/// // Create a new environment where processes can use maximum 17 Wasm pages of
/// // memory (17 * 64KB) & 1 compute unit of instructions (~=100k CPU cycles)
/// let mut config = Config::new(1_200_000, Some(1));
/// // Allow all host functions
/// config.allow_namespace("");
/// let mut env = Environment::new(config).unwrap();
/// // Add the currently running module to the environment.
/// let module = env.add_this_module().unwrap();
///
/// // This process will fail because it uses too much memory
/// module
///     .spawn::<AsyncTask, _>((), |_| {
///         vec![0; 150_000];
///     })
///     .unwrap();
///
///  // This process will fail because it uses too much compute
/// module.spawn::<AsyncTask, _>((), |_| loop {}).unwrap();
/// ```
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

    /// Create a new environment from a configuration.
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

    /// Create a new environment on a remote node.
    pub fn new_remote(node_name: &str, config: Config) -> Result<Self, LunaticError> {
        let mut env_or_error_id = 0;
        let result = unsafe {
            host_api::process::create_remote_environment(
                config.id,
                node_name.as_ptr(),
                node_name.len(),
                &mut env_or_error_id as *mut u64,
            )
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

    /// Add the currently running module to the environment.
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
    /// name and version, it will be overwritten.
    pub fn register<M, S>(
        &self,
        name: &str,
        version: &str,
        process: Process<M, S>,
    ) -> Result<(), RegistryError>
    where
        S: Serializer<M>,
    {
        match unsafe {
            host_api::process::register(
                name.as_ptr(),
                name.len(),
                version.as_ptr(),
                version.len(),
                self.id,
                process.id(),
            )
        } {
            0 => Ok(()),
            1 => Err(RegistryError::IncorrectSemver),
            _ => unreachable!(),
        }
    }

    /// Unregister a process from the environment
    pub fn unregister(&self, name: &str, version: &str) -> Result<(), RegistryError> {
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
pub fn lookup<M, S>(name: &str, query: &str) -> Result<Option<Process<M, S>>, RegistryError>
where
    S: Serializer<M>,
{
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
        0 => Ok(Some(unsafe { Process::from(process_id) })),
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
    pub fn spawn<M, S>(
        &self,
        function: &str,
        params: &[Param],
    ) -> Result<Process<M, S>, LunaticError>
    where
        S: Serializer<M>,
    {
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
            Ok(unsafe { Process::from(process_or_error_id) })
        } else {
            Err(LunaticError::from(process_or_error_id))
        }
    }

    /// Spawn a new process and link it to the current one.
    pub fn spawn_link<M, S>(
        &self,
        function: &str,
        params: &[Param],
    ) -> Result<Process<M, S>, LunaticError>
    where
        S: Serializer<M>,
    {
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
            Ok(unsafe { Process::from(process_or_error_id) })
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
    /// Spawns a new process.
    ///
    /// TODO: Research if `spawn` and `spawn_link` could move the whole spawning procedure into the new
    ///       async task, so that there can't be any failure during the host call and we can return `T`
    ///       instead of a `Result` here.
    pub fn spawn<T, C>(&self, capture: C, handler: T::Handler) -> Result<T, LunaticError>
    where
        T: IntoProcess<C>,
    {
        <T as IntoProcess<C>>::spawn(Some(self.id), capture, handler)
    }

    /// Spawns a new process and link it to the parent.
    ///
    /// TODO: Research if `spawn` and `spawn_link` could move the whole spawning procedure into the new
    ///       async task, so that there can't be any failure during the host call and we can return `T`
    ///       instead of a `Result` here.
    pub fn spawn_link<T, C>(&self, capture: C, handler: T::Handler) -> Result<T, LunaticError>
    where
        T: IntoProcessLink<C>,
    {
        <T as IntoProcessLink<C>>::spawn_link(Some(self.id), capture, handler)
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
