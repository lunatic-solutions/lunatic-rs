use std::io;

#[derive(Debug, Copy, Clone)]
/// The IoError.
pub enum IoError {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    Interrupted,
    UnexpectedEof,
    OutOfMemory,
    Unsupported,
    Other,
}

impl IoError {
    pub(crate) fn into_io_error_with_text(self, str: impl ToString) -> io::Error {
        io::Error::new(From::from(self), str.to_string())
    }
}

impl From<u32> for IoError {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::NotFound,
            2 => Self::PermissionDenied,
            3 => Self::ConnectionRefused,
            4 => Self::ConnectionReset,
            5 => Self::ConnectionAborted,
            6 => Self::NotConnected,
            7 => Self::AddrInUse,
            8 => Self::AddrNotAvailable,
            9 => Self::BrokenPipe,
            10 => Self::AlreadyExists,
            11 => Self::WouldBlock,
            12 => Self::InvalidInput,
            13 => Self::InvalidData,
            14 => Self::TimedOut,
            15 => Self::WriteZero,
            16 => Self::Interrupted,
            17 => Self::UnexpectedEof,
            18 => Self::OutOfMemory,
            19 => Self::Unsupported,
            20 | 99 => Self::Other,
            _ => panic!("The Lunatic runtime sent an invalid error code."),
        }
    }
}

impl From<IoError> for io::ErrorKind {
    fn from(e: IoError) -> Self {
        match e {
            IoError::NotFound => Self::NotFound,
            IoError::PermissionDenied => Self::PermissionDenied,
            IoError::ConnectionRefused => Self::ConnectionRefused,
            IoError::ConnectionReset => Self::ConnectionReset,
            IoError::ConnectionAborted => Self::ConnectionAborted,
            IoError::NotConnected => Self::NotConnected,
            IoError::AddrInUse => Self::AddrInUse,
            IoError::AddrNotAvailable => Self::AddrNotAvailable,
            IoError::BrokenPipe => Self::BrokenPipe,
            IoError::AlreadyExists => Self::AlreadyExists,
            IoError::WouldBlock => Self::WouldBlock,
            IoError::InvalidInput => Self::InvalidInput,
            IoError::InvalidData => Self::InvalidData,
            IoError::TimedOut => Self::TimedOut,
            IoError::WriteZero => Self::WriteZero,
            IoError::Interrupted => Self::Interrupted,
            IoError::UnexpectedEof => Self::UnexpectedEof,
            // note that `OutOfMemory` and `Unsupported` have not yet been stabilised, but are on
            // track for stablisation in `1.53` and `1.54`
            IoError::OutOfMemory | IoError::Unsupported | IoError::Other => Self::Other,
        }
    }
}

impl From<IoError> for io::Error {
    fn from(err: IoError) -> Self {
        Self::new(From::from(err), format!("{:#?}", err))
    }
}
