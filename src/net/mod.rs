//! Networking related functions.

mod resolver;
mod tcp_listener;
mod tcp_stream;
mod udp;

use std::io::{Error, ErrorKind, Result};
use std::iter::Cloned;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::option::IntoIter;
use std::slice::Iter;

pub use resolver::{resolve, resolve_timeout, SocketAddrIterator};
pub use tcp_listener::TcpListener;
pub use tcp_stream::TcpStream;
pub use udp::UdpSocket;

/// A trait for objects which can be converted or resolved to one or more
/// [`SocketAddr`] values.
///
/// Currently, this trait is only used as an argument to lunatic functions
/// that need to reference a target socket address. To perform a `SocketAddr`
/// conversion directly, use [`resolve`]. By default it is implemented
/// for the following types:
///
///  * [`SocketAddr`]: [`to_socket_addrs`] is the identity function.
///
///  * [`SocketAddrV4`], [`SocketAddrV6`], `(`[`IpAddr`]`, `[`u16`]`)`,
///    `(`[`Ipv4Addr`]`, `[`u16`]`)`, `(`[`Ipv6Addr`]`, `[`u16`]`)`:
///    [`to_socket_addrs`] constructs a [`SocketAddr`] trivially.
///
///  * [`&str`]: the string should be either a string representation of a
///    [`SocketAddr`] as expected by its [`FromStr`] implementation or a string like
///    `<host_name>:<port>` pair where `<port>` is a [`u16`] value.
///
/// This trait allows constructing network objects like [`TcpStream`] easily with
/// values of various types for the bind/connection address. It is needed because
/// sometimes one type is more appropriate than the other: for simple uses a string
/// like `"localhost:12345"` is much nicer than manual construction of the corresponding
/// [`SocketAddr`], but sometimes [`SocketAddr`] value is *the* main source of the
/// address, and converting it to  some other type (e.g., a string) just for it to
/// be converted back to [`SocketAddr`] in constructor methods is pointless.
///
/// Addresses returned by the operating system that are not IP addresses are
/// silently ignored.
///
/// [`resolve`]: resolve
/// [`FromStr`]: std::str::FromStr
/// [`&str`]: str
/// [`TcpStream`]: crate::net::TcpStream
/// [`to_socket_addrs`]: ToSocketAddrs::to_socket_addrs
///
/// # Examples
///
/// Creating a [`SocketAddr`] iterator that yields one item:
///
/// ```
/// use lunatic::net::ToSocketAddrs;
/// use std::net::SocketAddr;
///
/// let addr = SocketAddr::from(([127, 0, 0, 1], 443));
/// let mut addrs_iter = addr.to_socket_addrs().unwrap();
///
/// assert_eq!(Some(addr), addrs_iter.next());
/// assert!(addrs_iter.next().is_none());
/// ```
///
/// Creating a [`SocketAddr`] iterator from a hostname:
///
/// ```no_run
/// use std::net::SocketAddr;
/// use lunatic::net::ToSocketAddrs;
///
/// // assuming 'localhost' resolves to 127.0.0.1
/// let mut addrs_iter = "localhost:443".to_socket_addrs().unwrap();
/// assert_eq!(addrs_iter.next(), Some(SocketAddr::from(([127, 0, 0, 1], 443))));
/// assert!(addrs_iter.next().is_none());
///
/// // assuming 'foo' does not resolve
/// assert!("foo:443".to_socket_addrs().is_err());
/// ```
///
/// Creating a [`SocketAddr`] iterator that yields multiple items:
///
/// ```
/// use std::net::SocketAddr;
/// use lunatic::net::ToSocketAddrs;
///
/// let addr1 = SocketAddr::from(([0, 0, 0, 0], 80));
/// let addr2 = SocketAddr::from(([127, 0, 0, 1], 443));
/// let addrs = vec![addr1, addr2];
///
/// let mut addrs_iter = (&addrs[..]).to_socket_addrs().unwrap();
///
/// assert_eq!(Some(addr1), addrs_iter.next());
/// assert_eq!(Some(addr2), addrs_iter.next());
/// assert!(addrs_iter.next().is_none());
/// ```
///
/// Attempting to create a [`SocketAddr`] iterator from an improperly formatted
/// socket address `&str` (missing the port):
///
/// ```
/// use std::io;
/// use lunatic::net::ToSocketAddrs;
///
/// let err = "127.0.0.1".to_socket_addrs().unwrap_err();
/// assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
/// ```
///
/// [`TcpStream::connect`] is an example of an function that utilizes
/// `ToSocketAddrs` as a trait bound on its parameter in order to accept
/// different types:
///
/// ```no_run
/// use lunatic::net::TcpStream;
/// use std::net::Ipv4Addr;
/// // or
/// let stream = TcpStream::connect("127.0.0.1:443");
/// // or
/// let stream = TcpStream::connect((Ipv4Addr::new(127, 0, 0, 1), 443));
/// ```
///
/// [`TcpStream::connect`]: TcpStream::connect
pub trait ToSocketAddrs {
    type Iter: Iterator<Item = std::net::SocketAddr>;
    fn to_socket_addrs(&self) -> Result<Self::Iter>;
}

impl ToSocketAddrs for &str {
    type Iter = SocketAddrIterator;

    fn to_socket_addrs(&self) -> Result<Self::Iter> {
        match resolve(self) {
            Ok(iter) => Ok(iter),
            Err(err) => Err(Error::new(ErrorKind::Other, err)),
        }
    }
}

impl ToSocketAddrs for String {
    type Iter = SocketAddrIterator;

    fn to_socket_addrs(&self) -> Result<Self::Iter> {
        match resolve(self) {
            Ok(iter) => Ok(iter),
            Err(err) => Err(Error::new(ErrorKind::Other, err)),
        }
    }
}

/* The rest is just forwarded to the standard library implementations */

impl ToSocketAddrs for SocketAddr {
    type Iter = IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> Result<Self::Iter> {
        <SocketAddr as std::net::ToSocketAddrs>::to_socket_addrs(self)
    }
}

impl ToSocketAddrs for (IpAddr, u16) {
    type Iter = IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> Result<Self::Iter> {
        <(IpAddr, u16) as std::net::ToSocketAddrs>::to_socket_addrs(self)
    }
}

impl ToSocketAddrs for (Ipv4Addr, u16) {
    type Iter = IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> Result<Self::Iter> {
        <(Ipv4Addr, u16) as std::net::ToSocketAddrs>::to_socket_addrs(self)
    }
}

impl ToSocketAddrs for (Ipv6Addr, u16) {
    type Iter = IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> Result<Self::Iter> {
        <(Ipv6Addr, u16) as std::net::ToSocketAddrs>::to_socket_addrs(self)
    }
}

impl ToSocketAddrs for SocketAddrV4 {
    type Iter = IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> Result<Self::Iter> {
        <SocketAddrV4 as std::net::ToSocketAddrs>::to_socket_addrs(self)
    }
}

impl ToSocketAddrs for SocketAddrV6 {
    type Iter = IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> Result<Self::Iter> {
        <SocketAddrV6 as std::net::ToSocketAddrs>::to_socket_addrs(self)
    }
}

impl<'a> ToSocketAddrs for &'a [SocketAddr] {
    type Iter = Cloned<Iter<'a, SocketAddr>>;
    fn to_socket_addrs(&self) -> Result<Self::Iter> {
        <&[SocketAddr] as std::net::ToSocketAddrs>::to_socket_addrs(self)
    }
}
