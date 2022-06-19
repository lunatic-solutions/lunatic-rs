use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    time::Duration,
};

use crate::{error::LunaticError, host};

/// Iterator over [`SocketAddr`]
#[derive(Debug)]
pub struct SocketAddrIterator {
    id: u64,
}

impl SocketAddrIterator {
    pub(crate) fn from(id: u64) -> Self {
        Self { id }
    }
}

impl Drop for SocketAddrIterator {
    fn drop(&mut self) {
        host::api::networking::drop_dns_iterator(self.id);
    }
}

impl Iterator for SocketAddrIterator {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        let mut addr_type: u32 = 0;
        let mut addr: [u8; 16] = [0; 16];
        let mut port: u16 = 0;
        let mut flowinfo: u32 = 0;
        let mut scope_id: u32 = 0;
        let next = host::api::networking::resolve_next(
            self.id,
            &mut addr_type as *mut u32 as u32,
            addr.as_mut_ptr() as u32,
            &mut port as *mut u16 as u16,
            &mut flowinfo as *mut u32 as u32,
            &mut scope_id as *mut u32 as u32,
        );

        if next == 0 {
            match addr_type {
                4 => {
                    let ip = Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]);
                    let socket_addr = SocketAddrV4::new(ip, port);
                    Some(socket_addr.into())
                }
                6 => {
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

/// Performs a DNS resolution.
///
/// The returned iterator may not actually yield any values depending on the
/// outcome of any resolution performed.
pub fn resolve(name: &str) -> Result<SocketAddrIterator, LunaticError> {
    resolve_timeout_(name, None)
}

/// Same as [`resolve`], but only waits for the duration of timeout to resolve.
pub fn resolve_timeout(name: &str, timeout: Duration) -> Result<SocketAddrIterator, LunaticError> {
    resolve_timeout_(name, Some(timeout))
}

fn resolve_timeout_(
    name: &str,
    timeout: Option<Duration>,
) -> Result<SocketAddrIterator, LunaticError> {
    let mut dns_iter_or_error_id: u64 = 0;
    let timeout_ms = match timeout {
        // If waiting time is smaller than 1ms, round it up to 1ms.
        Some(timeout) => match timeout.as_millis() {
            0 => 1,
            other => other as u32,
        },
        None => 0,
    };
    let result = host::api::networking::resolve(
        name.as_ptr() as u32,
        name.len() as u32,
        timeout_ms,
        &mut dns_iter_or_error_id as *mut u64 as u64,
    );
    if result != 0 {
        Err(LunaticError::from(dns_iter_or_error_id))
    } else {
        Ok(SocketAddrIterator {
            id: dns_iter_or_error_id,
        })
    }
}
