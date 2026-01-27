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
#[derive(Clone)]
pub struct TaskState<T> {
    pub mailbox: Mailbox<T>,
    pub sender: UnboundedSender<T>,
}
