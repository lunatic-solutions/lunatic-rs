use std::{
    convert::TryInto,
    io::{Error, ErrorKind, IoSlice, Read, Result, Write},
    mem::forget,
    net::SocketAddr,
    time::Duration,
};

use crate::{error::LunaticError, host_api, message::Message};

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
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        unsafe { host_api::networking::drop_tcp_stream(self.id) };
    }
}

impl Clone for TcpStream {
    fn clone(&self) -> Self {
        let id = unsafe { host_api::networking::clone_tcp_stream(self.id) };
        Self {
            id,
            read_timeout: self.read_timeout,
            write_timeout: self.write_timeout,
        }
    }
}

impl Message for TcpStream {
    fn from_bincode(data: &[u8], resources: &[u64]) -> (usize, Self) {
        // The serialized value for a tcp stream is the u64 index inside the resources array.
        // The resources array will contain the new resource index.
        let index = u64::from_le_bytes(data.try_into().unwrap());
        let proc = TcpStream::from(resources[index as usize]);
        (8, proc)
    }

    #[allow(clippy::wrong_self_convention)]
    unsafe fn to_bincode(self, dest: &mut Vec<u8>) {
        let index = host_api::message::push_tcp_stream(self.id);
        dest.extend(index.to_le_bytes());
        // By adding the tcp stream to the message it will be removed from our resources.
        // Dropping it would cause a trap.
        forget(self);
    }
}

impl TcpStream {
    pub(crate) fn from(id: u64) -> Self {
        TcpStream {
            id,
            read_timeout: 0,
            write_timeout: 0,
        }
    }

    /// Creates a TCP connection to the specified address.
    ///
    /// This method will create a new TCP socket and attempt to connect it to the provided `addr`,
    ///
    /// If `addr` yields multiple addresses, connecting will be attempted with each of the
    /// addresses until connecting to one succeeds. If none of the addresses result in a successful
    /// connection, the error from the last connect attempt is returned.
    pub fn connect<A>(addr: A, timeout: Option<Duration>) -> Result<Self>
    where
        A: super::ToSocketAddrs,
    {
        let mut id = 0;
        for addr in addr.to_socket_addrs()? {
            let timeout_ms = match timeout {
                Some(timeout) => timeout.as_millis() as u32,
                None => 0,
            };
            let result = match addr {
                SocketAddr::V4(v4_addr) => {
                    let ip = v4_addr.ip().octets();
                    let port = v4_addr.port() as u32;
                    unsafe {
                        host_api::networking::tcp_connect(
                            4,
                            ip.as_ptr(),
                            port,
                            0,
                            0,
                            timeout_ms,
                            &mut id as *mut u64,
                        )
                    }
                }
                SocketAddr::V6(v6_addr) => {
                    let ip = v6_addr.ip().octets();
                    let port = v6_addr.port() as u32;
                    let flow_info = v6_addr.flowinfo();
                    let scope_id = v6_addr.scope_id();
                    unsafe {
                        host_api::networking::tcp_connect(
                            6,
                            ip.as_ptr(),
                            port,
                            flow_info,
                            scope_id,
                            timeout_ms,
                            &mut id as *mut u64,
                        )
                    }
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
        let result = unsafe {
            host_api::networking::tcp_write_vectored(
                self.id,
                bufs.as_ptr() as *const u32,
                bufs.len(),
                self.write_timeout,
                &mut nwritten_or_error_id as *mut u64,
            )
        };
        if result == 0 {
            Ok(nwritten_or_error_id as usize)
        } else {
            let lunatic_error = LunaticError::from(nwritten_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }

    fn flush(&mut self) -> Result<()> {
        let mut error_id = 0;
        match unsafe { host_api::networking::tcp_flush(self.id, &mut error_id as *mut u64) } {
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
        let result = unsafe {
            host_api::networking::tcp_read(
                self.id,
                buf.as_mut_ptr(),
                buf.len(),
                self.read_timeout,
                &mut nread_or_error_id as *mut u64,
            )
        };
        if result == 0 {
            Ok(nread_or_error_id as usize)
        } else {
            let lunatic_error = LunaticError::from(nread_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }
}
