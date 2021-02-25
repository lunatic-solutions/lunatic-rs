mod errors;
mod tcp_listener;
mod tcp_stream;

pub use tcp_listener::TcpListener;
pub use tcp_stream::TcpStream;

use anyhow::Result;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

mod stdlib {
    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn resolve(name_ptr: *const u8, name_len: usize, resolver_id: *mut u32) -> u32;
        pub fn resolve_next(
            resolver_id: u32,
            addr: *mut u8,
            addr_len: *mut usize,
            port: *mut u16,
            flowinfo: *mut u32,
            scope_id: *mut u32,
        ) -> u32;
        pub fn remove_resolver(listener_id: u32);
    }
}

pub struct SocketAddrIterator {
    resolver_id: u32,
}

impl Drop for SocketAddrIterator {
    fn drop(&mut self) {
        unsafe {
            stdlib::remove_resolver(self.resolver_id);
        }
    }
}

impl Iterator for SocketAddrIterator {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        let mut addr: [u8; 16] = [0; 16];
        let mut addr_len: usize = 0;
        let mut port: u16 = 0;
        let mut flowinfo: u32 = 0;
        let mut scope_id: u32 = 0;
        let next = unsafe {
            stdlib::resolve_next(
                self.resolver_id,
                addr.as_mut_ptr(),
                &mut addr_len as *mut usize,
                &mut port as *mut u16,
                &mut flowinfo as *mut u32,
                &mut scope_id as *mut u32,
            )
        };

        if next == 0 {
            match addr_len {
                4 => {
                    let ip = Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]);
                    let socket_addr = SocketAddrV4::new(ip, port);
                    Some(socket_addr.into())
                }
                16 => {
                    let ip = Ipv6Addr::from(addr);
                    let socket_addr = SocketAddrV6::new(ip, port, flowinfo, scope_id);
                    Some(socket_addr.into())
                }
                _ => unreachable!("A socket address can only be v4 or v6"),
            }
        } else {
            None
        }
    }
}

pub fn resolve(name: &str) -> Result<SocketAddrIterator> {
    let mut resolver_id: u32 = 0;
    let result =
        unsafe { stdlib::resolve(name.as_ptr(), name.len(), &mut resolver_id as *mut u32) };
    if result != 0 {
        Err(errors::ResolveError::CanNotResolveAddress(result).into())
    } else {
        Ok(SocketAddrIterator { resolver_id })
    }
}
