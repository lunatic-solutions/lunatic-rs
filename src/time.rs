//! Contains helper structures to deal with time-related functionality.

use std::time::Duration;

use crate::ap::messages::{RequestMessage, ShutdownMessage};
use crate::ap::{AbstractProcess, DeferredRequestHandler, ProcessRef, RequestHandler};
use crate::host;
use crate::serializer::CanSerialize;

/// A reference to a timer created from send_after.
#[derive(Clone, Copy)]
pub struct TimerRef(u64);

impl TimerRef {
    pub(crate) fn new(timer_id: u64) -> Self {
        TimerRef(timer_id)
    }

    /// Cancel the timer, blocking until the timer is canceled.
    pub fn cancel(self) -> bool {
        unsafe { host::api::timer::cancel_timer(self.0) == 1 }
    }
}

/// Modifies `T` so that all functions on it will return a timeout.
///
/// It's used to time out calls such as [`ProcessRef::shutdown`],
/// [`ProcessRef::request`], etc.
pub struct WithTimeout<T> {
    timeout: Duration,
    item: T,
}

impl<T: AbstractProcess> WithTimeout<ProcessRef<T>> {
    pub fn from(timeout: Duration, item: ProcessRef<T>) -> Self {
        Self { timeout, item }
    }

    /// Shuts the [`AbstractProcess`] down.
    ///
    /// The function will only wait for the duration of the specified timeout on
    /// the process to shut down, before returning `Err(Timeout)`.
    #[track_caller]
    pub fn shutdown(&self) -> Result<(), Timeout>
    where
        // The serializer needs to be able to serialize values of `ShutdownMessage` & `()` for the
        // return value.
        T::Serializer: CanSerialize<ShutdownMessage<T::Serializer>>,
        T::Serializer: CanSerialize<()>,
    {
        self.item.shutdown_timeout(Some(self.timeout))
    }

    /// Make a request to the process.
    ///
    /// The function will only wait for the duration of the specified timeout on
    /// the response, before returning `Err(Timeout)`.
    #[track_caller]
    pub fn request<R: 'static>(&self, request: R) -> Result<T::Response, Timeout>
    where
        T: RequestHandler<R>,
        T::Serializer: CanSerialize<R>,
        T::Serializer: CanSerialize<T::Response>,
        T::Serializer: CanSerialize<RequestMessage<R, T::Response, T::Serializer>>,
    {
        self.item.request_timeout(request, Some(self.timeout))
    }

    /// Make a deferred request to the process.
    ///
    /// The function will only wait for the duration of the specified timeout on
    /// the response, before returning `Err(Timeout)`.
    #[track_caller]
    pub fn deferred_request<R: 'static>(&self, request: R) -> Result<T::Response, Timeout>
    where
        T: DeferredRequestHandler<R>,
        T::Serializer: CanSerialize<R>,
        T::Serializer: CanSerialize<T::Response>,
        T::Serializer: CanSerialize<RequestMessage<R, T::Response, T::Serializer>>,
    {
        self.item
            .deferred_request_timeout(request, Some(self.timeout))
    }
}

/// Error result for [`ProcessRef::shutdown`] & [`ProcessRef::request`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Timeout;

/// Modifies `T` so that all functions on it will be performed with a delay.
///
/// It's used to delay calls such as [`ProcessRef::send`].
pub struct WithDelay<T> {
    duration: Duration,
    item: T,
}

impl<T: AbstractProcess> WithDelay<ProcessRef<T>> {
    pub fn from(duration: Duration, item: ProcessRef<T>) -> Self {
        Self { duration, item }
    }

    /// Send message to the process after the specified duration has passed.
    ///
    /// This is a non-blocking function, meaning that `send` is going to be
    /// performed in the background while the execution continues. The call will
    /// return a reference to the timer allowing you to cancel the operation.
    #[track_caller]
    pub fn send<M: 'static>(&self, message: M) -> TimerRef
    where
        T::Serializer: CanSerialize<M>,
    {
        self.item.delayed_send(message, self.duration)
    }
}
