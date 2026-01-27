//! Lightweight reference to a task.

use tokio::sync::mpsc::UnboundedSender;

use crate::core::errors::SendResult;

/// A lightweight reference to a task for sending messages.
///
/// `TaskRef` is a cloneable reference that can be used to send messages to a task
/// without owning the full [`TaskHandle`](super::TaskHandle). This is useful for:
/// - Passing task references to other tasks
/// - Storing multiple references to the same task
/// - Enabling inter-task communication patterns
///
/// Unlike `TaskHandle`, a `TaskRef` cannot join or kill the task.
///
/// # Example
///
/// ```ignore
/// # TODO: Re-enable once derive macro hygiene is fixed
/// # use notizia::prelude::*;
/// # #[derive(Task)]
/// # #[task(message = PingMsg)]
/// # struct PingTask;
/// # impl Runnable<PingMsg> for PingTask {
/// #     async fn start(&self) {
/// #         // Get a reference to this task
/// #         let my_ref = self.this();
/// #         // Pass my_ref to another task so it can send messages back
/// #     }
/// # }
/// # #[derive(Clone)]
/// # struct PingMsg;
/// ```
#[derive(Debug, Clone)]
pub struct TaskRef<T> {
    sender: UnboundedSender<T>,
}

impl<T> TaskRef<T> {
    /// Create a new task reference.
    ///
    /// This is typically called by the generated code and not by user code directly.
    #[doc(hidden)]
    pub fn new(sender: UnboundedSender<T>) -> Self {
        TaskRef { sender }
    }

    /// Send a message to the referenced task.
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
    /// ```ignore
    /// # TODO: Re-enable once derive macro hygiene is fixed
    /// # use notizia::prelude::*;
    /// # #[derive(Clone)]
    /// # enum Signal { Ping }
    /// # #[derive(Task)]
    /// # #[task(message = Signal)]
    /// # struct Worker;
    /// # impl Runnable<Signal> for Worker {
    /// #     async fn start(&self) {
    /// #         let task_ref = self.this();
    /// #         // Send a message using the reference
    /// #         task_ref.send(Signal::Ping).expect("send failed");
    /// #     }
    /// # }
    /// ```
    pub fn send(&self, msg: T) -> SendResult<T> {
        self.sender.send(msg)
    }
}
