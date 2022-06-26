use std::{
    cell::UnsafeCell,
    io::{Error, ErrorKind, Result},
    net::SocketAddr,
    time::Duration,
};

use super::SocketAddrIterator;
use crate::{error::LunaticError, host};

/// A UDP socket.
///
/// After creating a `UdpSocket` by [`bind`]ing it to a socket address, data can be
/// [sent to] and [received from] any other socket address.
///
/// Although UDP is a connectionless protocol, this implementation provides an interface
/// to set an address where data should be sent and received from. After setting a remote
/// address with [`connect`], data can be sent to and received from that address with
/// [`send`] and [`recv`].
///
/// As stated in the User Datagram Protocol's specification in [IETF RFC 768], UDP is
/// an unordered, unreliable protocol; refer to [`TcpListener`] and [`TcpStream`] for TCP
/// primitives.
///
/// [`bind`]: UdpSocket::bind
/// [`connect`]: UdpSocket::connect
/// [IETF RFC 768]: https://tools.ietf.org/html/rfc768
/// [`recv`]: UdpSocket::recv
/// [received from]: UdpSocket::recv_from
/// [`send`]: UdpSocket::send
/// [sent to]: UdpSocket::send_to
/// [`TcpListener`]: crate::net::TcpListener
/// [`TcpStream`]: crate::net::TcpStream
///
/// # Examples
///
/// ```no_run
/// use lunatic::net::UdpSocket;
///
/// #[lunatic::main]
/// fn main(_: Mailbox<()>) -> std::io::Result<()> {
///     {
///         let socket = UdpSocket::bind("127.0.0.1:34254")?;
///
///         // Receives a single datagram message on the socket. If `buf` is too small to hold
///         // the message, it will be cut off.
///         let mut buf = [0; 10];
///         let (amt, src) = socket.recv_from(&mut buf)?;
///
///         // Redeclare `buf` as slice of the received data and send reverse data back to origin.
///         let buf = &mut buf[..amt];
///         buf.reverse();
///         socket.send_to(buf, &src)?;
///     } // the socket is closed here
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct UdpSocket {
    id: u64,

    // Issue - https://github.com/lunatic-solutions/lunatic/issues/95
    read_timeout: u32,  // ms
    write_timeout: u32, // ms

    // If the UDP Socket is serialized it will be removed from our resources, so we can't call
    // `drop_udp_socket()` anymore on it.
    consumed: UnsafeCell<bool>,
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        // Only drop stream if it's not already consumed
        if unsafe { !*self.consumed.get() } {
            unsafe { host::api::networking::drop_udp_socket(self.id) };
        }
    }
}

