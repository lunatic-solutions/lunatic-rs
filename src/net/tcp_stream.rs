use std::{
    cell::UnsafeCell,
    io::{Error, ErrorKind, IoSlice, Read, Result, Write},
    net::SocketAddr,
    time::Duration,
};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{error::LunaticError, host};

/// A TCP connection.
///
/// A [`TcpStream`] can be created by [`connect`][`TcpStream::connect()`]ing to an endpoint or by
/// [`accept`][`super::TcpListener::accept()`]ing an incoming connection.
///
/// [`TcpStream`] is a bidirectional stream that implements traits [`Read`] and [`Write`].
///
/// Cloning a [`TcpStream`] creates another handle to the same socket. The socket will be closed
/// when all handles to it are dropped.
///
/// The Transmission Control Protocol is specified in [IETF RFC 793].
///
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
#[derive(Debug)]
pub struct TcpStream {
    id: u64,
    read_timeout: u32,  // ms
    write_timeout: u32, // ms
    // If the TCP stream is serialized it will be removed from our resources, so we can't call
    // `drop_tcp_stream()` anymore on it.
    consumed: UnsafeCell<bool>,
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        // Only drop stream if it's not already consumed
        if unsafe { !*self.consumed.get() } {
            host::api::networking::drop_tcp_stream(self.id);
        }
    }
}

impl Clone for TcpStream {
    fn clone(&self) -> Self {
        let id = host::api::networking::clone_tcp_stream(self.id);
        Self {
            id,
            read_timeout: self.read_timeout,
            write_timeout: self.write_timeout,
            consumed: UnsafeCell::new(false),
        }
    }
}

impl Serialize for TcpStream {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Mark process as consumed
        unsafe { *self.consumed.get() = true };
        // TODO: Timeout info is not serialized
        let index = host::api::message::push_tcp_stream(self.id);
        serializer.serialize_u64(index)
    }
}
struct TcpStreamVisitor;
impl<'de> Visitor<'de> for TcpStreamVisitor {
    type Value = TcpStream;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an u64 index")
    }

    fn visit_u64<E>(self, index: u64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        let id = host::api::message::take_tcp_stream(index);
        Ok(TcpStream::from(id))
    }
}

impl<'de> Deserialize<'de> for TcpStream {
    fn deserialize<D>(deserializer: D) -> std::result::Result<TcpStream, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(TcpStreamVisitor)
    }
}

impl TcpStream {
    pub(crate) fn from(id: u64) -> Self {
        TcpStream {
            id,
            read_timeout: 0,
            write_timeout: 0,
            consumed: UnsafeCell::new(false),
        }
    }

    /// Sets the read timeout.
    ///
    /// If the value specified is `None`, then read calls will block indefinitely.
    pub fn set_read_timeout(&mut self, duration: Option<Duration>) {
        match duration {
            None => self.read_timeout = 0,
            Some(duration) => self.read_timeout = duration.as_millis() as u32,
        }
    }

    /// Sets the write timeout.
    ///
    /// If the value specified is `None`, then write calls will block indefinitely.
    pub fn set_write_timeout(&mut self, duration: Option<Duration>) {
        match duration {
            None => self.write_timeout = 0,
            Some(duration) => self.write_timeout = duration.as_millis() as u32,
        }
    }

    /// Creates a TCP connection to the specified address.
    ///
    /// This method will create a new TCP socket and attempt to connect it to the provided `addr`,
    ///
    /// If `addr` yields multiple addresses, connecting will be attempted with each of the
    /// addresses until connecting to one succeeds. If none of the addresses result in a successful
    /// connection, the error from the last connect attempt is returned.
    pub fn connect<A>(addr: A) -> Result<Self>
    where
        A: super::ToSocketAddrs,
    {
        TcpStream::connect_timeout_(addr, None)
    }

    /// Same as [`TcpStream::connect`], but only waits for the duration of timeout to connect.
    pub fn connect_timeout<A>(addr: A, timeout: Duration) -> Result<Self>
    where
        A: super::ToSocketAddrs,
    {
        TcpStream::connect_timeout_(addr, Some(timeout))
    }

    fn connect_timeout_<A>(addr: A, timeout: Option<Duration>) -> Result<Self>
    where
        A: super::ToSocketAddrs,
    {
        let mut id = 0;
        for addr in addr.to_socket_addrs()? {
            let timeout_ms = match timeout {
                // If waiting time is smaller than 1ms, round it up to 1ms.
                Some(timeout) => match timeout.as_millis() {
                    0 => 1,
                    other => other as u32,
                },
                None => 0,
            };
            let result = match addr {
                SocketAddr::V4(v4_addr) => {
                    let ip = v4_addr.ip().octets();
                    let port = v4_addr.port() as u32;
                    host::api::networking::tcp_connect(
                        4,
                        ip.as_ptr() as u32,
                        port,
                        0,
                        0,
                        timeout_ms,
                        &mut id as *mut u64 as u64,
                    )
                }
                SocketAddr::V6(v6_addr) => {
                    let ip = v6_addr.ip().octets();
                    let port = v6_addr.port() as u32;
                    let flow_info = v6_addr.flowinfo();
                    let scope_id = v6_addr.scope_id();
                    host::api::networking::tcp_connect(
                        6,
                        ip.as_ptr() as u32,
                        port,
                        flow_info,
                        scope_id,
                        timeout_ms,
                        &mut id as *mut u64 as u64,
                    )
                }
            };
            if result == 0 {
                return Ok(TcpStream::from(id));
            }
        }
        let lunatic_error = LunaticError::from(id);
        Err(Error::new(ErrorKind::Other, lunatic_error))
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let io_slice = IoSlice::new(buf);
        self.write_vectored(&[io_slice])
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> {
        let mut nwritten_or_error_id: u64 = 0;
        let result = host::api::networking::tcp_write_vectored(
            self.id,
            bufs.as_ptr() as u32,
            bufs.len() as u32,
            self.write_timeout,
            &mut nwritten_or_error_id as *mut u64 as u64,
        );
        if result == 0 {
            Ok(nwritten_or_error_id as usize)
        } else {
            let lunatic_error = LunaticError::from(nwritten_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }

    fn flush(&mut self) -> Result<()> {
        let mut error_id = 0;
        match host::api::networking::tcp_flush(self.id, &mut error_id as *mut u64 as u64) {
            0 => Ok(()),
            _ => {
                let lunatic_error = LunaticError::from(error_id);
                Err(Error::new(ErrorKind::Other, lunatic_error))
            }
        }
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut nread_or_error_id: u64 = 0;
        let result = host::api::networking::tcp_read(
            self.id,
            buf.as_mut_ptr() as u32,
            buf.len() as u32,
            self.read_timeout,
            &mut nread_or_error_id as *mut u64 as u64,
        );
        if result == 0 {
            Ok(nread_or_error_id as usize)
        } else {
            let lunatic_error = LunaticError::from(nread_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }
}
