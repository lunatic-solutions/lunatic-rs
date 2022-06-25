use std::{
    cell::UnsafeCell,
    io::{Error, ErrorKind, Result},
    net::SocketAddr,
};

use super::SocketAddrIterator;
use crate::{error::LunaticError, host};

#[derive(Debug)]
pub struct UdpSocket {
    id: u64,

    // Issue - https://github.com/lunatic-solutions/lunatic/issues/95
    // read_timeout: u32,  // ms
    // write_timeout: u32, // ms

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
                0, // self.write_timeout ?
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
                0, // self.read_timeout ?
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
                0, // self.read_timeout ?
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
}
