use crate::{host, LunaticError};

/// Process configurations determine permissions of processes.
///
/// The functions `spawn_config` & `spawn_link_config` can be used to create
/// processes with a specific configuration.
pub struct ProcessConfig(ProcessConfigType);

enum ProcessConfigType {
    /// ID of a configuration held by the host as a resource.
    Config(u64),
    /// Indicates that the configuration should be inherited from the parent
    /// process.
    Inherit,
}

impl Drop for ProcessConfigType {
    fn drop(&mut self) {
        match self {
            ProcessConfigType::Config(id) => unsafe { host::api::process::drop_config(*id) },
            ProcessConfigType::Inherit => (),
        }
    }
}

impl std::fmt::Debug for ProcessConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ProcessConfigType::Config(_) => f
                .debug_struct("ProcessConfig")
                .field("max_memory", &self.get_max_memory())
                .field("max_fuel", &self.get_max_fuel())
                .field("can_compile_modules", &self.can_compile_modules())
                .field("can_create_configs", &self.can_create_configs())
                .field("can_spawn_processes", &self.can_spawn_processes())
                .finish(),
            ProcessConfigType::Inherit => f.debug_struct("ProcessConfig::Inherit").finish(),
        }
    }
}

impl ProcessConfig {
    /// Create a new process configuration with all permissions denied.
    ///
    /// There is no memory or fuel limit set on the newly created configuration,
    /// they are not inherited from parent.
    pub fn new() -> Result<Self, LunaticError> {
        match unsafe { host::api::process::create_config() } {
            -1 => Err(LunaticError::PermissionDenied),
            id => Ok(Self(ProcessConfigType::Config(id as u64))),
        }
    }

    pub(crate) fn inherit() -> Self {
        Self(ProcessConfigType::Inherit)
    }

    /// Returns the id of the configuration resource or -1 in case it's an
    /// inherited configuration.
    pub fn id(&self) -> i64 {
        match self.0 {
            ProcessConfigType::Config(id) => id as i64,
            ProcessConfigType::Inherit => -1,
        }
    }

    /// Sets the maximum amount of memory in bytes that can be used by a
    /// process.
    ///
    /// If a process tries to allocate more memory with `memory.grow`, the
    /// instruction is going to return -1.
    pub fn set_max_memory(&mut self, max_memory: u64) {
        unsafe { host::api::process::config_set_max_memory(self.id() as u64, max_memory) };
    }

    /// Returns the maximum amount of memory in bytes.
    pub fn get_max_memory(&self) -> u64 {
        unsafe { host::api::process::config_get_max_memory(self.id() as u64) }
    }

    /// Sets the maximum amount of fuel available to the process.
    ///
    /// One unit of fuel is approximately 100k wasm instructions. If a process
    /// runs out of fuel it will trap.
    pub fn set_max_fuel(&mut self, max_fuel: u64) {
        unsafe { host::api::process::config_set_max_fuel(self.id() as u64, max_fuel) };
    }

    /// Returns the maximum amount of fuel.
    pub fn get_max_fuel(&self) -> u64 {
        unsafe { host::api::process::config_get_max_fuel(self.id() as u64) }
    }

    /// Sets the ability of a process to compile WebAssembly modules.
    pub fn set_can_compile_modules(&mut self, can: bool) {
        unsafe { host::api::process::config_set_can_compile_modules(self.id() as u64, can as u32) };
    }

    /// Returns true if processes can compile WebAssembly modules.
    pub fn can_compile_modules(&self) -> bool {
        (unsafe { host::api::process::config_can_compile_modules(self.id() as u64) }) > 0
    }

    /// Sets the ability of a process to create their own sub-configuration.
    ///
    /// This setting can be dangerous. If a process is missing a permission, but
    /// has the possibility to create new configurations, it can spawn
    /// sub-processes using a new config that has the permission enabled.
    pub fn set_can_create_configs(&mut self, can: bool) {
        unsafe { host::api::process::config_set_can_create_configs(self.id() as u64, can as u32) };
    }

    /// Returns true if processes can create their own configurations.
    pub fn can_create_configs(&self) -> bool {
        (unsafe { host::api::process::config_can_create_configs(self.id() as u64) }) > 0
    }

    /// Sets the ability of a process to spawn sub-processes.
    pub fn set_can_spawn_processes(&mut self, can: bool) {
        unsafe { host::api::process::config_set_can_spawn_processes(self.id() as u64, can as u32) };
    }

    /// Returns true if processes can spawn sub-processes.
    pub fn can_spawn_processes(&self) -> bool {
        (unsafe { host::api::process::config_can_spawn_processes(self.id() as u64) }) > 0
    }

    /// Adds environment variable.
    pub fn add_environment_variable(&mut self, key: &str, value: &str) {
        unsafe {
            host::api::wasi::config_add_environment_variable(
                self.id() as u64,
                key.as_ptr(),
                key.len(),
                value.as_ptr(),
                value.len(),
            )
        }
    }

    /// Adds command line argument.
    pub fn add_command_line_argument(&mut self, argument: &str) {
        unsafe {
            host::api::wasi::config_add_command_line_argument(
                self.id() as u64,
                argument.as_ptr(),
                argument.len(),
            )
        }
    }

    #[rustversion::before(1.67)]
    /// Mark a directory as pre-opened.
    ///
    /// This API is only available in Rust 1.66 and below. See:
    /// - https://github.com/rust-lang/rust/issues/107635
    /// - https://github.com/rust-lang/rust/pull/108097
    pub fn preopen_dir(&mut self, dir: &str) {
        unsafe { host::api::wasi::config_preopen_dir(self.id() as u64, dir.as_ptr(), dir.len()) }
    }
}
