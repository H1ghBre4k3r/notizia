//! Core traits for task behavior.

use std::future::Future;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::core::errors::RecvResult;
use crate::{TerminateReason, core::Mailbox};

use super::{TaskHandle, TaskRef};

/// User-facing trait for implementing task logic.
///
/// This trait must be implemented for any type that wants to act as a task.
/// The [`start`](Self::start) method contains the main logic of the task and
/// will be called when the task is spawned.
///
/// # Example
///
/// ```ignore
/// # TODO: Re-enable once derive macro hygiene is fixed
/// use notizia::prelude::*;
/// # #[derive(Debug, Clone)]
/// # enum Signal { Stop }
///
/// #[derive(Task)]
/// #[task(message = Signal)]
/// struct Worker {
///     id: usize,
/// }
///
/// impl Runnable<Signal> for Worker {
///     async fn start(&self) {
///         loop {
///             match recv!(self) {
///                 Ok(msg) => println!("Worker {} received {:?}", self.id, msg),
///                 Err(_) => break,
///             }
///         }
///     }
/// }
/// ```
pub trait Runnable<T>: Send + Sync {
    /// The main logic of the task.
    ///
    /// This method is called when the task is spawned and should contain
    /// the task's event loop or main logic.
    fn start(&self) -> impl Future<Output = ()> + Send;

    /// Cleanup hook called when the task is terminating.
    ///
    /// This method is called automatically after `start()` completes,
    /// regardless of whether it completed normally or panicked. Use this
    /// for cleanup operations like:
    /// - Closing file handles
    /// - Flushing buffers
    /// - Saving state
    /// - Releasing resources
    ///
    /// The default implementation does nothing.
    ///
    /// # Arguments
    ///
    /// * `reason` - Why the task is terminating ([`Normal`](crate::TerminateReason::Normal) or [`Panic`](crate::TerminateReason::Panic))
    ///
    /// # Panics
    ///
    /// If this method panics, the panic is caught and logged but does not
    /// prevent the task from completing. The task will still terminate
    /// successfully.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use notizia::prelude::*;
    /// # #[derive(Clone)] enum Msg { Stop }
    /// # #[derive(Task)]
    /// # #[task(message = Msg)]
    /// struct Worker {
    ///     file: Arc<Mutex<File>>,
    /// }
    ///
    /// impl Runnable<Msg> for Worker {
    ///     async fn start(&self) {
    ///         loop {
    ///             match recv!(self) {
    ///                 Ok(Msg::Stop) => break,
    ///                 Err(_) => break,
    ///             }
    ///         }
    ///     }
    ///     
    ///     async fn terminate(&self, reason: TerminateReason) {
    ///         // Cleanup regardless of why we're stopping
    ///         match reason {
    ///             TerminateReason::Normal => {
    ///                 println!("Shutting down gracefully");
    ///                 self.file.lock().await.flush().await.ok();
    ///             }
    ///             TerminateReason::Panic(msg) => {
    ///                 eprintln!("Task crashed: {}", msg);
    ///                 // Still try to flush
    ///                 self.file.lock().await.flush().await.ok();
    ///             }
    ///         }
    ///     }
    /// }
    fn terminate(&self, reason: TerminateReason) -> impl Future<Output = ()> + Send {
        // Default no-op implementation
        async move {
            let _ = reason;
        }
    }
}

/// Internal trait implemented by the derive macro.
///
/// This trait is automatically implemented by the `#[derive(Task)]` macro
/// and should not be implemented manually. It provides the infrastructure
/// for task spawning, message passing, and lifecycle management.
///
/// The trait combines the user-facing [`Runnable`] trait with internal
/// machinery for channel setup and task-local state management.
pub trait Task<T>: Runnable<T>
where
    T: Send,
{
    /// Internal setup method (do not call directly).
    ///
    /// This method is called by the generated code to set up the receiver
    /// and start the task logic.
    #[doc(hidden)]
    fn __setup(
        &self,
        receiver: UnboundedReceiver<T>,
    ) -> impl Future<Output = TerminateReason> + Send;

    /// Get the mailbox for this task.
    ///
    /// Returns the mailbox associated with this task, which can be used
    /// to receive messages.
    fn mailbox(&self) -> Mailbox<T>;

    /// Run the task, returning a handle.
    ///
    /// This method spawns the task on the Tokio runtime and returns a
    /// [`TaskHandle`] that can be used to send messages, wait for completion,
    /// or kill the task.
    ///
    /// This is equivalent to using the `spawn!()` macro.
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
    /// let handle = worker.run();
    /// handle.join().await;
    /// # }
    /// ```
    fn run(self) -> TaskHandle<T>;

    /// Alias for [`run`](Self::run). Spawns the task and returns a handle.
    ///
    /// This method is provided as an alias to match the naming of the `spawn!()`
    /// macro. It is functionally identical to `run()`.
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
    /// let handle = worker.spawn();  // Same as worker.run()
    /// handle.join().await;
    /// # }
    /// ```
    #[inline]
    fn spawn(self) -> TaskHandle<T>
    where
        Self: Sized,
    {
        self.run()
    }

    /// Receive a message from the task's mailbox.
    ///
    /// This method awaits a message from the task's mailbox. It should be
    /// called from within the task's `start()` method.
    ///
    /// # Errors
    ///
    /// Returns [`RecvError::Closed`](crate::core::errors::RecvError::Closed)
    /// if the channel has been closed.
    ///
    /// Returns [`RecvError::Poisoned`](crate::core::errors::RecvError::Poisoned)
    /// if the mailbox is in an invalid state.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # TODO: Re-enable once derive macro hygiene is fixed
    /// # use notizia::prelude::*;
    /// # #[derive(Clone)]
    /// # enum Signal { Stop }
    /// # #[derive(Task)]
    /// # #[task(message = Signal)]
    /// # struct Worker;
    /// impl Runnable<Signal> for Worker {
    ///     async fn start(&self) {
    ///         // Using the method directly
    ///         let msg = self.recv().await.unwrap();
    ///         
    ///         // Or using the macro (equivalent)
    ///         let msg = recv!(self).unwrap();
    ///     }
    /// }
    /// ```
    fn recv(&self) -> impl Future<Output = RecvResult<T>> + Send {
        async move { self.mailbox().recv().await }
    }

    /// Get a reference to this task.
    ///
    /// Returns a [`TaskRef`] that can be used to send messages to this task.
    /// This is useful for passing lightweight references to other tasks.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # TODO: Re-enable once derive macro hygiene is fixed
    /// # use notizia::prelude::*;
    /// # #[derive(Clone)]
    /// # enum Signal {}
    /// # #[derive(Task)]
    /// # #[task(message = Signal)]
    /// # struct Worker;
    /// # impl Runnable<Signal> for Worker {
    /// #     async fn start(&self) {
    /// #         let other_task_ref = self.this();
    /// #         // Pass other_task_ref to another task
    /// #     }
    /// # }
    /// ```
    fn this(&self) -> TaskRef<T>;
}
