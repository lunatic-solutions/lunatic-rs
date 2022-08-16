use std::{
    cell::UnsafeCell,
    io::{Error, ErrorKind, IoSlice, Read, Result, Write},
    net::SocketAddr,
    time::Duration,
};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{self as lunatic, error::LunaticError, host, spawn_link, Mailbox, Process};

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
    // If the TCP stream is serialized it will be removed from our resources, so we can't call
    // `drop_tcp_stream()` anymore on it.
    consumed: UnsafeCell<bool>,
    // Duration of the read timeout for this particular TcpStream. If set to `None` then the read blocks indefinitely
    // If within the given `Duration` no data is read from the TcpStream an Error with `ErrorKind::TimedOut` is returned
    read_timeout: Option<Duration>,
    // Duration of the write timeout for this particular TcpStream. If set to `None` then the read blocks indefinitely
    // If within the given `Duration` no data is written to the TcpStream an Error with `ErrorKind::TimedOut` is returned
    write_timeout: Option<Duration>,
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        // Only drop stream if it's not already consumed
        if unsafe { !*self.consumed.get() } {
            unsafe { host::api::networking::drop_tcp_stream(self.id) };
        }
    }
}

impl Clone for TcpStream {
    fn clone(&self) -> Self {
        let id = unsafe { host::api::networking::clone_tcp_stream(self.id) };
        Self {
            id,
            consumed: UnsafeCell::new(false),
            read_timeout: None,
            write_timeout: None,
        }
    }
}

impl Serialize for TcpStream {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Mark process as consumed
        unsafe { *self.consumed.get() = true };
        // TODO: Timeout info is not serialized
        let index = unsafe { host::api::message::push_tcp_stream(self.id) };
        serializer.serialize_u64(index)
    }
}
struct TcpStreamVisitor;
impl<'de> Visitor<'de> for TcpStreamVisitor {
    type Value = TcpStream;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an u64 index")
    }

    fn visit_u64<E>(self, index: u64) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        let id = unsafe { host::api::message::take_tcp_stream(index) };
        Ok(TcpStream::from(id))
    }
}

impl<'de> Deserialize<'de> for TcpStream {
    fn deserialize<D>(deserializer: D) -> std::result::Result<TcpStream, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(TcpStreamVisitor)
    }
}

