use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("Can't resolve address")]
    CanNotResolveAddress(u32),
}

#[derive(Error, Debug)]
pub enum TcpListenerError {
    #[error("Can't bind the socket")]
    CanNotBindingToSocket(u32),
}

#[derive(Error, Debug)]
pub enum TcpStreamError {
    #[error("Can't establish TCP connection")]
    CanNotEstablishTcpConnection(u32),
}
