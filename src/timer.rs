use crate::host;

/// A reference to a timer created from send_after.
#[derive(Clone, Copy)]
pub struct TimerRef(u64);

impl TimerRef {
    pub(crate) fn new(timer_id: u64) -> Self {
        TimerRef(timer_id)
    }

    /// Cancel the timer, blocking until the timer is canceled.
    pub fn cancel(self) -> bool {
        host::api::timer::cancel_timer(self.0) == 1
    }
}
