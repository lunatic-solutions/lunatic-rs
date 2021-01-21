use anyhow::Result;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use std::{fmt, net::SocketAddr, rc::Rc};

use super::{errors, TcpStream};

mod stdlib {
    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn tcp_bind(
            addr_ptr: *const u8,
            addr_len: usize,
            port: u16,
            listener_id: *mut u32,
        ) -> u32;
        pub fn close_tcp_listener(listener: u32);
        pub fn tcp_accept(listener: u32, tcp_socket: *mut u32) -> u32;
        pub fn tcp_listener_serialize(tcp_stream: u32) -> u32;
        pub fn tcp_listener_deserialize(tcp_stream: u32) -> u32;
    }
}

/// A TCP server, listening for connections.
///
/// After creating a [`TcpListener`] by [`bind`][`TcpListener::bind()`]ing it to an address, it
/// listens for incoming TCP connections. These can be accepted by calling
/// [`accept()`][`TcpListener::accept()`] or by awaiting items from the stream of
/// [`incoming`][`TcpListener::incoming()`] connections.
///
/// Cloning a [`TcpListener`] creates another handle to the same socket. The socket will be closed
/// when all handles to it are dropped.
///
/// The Transmission Control Protocol is specified in [IETF RFC 793].
///
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
///
/// # Examples
///
/// ```no_run
/// use lunatic::net::TcpListener;
///
/// ```
#[derive(Clone)]
pub struct TcpListener {
    inner: Rc<TcpListenerInner>,
}
pub struct TcpListenerInner {
    id: u32,
}

impl Drop for TcpListenerInner {
    fn drop(&mut self) {
        unsafe { stdlib::close_tcp_listener(self.id) };
    }
}

impl TcpListener {
    pub(crate) fn from(id: u32) -> Self {
        TcpListener {
            inner: Rc::new(TcpListenerInner { id }),
        }
    }

    /// Creates a new [`TcpListener`] bound to the given address.
    ///
    /// Binding with a port number of 0 will request that the operating system assigns an available
    /// port to this listener.
    ///
    /// If `addr` yields multiple addresses, binding will be attempted with each of the addresses
    /// until one succeeds and returns the listener. If none of the addresses succeed in creating a
    /// listener, the error from the last attempt is returned.
    pub fn bind<A>(addr: A) -> Result<Self>
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
                    unsafe { stdlib::tcp_bind(ip.as_ptr(), ip.len(), port, &mut id as *mut u32) }
                }
                SocketAddr::V6(v6_addr) => {
                    let ip: [u8; 16] = v6_addr.ip().octets();
                    let port: u16 = v6_addr.port().into();
                    unsafe { stdlib::tcp_bind(ip.as_ptr(), ip.len(), port, &mut id as *mut u32) }
                }
            };
            if result == 0 {
                return Ok(Self {
                    inner: Rc::new(TcpListenerInner { id }),
                });
            }
        }
        Err(errors::TcpListenerError::CanNotBindingToSocket(result).into())
    }

    /// Accepts a new incoming connection.
    ///
    /// Returns a TCP stream.
    pub fn accept(&self) -> Result<TcpStream, u32> {
        let mut tcp_stream_id = 0;
        let result = unsafe { stdlib::tcp_accept(self.inner.id, &mut tcp_stream_id as *mut u32) };
        if result == 0 {
            Ok(TcpStream::from(tcp_stream_id))
        } else {
            Err(result)
        }
    }
}

impl Serialize for TcpListener {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let serialized_tcp_stream = unsafe { stdlib::tcp_listener_serialize(self.inner.id) };
        serializer.serialize_u32(serialized_tcp_stream)
    }
}

struct TcpListenerVisitor {}

impl<'de> Visitor<'de> for TcpListenerVisitor {
    type Value = TcpListener;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an pointer to an id containing a  tcp_stream")
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let tcp_stream_id = unsafe { stdlib::tcp_listener_deserialize(value) };
        Ok(TcpListener::from(tcp_stream_id))
    }
}

impl<'de> Deserialize<'de> for TcpListener {
    fn deserialize<D>(deserializer: D) -> Result<TcpListener, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(TcpListenerVisitor {})
    }
}
