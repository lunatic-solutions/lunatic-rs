use std::ptr;

use super::Process;

mod stdlib {
    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn load_module(data: *const u8, data_len: usize, module: *mut u32) -> u32;
        pub fn unload_module(id: u32);
        pub fn create_import(namespace: *const u8, namespace_len: usize, module: u32) -> u32;
        pub fn remove_import(id: u32);
        pub fn spawn_from_module(
            id: u32,
            name: *const u8,
            name_len: usize,
            max_memory: u32,
            imports: *const u8,
            imports_len: usize,
        ) -> u32;
    }
}

/// Represents a Wasm binary module that can be used to spawn processes or as an import
/// to other modules.
pub struct Module {
    id: u32,
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe { stdlib::unload_module(self.id) };
    }
}

impl Module {
    // Create a Module from a binary Wasm module.
    pub fn load(data: &[u8]) -> Result<Self, ()> {
        let mut id = 0;
        match unsafe { stdlib::load_module(data.as_ptr(), data.len(), &mut id as *mut u32) } {
            0 => Ok(Self { id }),
            _ => Err(()),
        }
    }

    // Spawn a new process.
    pub fn spawn(&self, function: &str, max_memory: u32) -> Process {
        let process_id = unsafe {
            stdlib::spawn_from_module(
                self.id,
                function.as_ptr(),
                function.len(),
                max_memory,
                ptr::null(),
                0,
            )
        };
        Process::from(process_id)
    }

    // Spawn a new process using modules as imports.
    pub fn spawn_with(
        &self,
        function: &str,
        max_memory: u32,
        imports: &[(String, Module)],
    ) -> Process {
        let imports: Vec<u32> = imports
            .iter()
            .map(|import| unsafe {
                stdlib::create_import(import.0.as_ptr(), import.0.len(), import.1.id)
            })
            .collect();
        let process_id = unsafe {
            stdlib::spawn_from_module(
                self.id,
                function.as_ptr(),
                function.len(),
                max_memory,
                imports.as_ptr() as *const u8,
                imports.len() * 4, // We need the length as a u8 buffer
            )
        };
        imports
            .iter()
            .for_each(|import| unsafe { stdlib::remove_import(*import) });
        Process::from(process_id)
    }
}
