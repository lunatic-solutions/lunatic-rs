use crate::host_api::error;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

/// An opaque error returned from host calls.
///
/// Host calls can have a big number of failure reasons and it's impossible to enumerate all of
/// them. This is especially true for calls that involve compiling raw binary data to WebAssembly
/// modules. Because of this an opaque error ID is returned from host that can be transformed to
/// a string.
#[derive(Error)]
pub struct LunaticError {
    id: u64,
}

impl Drop for LunaticError {
    fn drop(&mut self) {
        unsafe { error::drop(self.id) };
    }
}

impl LunaticError {
    pub(crate) fn from(id: u64) -> Self {
        Self { id }
    }
}

impl Debug for LunaticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let size = unsafe { error::string_size(self.id) };
        let mut buff = vec![0; size as usize];
        unsafe { error::to_string(self.id, buff.as_mut_ptr()) };
        let error = std::str::from_utf8(&buff).unwrap();
        write!(f, "{}", error)
    }
}

impl Display for LunaticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let size = unsafe { error::string_size(self.id) };
        let mut buff = vec![0; size as usize];
        unsafe { error::to_string(self.id, buff.as_mut_ptr()) };
        let error = std::str::from_utf8(&buff).unwrap();
        write!(f, "{}", error)
    }
}
