//! Task-local state (internal use only).

use tokio::sync::mpsc::UnboundedSender;

use super::Mailbox;

/// Internal state stored in task-local storage.
///
/// This type is used internally by the generated code to store per-task
/// state including the mailbox and sender. It is stored using Tokio's
/// `task_local!` macro.
///
/// This type is hidden from documentation as it's an implementation detail.
pub struct TaskState<T> {
    pub mailbox: Mailbox<T>,
    pub sender: UnboundedSender<T>,
}

// Manual Clone implementation to avoid requiring T: Clone
// Both Mailbox<T> and UnboundedSender<T> are Clone regardless of T
impl<T> Clone for TaskState<T> {
    fn clone(&self) -> Self {
        TaskState {
            mailbox: self.mailbox.clone(),
            sender: self.sender.clone(),
        }
    }
}
