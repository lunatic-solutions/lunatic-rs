use std::{
    cell::UnsafeCell,
    io::{Error, ErrorKind, IoSlice, Read, Result, Write},
    time::Duration,
};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{error::LunaticError, host};

const TIMEOUT: u32 = 9027;

/// A TCP connection.
///
/// A [`TlsStream`] can be created by [`connect`][`TlsStream::connect()`]ing to an endpoint or by
/// [`accept`][`super::TcpListener::accept()`]ing an incoming connection.
///
/// [`TlsStream`] is a bidirectional stream that implements traits [`Read`] and [`Write`].
///
/// Cloning a [`TlsStream`] creates another handle to the same socket. The socket will be closed
/// when all handles to it are dropped.
///
/// The Transmission Control Protocol is specified in [IETF RFC 793].
///
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
#[derive(Debug)]
pub struct TlsStream {
    id: u64,
    // If the TLS stream is serialized it will be removed from our resources, so we can't call
    // `drop_tls_stream()` anymore on it.
    consumed: UnsafeCell<bool>,
}

impl Drop for TlsStream {
    fn drop(&mut self) {
        // Only drop stream if it's not already consumed
        if unsafe { !*self.consumed.get() } {
            unsafe { host::api::networking::drop_tls_stream(self.id) };
        }
    }
}

impl Clone for TlsStream {
    fn clone(&self) -> Self {
        let id = unsafe { host::api::networking::clone_tls_stream(self.id) };
        Self {
            id,
            consumed: UnsafeCell::new(false),
        }
    }
}

impl Serialize for TlsStream {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Mark process as consumed
        unsafe { *self.consumed.get() = true };
        // TODO: Timeout info is not serialized
        let index = unsafe { host::api::message::push_tls_stream(self.id) };
        // panic!("Need stacktrace");
        serializer.serialize_u64(index)
    }
}
struct TlsStreamVisitor;
impl<'de> Visitor<'de> for TlsStreamVisitor {
    type Value = TlsStream;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an u64 index")
    }

    fn visit_u64<E>(self, index: u64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        let id = unsafe { host::api::message::take_tls_stream(index) };
        Ok(TlsStream::from(id))
    }
}

impl<'de> Deserialize<'de> for TlsStream {
    fn deserialize<D>(deserializer: D) -> std::result::Result<TlsStream, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(TlsStreamVisitor)
    }
}

impl TlsStream {
    pub(crate) fn from(id: u64) -> Self {
        TlsStream {
            id,
            consumed: UnsafeCell::new(false),
        }
    }

    /// Creates a TLS connection to the specified address.
    ///
    /// This method will create a new TLS socket and attempt to connect it to the provided `addr`,
    ///
    /// If `addr` yields multiple addresses, connecting will be attempted with each of the
    /// addresses until connecting to one succeeds. If none of the addresses result in a successful
    /// connection, the error from the last connect attempt is returned.
    pub fn connect(addr: &str, port: u32) -> Result<Self> {
        TlsStream::connect_timeout_(addr, None, port, vec![])
    }

    /// Creates a TLS connection to the specified address with custom certificates.
    ///
    /// This method will create a new TLS socket and attempt to connect it to the provided `addr`,
    ///
    /// If `addr` yields multiple addresses, connecting will be attempted with each of the
    /// addresses until connecting to one succeeds. If none of the addresses result in a successful
    /// connection, the error from the last connect attempt is returned.
    pub fn connect_with_certs(addr: &str, port: u32, certs: Vec<Vec<u8>>) -> Result<Self> {
        TlsStream::connect_timeout_(addr, None, port, certs)
    }

    /// Same as [`TlsStream::connect`], but only waits for the duration of timeout to connect.
    pub fn connect_timeout(
        addr: &str,
        timeout: Duration,
        port: u32,
        certs: Vec<Vec<u8>>,
    ) -> Result<Self> {
        TlsStream::connect_timeout_(addr, Some(timeout), port, certs)
    }