impl TcpStream {
    pub(crate) fn from(id: u64) -> Self {
        TcpStream {
            id,
            consumed: UnsafeCell::new(false),
            read_timeout: None,
            write_timeout: None,
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
        A: super::ToSocketAddrs,
    {
        TcpStream::connect_timeout_(addr, None)
    }

    /// Same as [`TcpStream::connect`], but only waits for the duration of timeout to connect.
    pub fn connect_timeout<A>(addr: A, timeout: Duration) -> Result<Self>
    where
        A: super::ToSocketAddrs,
    {
        TcpStream::connect_timeout_(addr, Some(timeout))
    }

    fn connect_timeout_<A>(addr: A, timeout: Option<Duration>) -> Result<Self>
    where
        A: super::ToSocketAddrs,
    {
        let mut id = 0;
        for addr in addr.to_socket_addrs()? {
            let timeout_ms = match timeout {
                Some(timeout) => timeout.as_millis() as u64,
                None => u64::MAX,
            };
            let result = match addr {
                SocketAddr::V4(v4_addr) => {
                    let ip = v4_addr.ip().octets();
                    let port = v4_addr.port() as u32;
                    unsafe {
                        host::api::networking::tcp_connect(
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
                        host::api::networking::tcp_connect(
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

    pub fn set_write_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        self.write_timeout = dur;
        Ok(())
    }

    pub fn set_read_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        // Only drop stream if it's not already consumed
        if unsafe { !*self.consumed.get() } {
            self.read_timeout = dur;
            return Ok(());
        }
        Err(Error::from(ErrorKind::InvalidInput))
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // if let Some(duration) = self.read_timeout {}
        let io_slice = IoSlice::new(buf);
        self.write_vectored(&[io_slice])
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> {
        let mut nwritten_or_error_id: u64 = 0;
        let result = unsafe {
            host::api::networking::tcp_write_vectored(
                self.id,
                bufs.as_ptr() as *const u32,
                bufs.len(),
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
        match unsafe { host::api::networking::tcp_flush(self.id, &mut error_id as *mut u64) } {
            0 => Ok(()),
            _ => {
                let lunatic_error = LunaticError::from(error_id);
                Err(Error::new(ErrorKind::Other, lunatic_error))
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) enum TcpTimeoutResponse {
    Read(Vec<u8>),
    ReadError(u64),
    Write(usize),
    WriteError(u64),
    TimedOut,
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let stream = self.clone();
        if let Some(duration) = self.read_timeout {
            let len = buf.len();
            let reader_process = Process::spawn_link(
                (stream, len, duration),
                |(stream, len, duration),

                 protocol: lunatic::protocol::Protocol<
                    lunatic::protocol::Send<_, lunatic::protocol::TaskEnd>,
                >| {
                    let mailbox: Mailbox<TcpTimeoutResponse> = unsafe { Mailbox::new() };
                    let _ = protocol.send(read_with_timeout(stream, len, duration, mailbox));
                },
            );
            return match reader_process.result() {
                TcpTimeoutResponse::Read(new_buf) => {
                    let old_len = buf.len();
                    buf[old_len..old_len + new_buf.len()].copy_from_slice(new_buf.as_slice());
                    Ok(new_buf.len())
                }
                TcpTimeoutResponse::TimedOut => Err(Error::from(ErrorKind::TimedOut)),
                TcpTimeoutResponse::Write(size) => Ok(size),
                TcpTimeoutResponse::ReadError(code) | TcpTimeoutResponse::WriteError(code) => {
                    let lunatic_error = LunaticError::from(code);
                    Err(Error::new(ErrorKind::Other, lunatic_error))
                }
            };
        }
        // if no timeout was provided, continue with default blocking behaviour
        let mut nread_or_error_id: u64 = 0;
        let result = unsafe {
            host::api::networking::tcp_read(
                self.id,
                buf.as_mut_ptr(),
                buf.len(),
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

fn read_with_timeout(
    stream: TcpStream,
    len: usize,
    duration: Duration,
    mailbox: Mailbox<TcpTimeoutResponse>,
) -> TcpTimeoutResponse {
    // invoke timeout call
    let this = mailbox.this();
    this.send_after(TcpTimeoutResponse::TimedOut, duration);
    let _ = spawn_link!(@task |stream, len, this| {
        let mut buf = Vec::with_capacity(len);
        let mut nread_or_error_id: u64 = 0;
        let result = unsafe {
            host::api::networking::tcp_read(
                stream.id,
                buf.as_mut_ptr(),
                len,
                &mut nread_or_error_id as *mut u64,
            )
        };
        if result == 0 {
            this.send(TcpTimeoutResponse::Read(buf));
        } else {
            this.send(TcpTimeoutResponse::ReadError(nread_or_error_id));
        }
    });
    mailbox.receive()
}

// TODO: needs to serialize IoSlice
// fn write_with_timeout(
//     stream: TcpStream,
//     bufs: &[IoSlice<'_>],
//     duration: Duration,
//     mailbox: Mailbox<TcpTimeoutResponse>,
// ) -> TcpTimeoutResponse {
//     // invoke timeout call
//     let this = mailbox.this();
//     this.send_after(TcpTimeoutResponse::TimedOut, duration);
//     let _ = spawn_link!(@task |stream, bufs, this| {
//             let mut nwritten_or_error_id: u64 = 0;
//             let result = unsafe {
//                 host::api::networking::tcp_write_vectored(
//                     stream.id,
//                     bufs.as_ptr() as *const u32,
//                     bufs.len(),
//                     &mut nwritten_or_error_id as *mut u64,
//                 )
//             };
//             if result == 0 {
//                 this.send(TcpTimeoutResponse::Write(result as usize));
//             } else {
//                 this.send(TcpTimeoutResponse::ReadError(nwritten_or_error_id));
//             }
//     });
//     mailbox.receive()
// }
