use crate::host_api;

/// Environment configuration.
pub enum ProcessConfig {
    // ID of a configuration held by the host as a resource.
    Config(u64),
    // Indicates that the configuration should be inherited from the parent process.
    Inherit,
}

impl Drop for ProcessConfig {
    fn drop(&mut self) {
        match self {
            ProcessConfig::Config(id) => unsafe { host_api::process::drop_config(*id) },
            ProcessConfig::Inherit => (),
        }
    }
}

impl ProcessConfig {
    /// Create a new process configuration with all permissions denied.
    ///
    /// There is no memory or fuel limit set on the newly created configuration, they are not
    /// inherited from parent.
    pub fn new() -> Self {
        let id = unsafe { host_api::process::create_config() };
        Self::Config(id)
    }

    pub(crate) fn inherit() -> Self {
        Self::Inherit
    }

    /// Returns the id of the configuration resource or -1 in case it's an inherited configuration.
    pub fn id(&self) -> i64 {
        match self {
            ProcessConfig::Config(id) => *id as i64,
            ProcessConfig::Inherit => -1,
        }
    }

    /// Sets the maximum amount of memory in bytes that can be used by a process.
    ///
    /// If a process tries to allocate more memory with `memory.grow`, the instruction is going to
    /// return -1.
    pub fn set_max_memory(&mut self, max_memory: u64) {
        unsafe { host_api::process::config_set_max_memory(self.id() as u64, max_memory) };
    }

    /// Returns the maximum amount of memory in bytes.
    pub fn get_max_memory(&self) -> u64 {
        unsafe { host_api::process::config_get_max_memory(self.id() as u64) }
    }

    /// Sets the maximum amount of fuel available to the process.
    ///
    /// One unit of fuel is approximately 100k wasm instructions. If a process runs out of fuel it
    /// will trap.
    pub fn set_max_fuel(&mut self, max_fuel: u64) {
        unsafe { host_api::process::config_set_max_fuel(self.id() as u64, max_fuel) };
    }

    /// Returns the maximum amount of fuel.
    pub fn get_max_fuel(&self) -> u64 {
        unsafe { host_api::process::config_get_max_fuel(self.id() as u64) }
    }

    /// Sets the ability of a process to create their own sub-configuration.
    ///
    /// This setting can be dangerous. If a process is missing a permission, but has the
    /// possibility to create new configurations, it can spawn sub-processes using a new config
    /// that has the permission enabled.
    pub fn set_can_create_configs(&mut self, can: bool) {
        unsafe { host_api::process::config_set_can_create_configs(self.id() as u64, can as u32) };
    }

    /// Returns true if processes can create their own configurations.
    pub fn can_create_configs(&self) -> bool {
        (unsafe { host_api::process::config_can_create_configs(self.id() as u64) }) > 0
    }

    /// Sets the ability of a process to spawn sub-processes.
    pub fn set_can_spawn_processes(&mut self, can: bool) {
        unsafe { host_api::process::config_set_can_spawn_processes(self.id() as u64, can as u32) };
    }

    /// Returns true if processes can spawn sub-processes.
    pub fn can_spawn_processes(&self) -> bool {
        (unsafe { host_api::process::config_can_spawn_processes(self.id() as u64) }) > 0
    }
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self::new()
    }
}
