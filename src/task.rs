use std::marker::PhantomData;

use super::Process;
use crate::{host, Mailbox, ProcessConfig, Resource, Tag};

/// A one-off process spawned from a function that can capture some input from the parent and send
/// back a result.
///
/// When [`result`](Task::result) is called it will block until the async computation is done and
/// a result available. If you don't want to wait on the result use a regular
/// [`Process`](crate::Process).
///
/// # Example
///
/// ```
/// // Run computation in different process.
/// let task = Task::spawn_link((2, 3), |(a, b)| a + b);
/// // Wait for process to finish and get the result.
/// let result = task.result();
/// assert_eq!(result, 5);
/// ```
#[must_use = "If `result()` is not called on `Task` it will leak memory when dropped."]
pub struct Task<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    process: Process<()>,
    // A tag is used to match the return message to the correct task.
    tag: Tag,
    result_received: bool,
    phantom: PhantomData<T>,
}

impl<T> Task<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn spawn_link<C>(capture: C, entry: fn(C) -> T) -> Task<T>
    where
        C: serde::Serialize + serde::de::DeserializeOwned,
    {
        Task::spawn_link_(capture, entry, None)
    }

    pub fn spawn_link_config<C>(config: &ProcessConfig, capture: C, entry: fn(C) -> T) -> Task<T>
    where
        C: serde::Serialize + serde::de::DeserializeOwned,
    {
        Task::spawn_link_(capture, entry, Some(config))
    }

    fn spawn_link_<C>(capture: C, entry: fn(C) -> T, config: Option<&ProcessConfig>) -> Task<T>
    where
        C: serde::Serialize + serde::de::DeserializeOwned,
    {
        let tag = Tag::new();
        let parent = unsafe { <Process<T> as Resource>::from_id(host::api::process::this()) };
        let process = if let Some(config) = config {
            Process::<()>::spawn_link_config(
                config,
                (parent, tag, capture, entry as usize as i32),
                Task::starter::<C>,
            )
        } else {
            Process::<()>::spawn_link(
                (parent, tag, capture, entry as usize as i32),
                Task::starter::<C>,
            )
        };

        Task {
            process,
            tag: tag,
            result_received: false,
            phantom: PhantomData,
        }
    }

    // Entry point of the child process. This will call the passed in `entry` function.
    fn starter<C>((parent, tag, capture, entry): (Process<T>, Tag, C, i32), _: Mailbox<()>) {
        let entry: fn(C) -> T = unsafe { std::mem::transmute(entry) };
        let result = entry(capture);
        parent.tag_send(tag, result);
    }

    /// Returns a globally unique process ID.
    pub fn uuid(&self) -> u128 {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host::api::process::id(self.process.id(), &mut uuid as *mut [u8; 16]) };
        u128::from_le_bytes(uuid)
    }

    /// Wait for the result of the task.
    ///
    /// This function will block until the task returns a result. It must be called on all tasks
    /// or the returned result will stay forever inside the mailbox.
    pub fn result(mut self) -> T {
        self.result_received = true;
        unsafe { Mailbox::<T>::new() }.tag_receive(Some(&[self.tag]))
    }
}

// Processes are equal if their UUID is equal.
impl<T> PartialEq for Task<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

impl<T> std::fmt::Debug for Task<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("uuid", &self.uuid())
            .finish()
    }
}

impl<T> Drop for Task<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    #[track_caller]
    fn drop(&mut self) {
        if !self.result_received {
            eprintln!("warning: Dropping `Task<T>` without consuming it first with `result()` will leak memory.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lunatic_test::test;

    #[test]
    fn spawn_test() {
        let task = Task::spawn_link(1, |capture| {
            assert_eq!(capture, 1);
            2
        });
        assert_eq!(task.result(), 2);
    }

    #[test]
    #[should_panic]
    fn spawn_link_test() {
        let task = Task::spawn_link((), |_| {
            panic!("fails");
        });
        task.result();
    }
}
