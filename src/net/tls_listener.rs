use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;

use super::SocketAddrIterator;
use crate::{error::LunaticError, host, net::TlsStream};

/// A TLS server, listening for connections.
///
/// After creating a [`TlsListener`] by [`bind`][`TlsListener::bind()`]ing it to an address, it
/// listens for incoming encrypted TCP (TLS) connections. These can be accepted by calling
/// [`accept()`][`TlsListener::accept()`].
///
/// The Transmission Control Protocol is specified in [IETF RFC 793].
///
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
///
/// # Examples
///
/// ```no_run
/// use lunatic::{net, Mailbox, Process};
/// use std::io::{BufRead, BufReader, Write};
///
/// fn main() {
///     let listener = net::TlsListener::bind("127.0.0.1:0").unwrap();
///     while let Ok((tls_stream, _peer)) = listener.accept() {
///         // Handle connections in a new process
///         Process::spawn(tls_stream, handle);
///     }
/// }
///
/// fn handle(mut tls_stream: net::TlsStream, _: Mailbox<()>) {
///     let mut buf_reader = BufReader::new(tls_stream.clone());
///     loop {
///         let mut buffer = String::new();
///         let read = buf_reader.read_line(&mut buffer).unwrap();
///         if buffer.contains("exit") || read == 0 {
///             return;
///         }
///         tls_stream.write(buffer.as_bytes()).unwrap();
///     }
/// }
/// ```
#[derive(Debug)]
pub struct TlsListener {
    id: u64,
    certs: Vec<u8>,
    keys: Vec<u8>,
}

impl Drop for TlsListener {
    fn drop(&mut self) {
        unsafe { host::api::networking::drop_tls_listener(self.id) };
    }
}

impl TlsListener {
    /// Creates a new [`TlsListener`] bound to the given address.
    ///
    /// Binding with a port number of 0 will request that the operating system assigns an available
    /// port to this listener.
    ///
    /// If `addr` yields multiple addresses, binding will be attempted with each of the addresses
    /// until one succeeds and returns the listener. If none of the addresses succeed in creating a
    /// listener, the error from the last attempt is returned.
    pub fn bind<A>(addr: A, certs: Vec<u8>, keys: Vec<u8>) -> Result<Self>
    where
        A: super::ToSocketAddrs,
    {
        let mut id = 0;
        for addr in addr.to_socket_addrs()? {
            let result = match addr {
                SocketAddr::V4(v4_addr) => {
                    let ip = v4_addr.ip().octets();
                    let port = v4_addr.port() as u32;
                    unsafe {
                        host::api::networking::tls_bind(
                            4,
                            ip.as_ptr(),
                            port,
                            0,
                            0,
                            &mut id as *mut u64,
                            certs.as_ptr() as *const u32,
                            certs.len(),
                            keys.as_ptr() as *const u32,
                            keys.len(),
                        )
                    }
                }
                SocketAddr::V6(v6_addr) => {
                    let ip = v6_addr.ip().octets();
                    let port = v6_addr.port() as u32;
                    let flow_info = v6_addr.flowinfo();
                    let scope_id = v6_addr.scope_id();
                    unsafe {
                        host::api::networking::tls_bind(
                            6,
                            ip.as_ptr(),
                            port,
                            flow_info,
                            scope_id,
                            &mut id as *mut u64,
                            certs.as_ptr() as *const u32,
                            certs.len(),
                            keys.as_ptr() as *const u32,
                            keys.len(),
                        )
                    }
                }
            };
            if result == 0 {
                return Ok(Self { id, certs, keys });
            }
        }
        let lunatic_error = LunaticError::from(id);
        Err(Error::new(ErrorKind::Other, lunatic_error))
    }

    /// Accepts a new incoming connection.
    ///
    /// This will block and typically needs its own dedicated child process loop.
    ///
    /// Returns a TLS stream and the peer address.
    pub fn accept(&self) -> Result<(TlsStream, SocketAddr)> {
        let mut tls_stream_or_error_id = 0;
        let mut dns_iter_id = 0;
        let result = unsafe {
            host::api::networking::tls_accept(
                self.id,
                &mut tls_stream_or_error_id as *mut u64,
                &mut dns_iter_id as *mut u64,
            )
        };
        if result == 0 {
            let tls_stream = TlsStream::from(tls_stream_or_error_id);
            let mut dns_iter = SocketAddrIterator::from(dns_iter_id);
            let peer = dns_iter.next().expect("must contain one element");
            println!("RETURNING FROM ACCEPT {:?} | {:?}", tls_stream, peer);
            Ok((tls_stream, peer))
        } else {
            let lunatic_error = LunaticError::from(tls_stream_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }

    /// Returns the local address that this listener is bound to.
    ///
    /// This can be useful, for example, to identify when binding to port 0 which port was assigned by the OS.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let mut dns_iter_or_error_id = 0;
        let result = unsafe {
            host::api::networking::tls_local_addr(self.id, &mut dns_iter_or_error_id as *mut u64)
        };
        if result == 0 {
            let mut dns_iter = SocketAddrIterator::from(dns_iter_or_error_id);
            let addr = dns_iter.next().expect("must contain one element");
            Ok(addr)
        } else {
            let lunatic_error = LunaticError::from(dns_iter_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }
}
