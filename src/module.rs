use std::u128;

use crate::{error::LunaticError, host, serializer::Serializer, Process, Resource};

/// A compiled instance of a WebAssembly module.
///
/// Creating a module will also JIT compile it, this can be a compute intensive tasks.
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

impl WasmModule {
    /// Compiles a WebAssembly module.
    ///
    /// Once a module is compiled, functions like [`spawn`](Self::spawn) can be used to spawn new
    /// processes from it.
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

    /// Returns the id of the module resource or -1 in case it's an inherited module.
    pub fn id(&self) -> i64 {
        match self {
            WasmModule::Module(id) => *id as i64,
            WasmModule::Inherit => -1,
        }
    }

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
            host::api::process::spawn(
                0,
                -1,
                self.id(),
                function.as_ptr(),
                function.len(),
                params.as_ptr(),
                params.len(),
                &mut process_or_error_id as *mut u64,
            )
        };

        if result == 0 {
            Ok(unsafe { Process::from_id(process_or_error_id) })
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
            host::api::process::spawn(
                1,
                -1,
                self.id(),
                function.as_ptr(),
                function.len(),
                params.as_ptr(),
                params.len(),
                &mut process_or_error_id as *mut u64,
            )
        };

        if result == 0 {
            Ok(unsafe { Process::from_id(process_or_error_id) })
        } else {
            Err(LunaticError::from(process_or_error_id))
        }
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
