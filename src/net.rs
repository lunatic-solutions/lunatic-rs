use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use std::io::{self, Error, ErrorKind, IoSlice, IoSliceMut, Read, Write};
use std::{ffi::c_void, fmt};

mod stdlib {
    use std::ffi::c_void;

    #[link(wasm_import_module = "lunatic")]
    extern "C" {
        pub fn tcp_bind_str(addr_ptr: *const u8, addr_len: usize, listener: *mut u32) -> u32;
        pub fn tcp_accept(listener: u32, tcp_socket: *mut u32) -> u32;
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
        pub fn tcp_stream_serialize(tcp_stream: u32) -> u64;
        pub fn tcp_stream_deserialize(tcp_stream: u64) -> u32;
    }
}

pub struct TcpListener {
    id: u32,
}

impl TcpListener {
    pub fn bind(addr: &str) -> Result<Self, u32> {
        let mut id = 0;
        let result =
            unsafe { stdlib::tcp_bind_str(addr.as_ptr(), addr.len(), &mut id as *mut u32) };
        if result == 0 {
            Ok(Self { id })
        } else {
            Err(result)
        }
    }

    pub fn accept(&self) -> Result<TcpStream, u32> {
        let mut tcp_stream_id = 0;
        let result = unsafe { stdlib::tcp_accept(self.id, &mut tcp_stream_id as *mut u32) };
        if result == 0 {
            // TODO: We never use socket_addr_id, this leaks the id.
            Ok(TcpStream { id: tcp_stream_id })
        } else {
            Err(result)
        }
    }
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        drop(self.id);
    }
}

#[derive(Clone)]
pub struct TcpStream {
    id: u32,
}

impl TcpStream {
    pub unsafe fn from_id(id: u32) -> Self {
        Self { id }
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        drop(self.id);
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
                self.id,
                bufs.as_ptr() as *const c_void,
                bufs.len(),
                &mut nwritten as *mut usize,
            )
        };
        if result == 0 {
            Ok(nwritten)
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
                self.id,
                bufs.as_mut_ptr() as *mut c_void,
                bufs.len(),
                &mut nread as *mut usize,
            )
        };
        if result == 0 {
            Ok(nread)
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
        let serialized_tcp_stream = unsafe { stdlib::tcp_stream_serialize(self.id) };
        serializer.serialize_u64(serialized_tcp_stream)
    }
}

struct TcpStreamVisitor {}

impl<'de> Visitor<'de> for TcpStreamVisitor {
    type Value = TcpStream;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an pointer to an id containing a  tcp_stream")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let tcp_stream_id = unsafe { stdlib::tcp_stream_deserialize(value) };
        unsafe { Ok(TcpStream::from_id(tcp_stream_id)) }
    }
}

impl<'de> Deserialize<'de> for TcpStream {
    fn deserialize<D>(deserializer: D) -> Result<TcpStream, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(TcpStreamVisitor {})
    }
}
