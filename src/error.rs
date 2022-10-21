use std::fmt::{Debug, Display, Formatter};

use thiserror::Error;

use crate::host::api::error;

/// An opaque error returned from host calls.
///
/// Host calls can have a big number of failure reasons, and it's impossible to
/// enumerate all of them. This is especially true for calls that involve
/// compiling raw binary data to WebAssembly modules. Because of this an opaque
/// error ID is returned from host that can be transformed to a string.
#[derive(Error)]
pub enum LunaticError {
    Error(u64),
    PermissionDenied,
}

impl Drop for LunaticError {
    fn drop(&mut self) {
        match self {
            LunaticError::Error(id) => {
                unsafe { error::drop(*id) };
            }
            LunaticError::PermissionDenied => (),
        }
    }
}

impl LunaticError {
    pub(crate) fn from(id: u64) -> Self {
        LunaticError::Error(id)
    }
}

impl Debug for LunaticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            LunaticError::Error(id) => {
                let size = unsafe { error::string_size(*id) };
                let mut buff = vec![0; size as usize];
                unsafe { error::to_string(*id, buff.as_mut_ptr()) };
                let error = std::str::from_utf8(&buff).unwrap();
                write!(f, "{}", error)
            }
            LunaticError::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}

impl Display for LunaticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            LunaticError::Error(id) => {
                let size = unsafe { error::string_size(*id) };
                let mut buff = vec![0; size as usize];
                unsafe { error::to_string(*id, buff.as_mut_ptr()) };
                let error = std::str::from_utf8(&buff).unwrap();
                write!(f, "{}", error)
            }
            LunaticError::PermissionDenied => write!(f, "Permission denied"),
        }
    }
}
