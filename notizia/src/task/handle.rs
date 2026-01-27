//! Task handle for controlling spawned tasks.

use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

use crate::core::errors::SendResult;
use crate::{ShutdownError, ShutdownResult, TerminateReason};

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
    handle: JoinHandle<TerminateReason>,
}

impl<T> TaskHandle<T>
where
    T: 'static,
{
    /// Create a new task handle.
    ///
    /// This is typically called by the generated code and not by user code directly.
    #[doc(hidden)]
    pub fn new(sender: UnboundedSender<T>, handle: JoinHandle<TerminateReason>) -> Self {
        TaskHandle { sender, handle }
    }

    /// Wait for the task to complete without signaling shutdown.
    ///
    /// This method does NOT close the message channel. It simply waits
    /// for the task to complete on its own. The task's `terminate()` hook
    /// will still be called when the task finishes.
    ///
    /// Use [`shutdown()`](Self::shutdown) to actively signal shutdown
    /// and enforce a timeout.
    ///
    /// Returns the reason the task terminated.
    ///
    /// # Errors
    ///
    /// Returns a [`JoinError`](tokio::task::JoinError) if the task was
    /// aborted or an unexpected error occurred (rare).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use notizia::prelude::*;
    /// # #[derive(Task)]
    /// # #[task(message = Signal)]
    /// # struct Worker;
    /// # impl Runnable<Signal> for Worker {
    /// #     async fn start(&self) {
    /// #         // Task stops on its own after some work
    /// #     }
    /// # }
    /// # #[derive(Clone)]
    /// # enum Signal {}
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), tokio::task::JoinError> {
    /// let worker = Worker;
    /// let handle = spawn!(worker);
    ///
    /// // Task will finish on its own
    /// let reason = handle.join().await?;
    ///
    /// println!("Task finished: {}", reason);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn join(self) -> Result<TerminateReason, tokio::task::JoinError> {
        self.handle.await
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

    /// Gracefully shutdown the task with a timeout.
    ///
    /// This method initiates a graceful shutdown by:
    /// 1. Closing the message channel (task receives `RecvError::Closed`)
    /// 2. Waiting for the task's `start()` to complete
    /// 3. Calling the task's `terminate()` hook
    /// 4. Enforcing the timeout; aborting if exceeded
    ///
    /// Returns the reason the task terminated (`Normal` or `Panic`).
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum duration to wait for `terminate()` to complete
    ///
    /// # Errors
    ///
    /// Returns [`ShutdownError::Timeout`] if `terminate()` takes longer than the timeout.
    /// In this case, the task is forcefully aborted.
    ///
    /// Returns [`ShutdownError::JoinError`] if an unexpected join error occurs.
    ///
    /// # Notes
    ///
    /// - This method works by dropping the sender, which closes the channel
    /// - If [`TaskRef`](crate::TaskRef) clones exist, they keep the channel alive
    /// - Tasks must handle `RecvError::Closed` to detect shutdown
    /// - Message queue draining is the task's responsibility
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use notizia::prelude::*;
    /// # use std::time::Duration;
    /// # #[derive(Task)]
    /// # #[task(message = Signal)]
    /// # struct Worker;
    /// # impl Runnable<Signal> for Worker {
    /// #     async fn start(&self) {
    /// #         loop {
    /// #             match recv!(self) {
    /// #                 Ok(_) => {}
    /// #                 Err(_) => break,
    /// #             }
    /// #         }
    /// #     }
    /// #     async fn terminate(&self, _reason: TerminateReason) {
    /// #         // Cleanup resources
    /// #     }
    /// # }
    /// # #[derive(Clone)]
    /// # enum Signal { Stop }
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ShutdownError> {
    /// let worker = Worker;
    /// let handle = spawn!(worker);
    ///
    /// // Send some work...
    /// // handle.send(Signal::Stop)?;
    ///
    /// // Gracefully shutdown with 5 second timeout
    /// match handle.shutdown(Duration::from_secs(5)).await {
    ///     Ok(TerminateReason::Normal) => println!("Clean shutdown"),
    ///     Ok(TerminateReason::Panic(msg)) => eprintln!("Task panicked: {}", msg),
    ///     Err(ShutdownError::Timeout) => eprintln!("Shutdown timed out"),
    ///     Err(e) => eprintln!("Shutdown error: {}", e),
    /// }
    /// # Ok(())
    /// # }
    ///    
    pub async fn shutdown(self, timeout: Duration) -> ShutdownResult {
        // Step 1: Close the channel to signal shutdown
        // When the sender is dropped, receivers get RecvError::Closed
        // Note: If TaskRef clones exist, they keep the channel alive
        drop(self.sender);

        // Step 2: Wait for the task to complete with timeout
        // The task will:
        //   - Complete start() (normally or with panic)
        //   - Call terminate(reason)
        //   - Return TerminateReason
        match tokio::time::timeout(timeout, self.handle).await {
            // Timeout succeeded, join succeeded - task completed
            Ok(Ok(reason)) => Ok(reason),

            // Timeout succeeded, but join failed
            // This shouldn't happen since we catch panics in __setup
            Ok(Err(join_err)) => Err(ShutdownError::JoinError(join_err)),

            // Timeout elapsed - terminate() took too long
            Err(_elapsed) => Err(ShutdownError::Timeout),
        }
    }

    /// Get a reference to this task.
    ///
    /// Returns a [`TaskRef`](super::TaskRef) that can be used to send messages to this task.
    /// This is useful for passing lightweight references to other tasks.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use notizia::prelude::*;
    /// # #[derive(Clone)]
    /// # enum Signal { Ping }
    /// # #[derive(Task)]
    /// # #[task(message = Signal)]
    /// # struct Worker;
    /// # impl Runnable<Signal> for Worker {
    /// #     async fn start(&self) {}
    /// # }
    /// # #[tokio::main]
    /// # async fn main() {
    /// let worker = Worker;
    /// let handle = spawn!(worker);
    ///
    /// // Get a reference to send to other tasks
    /// let task_ref = handle.this();
    /// # }
    /// ```
    pub fn this(&self) -> super::TaskRef<T> {
        super::TaskRef::new(self.sender.clone())
    }
}