    fn connect_timeout_(
        addr: &str,
        timeout: Option<Duration>,
        port: u32,
        certs: Vec<Vec<u8>>,
    ) -> Result<Self> {
        let mut id = 0;
        let timeout_ms = match timeout {
            Some(timeout) => timeout.as_millis() as u64,
            None => u64::MAX,
        };
        let certs_mapped: Vec<(u32, *mut u32)> = certs
            .iter()
            .map(|cert| (cert.len() as u32, cert.as_ptr() as *mut u32))
            .collect();
        let result = unsafe {
            host::api::networking::tls_connect(
                addr.as_ptr(),
                addr.len() as u32,
                port,
                timeout_ms,
                &mut id as *mut u64,
                certs_mapped.as_ptr() as *mut u8,
                certs_mapped.len() as u32,
            )
        };
        if result == 0 {
            return Ok(TlsStream::from(id));
        }
        let lunatic_error = LunaticError::from(id);
        Err(Error::new(ErrorKind::Other, lunatic_error))
    }

    /// Sets write timeout for TlsStream
    ///
    /// This method will change the timeout for everyone holding a reference to the TlsStream
    /// Once a timeout is set, it can be removed by sending `None`
    pub fn set_write_timeout(&mut self, duration: Option<Duration>) -> Result<()> {
        unsafe {
            let code = host::api::networking::set_write_timeout(
                self.id,
                duration.map_or(u64::MAX, |d| d.as_millis() as u64),
            );
            if code != 0 {
                let lunatic_error = LunaticError::from(code as u64);
                return Err(Error::new(ErrorKind::Other, lunatic_error));
            }
        }
        Ok(())
    }

    /// Gets write timeout for TlsStream
    ///
    /// This method retrieves the write timeout duration of the TlsStream if any
    pub fn write_timeout(&self) -> Option<Duration> {
        unsafe {
            match host::api::networking::get_write_timeout(self.id) {
                u64::MAX => None,
                millis => Some(Duration::from_millis(millis)),
            }
        }
    }

    /// Sets read timeout for TlsStream
    ///
    /// This method will change the timeout for everyone holding a reference to the TlsStream
    /// Once a timeout is set, it can be removed by sending `None`
    pub fn set_read_timeout(&mut self, duration: Option<Duration>) -> Result<()> {
        unsafe {
            let code = host::api::networking::set_read_timeout(
                self.id,
                duration.map_or(u64::MAX, |d| d.as_millis() as u64),
            );
            if code != 0 {
                let lunatic_error = LunaticError::from(code as u64);
                return Err(Error::new(ErrorKind::Other, lunatic_error));
            }
        }
        Ok(())
    }

    /// Gets read timeout for TlsStream
    ///
    /// This method retrieves the read timeout duration of the TlsStream if any
    pub fn read_timeout(&self) -> Option<Duration> {
        unsafe {
            match host::api::networking::get_read_timeout(self.id) {
                u64::MAX => None,
                millis => Some(Duration::from_millis(millis)),
            }
        }
    }
}

impl Write for TlsStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let io_slice = IoSlice::new(buf);
        self.write_vectored(&[io_slice])
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> {
        let mut nwritten_or_error_id: u64 = 0;
        let result = unsafe {
            host::api::networking::tls_write_vectored(
                self.id,
                bufs.as_ptr() as *const u32,
                bufs.len(),
                &mut nwritten_or_error_id as *mut u64,
            )
        };
        if result == 0 {
            Ok(nwritten_or_error_id as usize)
        } else if result == TIMEOUT {
            Err(Error::new(ErrorKind::TimedOut, "TlsStream write timed out"))
        } else {
            let lunatic_error = LunaticError::from(nwritten_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }

    fn flush(&mut self) -> Result<()> {
        let mut error_id = 0;
        match unsafe { host::api::networking::tls_flush(self.id, &mut error_id as *mut u64) } {
            0 => Ok(()),
            _ => {
                let lunatic_error = LunaticError::from(error_id);
                Err(Error::new(ErrorKind::Other, lunatic_error))
            }
        }
    }
}

impl Read for TlsStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut nread_or_error_id: u64 = 0;
        let result = unsafe {
            host::api::networking::tls_read(
                self.id,
                buf.as_mut_ptr(),
                buf.len(),
                &mut nread_or_error_id as *mut u64,
            )
        };
        if result == 0 {
            Ok(nread_or_error_id as usize)
        } else if result == TIMEOUT {
            Err(Error::new(ErrorKind::TimedOut, "TlsStream read timed out"))
        } else {
            let lunatic_error = LunaticError::from(nread_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }
}
