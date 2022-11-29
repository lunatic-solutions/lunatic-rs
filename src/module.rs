use std::u128;

use serde::{Deserialize, Serialize};

use crate::error::LunaticError;
use crate::host::api::distributed::node_id;
use crate::host::{self};
use crate::serializer::Serializer;
use crate::{Process, ProcessConfig, Tag};

/// A compiled instance of a WebAssembly module.
///
/// Creating a module will also JIT compile it, this can be a compute-intensive
/// tasks.
pub enum WasmModule {
    Module(u64),
    Inherit,
}

impl Drop for WasmModule {
    fn drop(&mut self) {
        match self {
            WasmModule::Module(id) => {
                unsafe { host::api::process::drop_module(*id) };
            }
            WasmModule::Inherit => (),
        }
    }
}

impl Serialize for WasmModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let index = unsafe { host::api::message::push_module(self.id() as u64) };
        serializer.serialize_u64(index)
    }
}

impl<'de> Deserialize<'de> for WasmModule {
    fn deserialize<D>(deserializer: D) -> std::result::Result<WasmModule, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let index = Deserialize::deserialize(deserializer)?;
        let id = unsafe { host::api::message::take_module(index) };
        Ok(WasmModule::Module(id))
    }
}

impl WasmModule {
    /// Compiles a WebAssembly module.
    ///
    /// Once a module is compiled, functions like [`spawn`](Self::spawn) can be
    /// used to spawn new processes from it.
    pub fn new(data: &[u8]) -> Result<Self, LunaticError> {
        let mut module_or_error_id: u64 = 0;

        let result = unsafe {
            host::api::process::compile_module(
                data.as_ptr(),
                data.len(),
                &mut module_or_error_id as *mut u64,
            )
        };
        if result == -1 {
            Err(LunaticError::PermissionDenied)
        } else if result != 0 {
            Err(LunaticError::from(module_or_error_id))
        } else {
            Ok(WasmModule::Module(module_or_error_id))
        }
    }

    pub(crate) fn inherit() -> Self {
        Self::Inherit
    }

    /// Returns the id of the module resource or -1 in case it's an inherited
    /// module.
    pub fn id(&self) -> i64 {
        match self {
            WasmModule::Module(id) => *id as i64,
            WasmModule::Inherit => -1,
        }
    }

    /// Spawn a new process and use `function` as the entry point. If the
    /// function takes arguments the passed in `params` need to exactly
    /// match their types.
    pub fn spawn<M, S>(
        &self,
        function: &str,
        params: &[Param],
    ) -> Result<Process<M, S>, LunaticError>
    where
        S: Serializer<M>,
    {
        self.spawn_(function, params, None, None)
    }

    /// Spawn a new process with a configuration, and use `function` as the
    /// entry point. If the function takes arguments the passed in `params`
    /// need to exactly match their types.
    pub fn spawn_config<M, S>(
        &self,
        function: &str,
        params: &[Param],
        config: &ProcessConfig,
    ) -> Result<Process<M, S>, LunaticError>
    where
        S: Serializer<M>,
    {
        self.spawn_(function, params, None, Some(config))
    }

    /// Spawn a new process and link it to the current one with the `tag`.
    pub fn spawn_link<M, S>(
        &self,
        function: &str,
        params: &[Param],
        tag: Tag,
    ) -> Result<Process<M, S>, LunaticError>
    where
        S: Serializer<M>,
    {
        self.spawn_(function, params, Some(tag), None)
    }

    /// Spawn a new process with a configuration, and link it to the current one
    /// with the `tag`.
    pub fn spawn_link_config<M, S>(
        &self,
        function: &str,
        params: &[Param],
        config: &ProcessConfig,
        tag: Tag,
    ) -> Result<Process<M, S>, LunaticError>
    where
        S: Serializer<M>,
    {
        self.spawn_(function, params, Some(tag), Some(config))
    }

    fn spawn_<M, S>(
        &self,
        function: &str,
        params: &[Param],
        link: Option<Tag>,
        config: Option<&ProcessConfig>,
    ) -> Result<Process<M, S>, LunaticError>
    where
        S: Serializer<M>,
    {
        let link = match link {
            Some(tag) => tag.id(),
            None => 0,
        };
        let config_id = config.map_or_else(|| ProcessConfig::inherit().id(), |config| config.id());
        let mut process_or_error_id = 0;
        let params: Vec<u8> = params_to_vec(params);
        let result = unsafe {
            host::api::process::spawn(
                link,
                config_id,
                self.id(),
                function.as_ptr(),
                function.len(),
                params.as_ptr(),
                params.len(),
                &mut process_or_error_id as *mut u64,
            )
        };

        if result == 0 {
            Ok(unsafe { Process::new(node_id(), process_or_error_id) })
        } else {
            Err(LunaticError::from(process_or_error_id))
        }
    }
}

pub enum Param {
    I32(i32),
    I64(i64),
    V128(i128),
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