impl UdpSocket {
    /// Sets the read timeout.
    ///
    /// If the value specified is `None`, then read calls will block indefinitely.
    pub fn set_read_timeout(&mut self, duration: Option<Duration>) -> Result<()> {
        match duration {
            None => self.read_timeout = 0,
            Some(duration) => self.read_timeout = duration.as_millis() as u32,
        };

        Ok(())
    }
    /// Gets the read timeout.
    ///
    /// If the value returned is `None`, then read calls will block indefinitely.
    pub fn read_timeout(&self) -> Result<Option<Duration>> {
        let result = match self.read_timeout {
            0 => None,
            _ => Some(Duration::from_millis(self.read_timeout.into())),
        };

        Ok(result)
    }
    /// Sets the write timeout.
    ///
    /// If the value specified is `None`, then write calls will block indefinitely.
    pub fn set_write_timeout(&mut self, duration: Option<Duration>) -> Result<()> {
        match duration {
            None => self.write_timeout = 0,
            Some(duration) => self.write_timeout = duration.as_millis() as u32,
        };

        Ok(())
    }
    /// Sets the write timeout.
    ///
    /// If the value specified is `None`, then write calls will block indefinitely.
    pub fn write_timeout(&self) -> Result<Option<Duration>> {
        let result = match self.write_timeout {
            0 => None,
            _ => Some(Duration::from_millis(self.write_timeout.into())),
        };

        Ok(result)
    }
    /// Creates a new [`UdpSocket`] bound to the given address.
    ///
    /// Binding with a port number of 0 will request that the operating system assigns an available
    /// port to this listener.
    ///
    /// If `addr` yields multiple addresses, binding will be attempted with each of the addresses
    /// until one succeeds and returns the listener. If none of the addresses succeed in creating a
    /// listener, the error from the last attempt is returned.
    pub fn bind<A>(addr: A) -> Result<Self>
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
                        host::api::networking::udp_bind(
                            4,
                            ip.as_ptr(),
                            port,
                            0,
                            0,
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
                        host::api::networking::udp_bind(
                            6,
                            ip.as_ptr(),
                            port,
                            flow_info,
                            scope_id,
                            &mut id as *mut u64,
                        )
                    }
                }
            };
            if result == 0 {
                return Ok(Self {
                    id,
                    read_timeout: 0,
                    write_timeout: 0,
                    consumed: UnsafeCell::new(false),
                });
            }
        }
        let lunatic_error = LunaticError::from(id);
        Err(Error::new(ErrorKind::Other, lunatic_error))
    }
    /// Returns the local address that this UdpSocket is bound to.
    ///
    /// This can be useful, for example, to identify when binding to port 0 which port was assigned by the OS.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        let mut dns_iter_or_error_id = 0;
        let result = unsafe {
            host::api::networking::udp_local_addr(self.id, &mut dns_iter_or_error_id as *mut u64)
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
    /// Connects this UDP socket to a remote address, allowing the `send` and
    /// `recv` syscalls to be used to send data and also applies filters to only
    /// receive data from the specified address.
    ///
    /// If `addr` yields multiple addresses, `connect` will be attempted with
    /// each of the addresses until the underlying OS function returns no
    /// error. Note that usually, a successful `connect` call does not specify
    /// that there is a remote server listening on the port, rather, such an
    /// error would only be detected after the first send. If the OS returns an
    /// error for each of the specified addresses, the error returned from the
    /// last connection attempt (the last address) is returned.
    ///
    /// # Examples
    ///
    /// Creates a UDP socket bound to `127.0.0.1:3400` and connect the socket to
    /// `127.0.0.1:8080`:
    ///
    /// ```no_run
    /// use lunatic::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:3400").expect("couldn't bind to address");
    /// socket.connect("127.0.0.1:8080").expect("connect function failed");
    /// ```
    ///
    /// Unlike in the TCP case, passing an array of addresses to the `connect`
    /// function of a UDP socket is not a useful thing to do: The OS will be
    /// unable to determine whether something is listening on the remote
    /// address without the application sending data.    
    pub fn connect<A>(&self, addr: A) -> Result<()>
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
                        host::api::networking::udp_connect(
                            self.id,
                            4,
                            ip.as_ptr(),
                            port,
                            0,
                            0,
                            0, // timeout_ms
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
                        host::api::networking::udp_connect(
                            self.id,
                            6,
                            ip.as_ptr(),
                            port,
                            flow_info,
                            scope_id,
                            0, // timeout_ms
                            &mut id as *mut u64,
                        )
                    }
                }
            };
            if result == 0 {
                //self.id = id;
                return Ok(());
            }
        }
        let lunatic_error = LunaticError::from(id);
        Err(Error::new(ErrorKind::Other, lunatic_error))
    }
    /// Sends data on the socket to the remote address to which it is connected.
    ///
    /// [`UdpSocket::connect`] will connect this socket to a remote address. This
    /// method will fail if the socket is not connected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lunatic::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.connect("127.0.0.1:8080").expect("connect function failed");
    /// socket.send(&[0, 1, 2]).expect("couldn't send message");
    /// ```
    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        let mut nsend_or_error_id: u64 = 0;
        let result = unsafe {
            host::api::networking::udp_send(
                self.id,
                buf.as_ptr(),
                buf.len(),
                self.write_timeout,
                &mut nsend_or_error_id as *mut u64,
            )
        };
        if result == 0 {
            Ok(nsend_or_error_id as usize)
        } else {
            let lunatic_error = LunaticError::from(nsend_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }
    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    ///
    /// Address type can be any implementor of [`super::ToSocketAddrs`] trait. See its
    /// documentation for concrete examples.
    ///
    /// It is possible for `addr` to yield multiple addresses, but `send_to`
    /// will only send data to the first address yielded by `addr`.
    ///
    /// This will return an error when the IP version of the local socket
    /// does not match that returned from [`super::ToSocketAddrs`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lunatic::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.send_to(&[0; 10], "127.0.0.1:4242").expect("couldn't send data");
    /// ```
    pub fn send_to<A>(&self, buf: &[u8], addr: A) -> Result<usize>
    where
        A: super::ToSocketAddrs,
    {
        let mut nsend_or_error_id = 0;
        for addr in addr.to_socket_addrs()? {
            let result = match addr {
                SocketAddr::V4(v4_addr) => {
                    let ip = v4_addr.ip().octets();
                    let port = v4_addr.port() as u32;
                    unsafe {
                        host::api::networking::udp_send_to(
                            self.id,
                            buf.as_ptr(),
                            buf.len(),
                            4,
                            ip.as_ptr(),
                            port,
                            0,
                            0,
                            self.write_timeout,
                            &mut nsend_or_error_id as *mut u64,
                        )
                    }
                }
                SocketAddr::V6(v6_addr) => {
                    let ip = v6_addr.ip().octets();
                    let port = v6_addr.port() as u32;
                    let flow_info = v6_addr.flowinfo();
                    let scope_id = v6_addr.scope_id();
                    unsafe {
                        host::api::networking::udp_send_to(
                            self.id,
                            buf.as_ptr(),
                            buf.len(),
                            6,
                            ip.as_ptr(),
                            port,
                            flow_info,
                            scope_id,
                            self.write_timeout,
                            &mut nsend_or_error_id as *mut u64,
                        )
                    }
                }
            };
            if result == 0 {
                return Ok(nsend_or_error_id as usize);
            }
        }
        let lunatic_error = LunaticError::from(nsend_or_error_id);
        Err(Error::new(ErrorKind::Other, lunatic_error))
    }
    /// Receives a single datagram message on the socket from the remote address to
    /// which it is connected. On success, returns the number of bytes read.
    ///
    /// The function must be called with valid byte array `buf` of sufficient size to
    /// hold the message bytes. If a message is too long to fit in the supplied buffer,
    /// excess bytes may be discarded.
    ///
    /// [`UdpSocket::connect`] will connect this socket to a remote address. This
    /// method will fail if the socket is not connected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lunatic::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.connect("127.0.0.1:8080").expect("connect function failed");
    /// let mut buf = [0; 10];
    /// match socket.recv(&mut buf) {
    ///     Ok(received) => println!("received {received} bytes {:?}", &buf[..received]),
    ///     Err(e) => println!("recv function failed: {e:?}"),
    /// }
    /// ```
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let mut nrecv_or_error_id: u64 = 0;
        let result = unsafe {
            host::api::networking::udp_receive(
                self.id,
                buf.as_mut_ptr(),
                buf.len(),
                self.read_timeout,
                &mut nrecv_or_error_id as *mut u64,
            )
        };
        if result == 0 {
            Ok(nrecv_or_error_id as usize)
        } else {
            let lunatic_error = LunaticError::from(nrecv_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }
    /// Receives a single datagram message on the socket. On success, returns the number
    /// of bytes read and the origin.
    ///
    /// The function must be called with valid byte array `buf` of sufficient size to
    /// hold the message bytes. If a message is too long to fit in the supplied buffer,
    /// excess bytes may be discarded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lunatic::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// let mut buf = [0; 10];
    /// let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
    ///                                         .expect("Didn't receive data");
    /// let filled_buf = &mut buf[..number_of_bytes];
    /// ```
    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let mut dns_iter_id = 0;
        let mut nrecv_or_error_id: u64 = 0;
        let result = unsafe {
            host::api::networking::udp_receive_from(
                self.id,
                buf.as_mut_ptr(),
                buf.len(),
                self.read_timeout,
                &mut nrecv_or_error_id as *mut u64,
                &mut dns_iter_id as *mut u64,
            )
        };
        if result == 0 {
            let mut dns_iter = SocketAddrIterator::from(dns_iter_id);
            let peer = dns_iter.next().expect("must contain one element");
            Ok((nrecv_or_error_id as usize, peer))
        } else {
            let lunatic_error = LunaticError::from(nrecv_or_error_id);
            Err(Error::new(ErrorKind::Other, lunatic_error))
        }
    }
    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent
    /// from this socket.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lunatic::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_ttl(42).expect("set_ttl call failed");
    /// ```
    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        // no result for this? it's () ?
        unsafe { host::api::networking::set_udp_socket_ttl(self.id, ttl) };
        // there is no error for this
        Ok(())
    }
    /// Sets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// When enabled, this socket is allowed to send packets to a broadcast
    /// address.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lunatic::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_broadcast(false).expect("set_broadcast call failed");
    /// ```
    pub fn set_broadcast(&self, broadcast: bool) -> Result<()> {
        // no result for this? it's () ?
        let api_broadcast = match broadcast {
            true => 1,
            false => 0,
        };
        unsafe { host::api::networking::set_udp_socket_broadcast(self.id, api_broadcast) };
        // there is no error for this
        Ok(())
    }
    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [`UdpSocket::set_ttl`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lunatic::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_ttl(42).expect("set_ttl call failed");
    /// assert_eq!(socket.ttl().unwrap(), 42);
    /// ```
    pub fn ttl(&self) -> Result<u32> {
        // there is no error for this?
        let result = unsafe { host::api::networking::get_udp_socket_ttl(self.id) };
        Ok(result)
    }
    /// Gets the value of the `SO_BROADCAST` option for this socket.
    ///
    /// For more information about this option, see [`UdpSocket::set_broadcast`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lunatic::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// socket.set_broadcast(false).expect("set_broadcast call failed");
    /// assert_eq!(socket.broadcast().unwrap(), false);
    /// ```
    pub fn broadcast(&self) -> Result<bool> {
        let result = unsafe { host::api::networking::get_udp_socket_broadcast(self.id) };
        match result {
            0 => Ok(false),
            _ => Ok(true),
        }
    }
    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned `UdpSocket` is a reference to the same socket that this
    /// object references. Both handles will read and write the same port, and
    /// options set on one socket will be propagated to the other.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lunatic::net::UdpSocket;
    ///
    /// let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    /// let socket_clone = socket.try_clone().expect("couldn't clone the socket");
    /// ```
    pub fn try_clone(&self) -> Result<UdpSocket> {
        let result = unsafe { host::api::networking::clone_udp_socket(self.id) };
        Ok(Self {
            id: result,
            read_timeout: 0,
            write_timeout: 0,
            consumed: UnsafeCell::new(false),
        })
    }
    /// Dummy fn - This is just to make porting from std easier?
    pub fn set_nonblocking(&self, _: bool) -> Result<()> {
        Ok(())
    }
    /// Dummy fn - This is just to make porting from std easier?
    pub fn take_error(&self) -> Result<Option<LunaticError>> {
        Ok(None)
    }
}
