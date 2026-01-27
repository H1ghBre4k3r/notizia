//! Task handle for controlling spawned tasks.

use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

use crate::core::errors::SendResult;

/// Handle for a spawned task.
///
/// A `TaskHandle` is returned when spawning a task and provides methods to:
/// - Send messages to the task
/// - Wait for the task to complete
/// - Abort the task
///
/// The handle owns the sender side of the channel and the Tokio join handle.
///
/// # Example
///
/// ```no_run
/// # use notizia::prelude::*;
/// # #[derive(Task)]
/// # #[task(message = Signal)]
/// # struct Worker;
/// # impl Runnable<Signal> for Worker {
/// #     async fn start(&self) {}
/// # }
/// # #[derive(Clone)]
/// # enum Signal { Stop }
/// # #[tokio::main]
/// # async fn main() {
/// let worker = Worker;
/// let handle = spawn!(worker);
///
/// // Send a message
/// handle.send(Signal::Stop).unwrap();
///
/// // Wait for completion
/// handle.join().await;
/// # }
/// ```
pub struct TaskHandle<T>
where
    T: 'static,
{
    sender: UnboundedSender<T>,
    handle: JoinHandle<()>,
}

impl<T> TaskHandle<T>
where
    T: 'static,
{
    /// Create a new task handle.
    ///
    /// This is typically called by the generated code and not by user code directly.
    #[doc(hidden)]
    pub fn new(sender: UnboundedSender<T>, handle: JoinHandle<()>) -> Self {
        TaskHandle { sender, handle }
    }

    /// Wait for the task to complete.
    ///
    /// This method consumes the handle and awaits the completion of the task.
    /// If the task panics, the panic will be propagated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use notizia::prelude::*;
    /// # #[derive(Task)]
    /// # #[task(message = Signal)]
    /// # struct Worker;
    /// # impl Runnable<Signal> for Worker {
    /// #     async fn start(&self) {}
    /// # }
    /// # #[derive(Clone)]
    /// # enum Signal {}
    /// # #[tokio::main]
    /// # async fn main() {
    /// let worker = Worker;
    /// let handle = spawn!(worker);
    ///
    /// // Do some work...
    ///
    /// // Wait for task to finish
    /// handle.join().await;
    /// # }
    /// ```
    pub async fn join(self) {
        let _ = self.handle.await;
    }

    /// Send a message to the task.
    ///
    /// Returns `Ok(())` if the message was sent successfully, or an error
    /// containing the message if the receiver has been dropped.
    ///
    /// # Errors
    ///
    /// Returns [`SendError`](crate::core::errors::SendError) if the task has
    /// terminated and the receiver has been dropped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use notizia::prelude::*;
    /// # #[derive(Task)]
    /// # #[task(message = Signal)]
    /// # struct Worker;
    /// # impl Runnable<Signal> for Worker {
    /// #     async fn start(&self) {}
    /// # }
    /// # #[derive(Clone)]
    /// # enum Signal { Ping }
    /// # #[tokio::main]
    /// # async fn main() {
    /// let worker = Worker;
    /// let handle = spawn!(worker);
    ///
    /// // Send a message using the method
    /// handle.send(Signal::Ping).expect("send failed");
    ///
    /// // Or using the macro (equivalent)
    /// send!(handle, Signal::Ping).expect("send failed");
    /// # }
    /// ```
    pub fn send(&self, msg: T) -> SendResult<T> {
        self.sender.send(msg)
    }

    /// Abort the task immediately.
    ///
    /// This method forcefully terminates the task. The task will not have
    /// an opportunity to clean up resources or finish processing messages.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use notizia::prelude::*;
    /// # #[derive(Task)]
    /// # #[task(message = Signal)]
    /// # struct Worker;
    /// # impl Runnable<Signal> for Worker {
    /// #     async fn start(&self) { loop {} }
    /// # }
    /// # #[derive(Clone)]
    /// # enum Signal {}
    /// # #[tokio::main]
    /// # async fn main() {
    /// let worker = Worker;
    /// let handle = spawn!(worker);
    ///
    /// // Forcefully terminate the task
    /// handle.kill();
    /// # }
    /// ```
    pub fn kill(self) {
        self.handle.abort();
    }
}
