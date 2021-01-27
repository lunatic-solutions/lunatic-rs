use anyhow::Result;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

use std::{ffi::c_void, fmt, net::SocketAddr};
use std::{
    io::{self, Error, ErrorKind, IoSlice, IoSliceMut, Read, Write},
    rc::Rc,
};

use super::errors;

mod stdlib {
    use std::ffi::c_void;

    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn tcp_connect(
            addr_ptr: *const u8,
            addr_len: usize,
            port: u16,
            listener_id: *mut u32,
        ) -> u32;
        pub fn close_tcp_stream(listener: u32);
        pub fn tcp_write_vectored(
            tcp_stream: u32,
            data: *const c_void,
            data_len: usize,
            nwritten: *mut usize,
        ) -> u32;
        pub fn tcp_read_vectored(
            tcp_stream: u32,
            data: *mut c_void,
            data_len: usize,
            nwritten: *mut usize,
        ) -> u32;
        pub fn tcp_stream_serialize(tcp_stream: u32) -> u32;
        pub fn tcp_stream_deserialize(tcp_stream: u32) -> u32;
    }
}

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
#[derive(Clone)]
pub struct TcpStream {
    inner: Rc<TcpStreamInner>,
}
pub struct TcpStreamInner {
    id: u32,
}

impl Drop for TcpStreamInner {
    fn drop(&mut self) {
        unsafe { stdlib::close_tcp_stream(self.id) };
    }
}

impl TcpStream {
    pub(crate) fn from(id: u32) -> Self {
        TcpStream {
            inner: Rc::new(TcpStreamInner { id }),
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
        A: std::net::ToSocketAddrs,
    {
        let mut result: u32 = 0;
        for addr in addr.to_socket_addrs()? {
            let mut id = 0;
            result = match addr {
                SocketAddr::V4(v4_addr) => {
                    let ip: [u8; 4] = v4_addr.ip().octets();
                    let port: u16 = v4_addr.port().into();
                    unsafe { stdlib::tcp_connect(ip.as_ptr(), ip.len(), port, &mut id as *mut u32) }
                }
                SocketAddr::V6(v6_addr) => {
                    let ip: [u8; 16] = v6_addr.ip().octets();
                    let port: u16 = v6_addr.port().into();
                    unsafe { stdlib::tcp_connect(ip.as_ptr(), ip.len(), port, &mut id as *mut u32) }
                }
            };
            if result == 0 {
                return Ok(Self {
                    inner: Rc::new(TcpStreamInner { id }),
                });
            }
        }
        Err(errors::TcpStreamError::CanNotEstablishTcpConnection(result).into())
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let io_slice = IoSlice::new(buf);
        self.write_vectored(&[io_slice])
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut nwritten: usize = 0;
        let result = unsafe {
            stdlib::tcp_write_vectored(
                self.inner.id,
                bufs.as_ptr() as *const c_void,
                bufs.len(),
                &mut nwritten as *mut usize,
            )
        };
        if result == 0 {
            if nwritten == 0 {
                Err(Error::new(
                    ErrorKind::ConnectionAborted,
                    "Connection closed",
                ))
            } else {
                Ok(nwritten)
            }
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!("write_vectored error: {}", result),
            ))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let io_slice = IoSliceMut::new(buf);
        self.read_vectored(&mut [io_slice])
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut nread: usize = 0;
        let result = unsafe {
            stdlib::tcp_read_vectored(
                self.inner.id,
                bufs.as_mut_ptr() as *mut c_void,
                bufs.len(),
                &mut nread as *mut usize,
            )
        };
        if result == 0 {
            if nread == 0 {
                Err(Error::new(
                    ErrorKind::ConnectionAborted,
                    "Connection closed",
                ))
            } else {
                Ok(nread)
            }
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!("read_vectored error: {}", result),
            ))
        }
    }
}

impl Serialize for TcpStream {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serialized_tcp_stream = unsafe { stdlib::tcp_stream_serialize(self.inner.id) };
        serializer.serialize_u32(serialized_tcp_stream)
    }
}

struct TcpStreamVisitor {}

impl<'de> Visitor<'de> for TcpStreamVisitor {
    type Value = TcpStream;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an pointer to an id containing a  tcp_stream")
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let tcp_stream_id = unsafe { stdlib::tcp_stream_deserialize(value) };
        Ok(TcpStream::from(tcp_stream_id))
    }
}

impl<'de> Deserialize<'de> for TcpStream {
    fn deserialize<D>(deserializer: D) -> Result<TcpStream, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(TcpStreamVisitor {})
    }
}
