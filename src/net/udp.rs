use std::io::{Error, ErrorKind, Result};
use std::net::SocketAddr;

use super::SocketAddrIterator;
use crate::{error::LunaticError, host};

#[derive(Debug)]
pub struct UdpSocket {
    id: u64,
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        host::api::networking::drop_udp_socket(self.id);
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
                    host::api::networking::udp_bind(
                        4,
                        ip.as_ptr() as u32,
                        port,
                        0,
                        0,
                        &mut id as *mut u64 as u64,
                    )
                }
                SocketAddr::V6(v6_addr) => {
                    let ip = v6_addr.ip().octets();
                    let port = v6_addr.port() as u32;
                    let flow_info = v6_addr.flowinfo();
                    let scope_id = v6_addr.scope_id();
                    host::api::networking::tcp_bind(
                        6,
                        ip.as_ptr() as u32,
                        port,
                        flow_info,
                        scope_id,
                        &mut id as *mut u64 as u64,
                    )
                }
            };
            if result == 0 {
                return Ok(Self { id });
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
        let result = host::api::networking::udp_local_addr(
            self.id,
            &mut dns_iter_or_error_id as *mut u64 as u64,
        );
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
