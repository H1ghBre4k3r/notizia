//! Convenience macros for task operations.
//!
//! This module provides ergonomic macros for common task operations:
//! - [`spawn!`] - Spawn a task
//! - [`send!`] - Send a message to a task
//! - [`recv!`] - Receive a message (must be awaited)
//!
//! These macros are provided for convenience and consistency with the
//! actor-like programming model. You can also use the underlying methods
//! directly if you prefer a more explicit style.

/// Spawn a task which implements [`Runnable`](crate::task::Runnable).
///
/// This macro is a convenient wrapper around the [`Task::run()`](crate::task::Task::run)
/// method. It spawns the task on the Tokio runtime and returns a
/// [`TaskHandle`](crate::task::TaskHandle).
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
/// // Equivalent to:
/// // let handle = worker.run();
/// // or:
/// // let handle = worker.spawn();
/// # }
/// ```
#[macro_export]
macro_rules! spawn {
    ($ident:ident) => {
        $ident.run()
    };
}

/// Send a message to a task.
///
/// This macro is a convenient wrapper around the `send()` method on
/// [`TaskHandle`](crate::task::TaskHandle) or [`TaskRef`](crate::task::TaskRef).
///
/// Returns a [`SendResult`](crate::core::errors::SendResult).
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
/// # let worker = Worker;
/// let handle = spawn!(worker);
/// send!(handle, Signal::Ping).expect("send failed");
///
/// // Equivalent to:
/// // handle.send(Signal::Ping).expect("send failed");
/// # }
/// ```
#[macro_export]
macro_rules! send {
    ($task:ident, $msg:expr) => {
        $task.send($msg)
    };
}

/// Receive a message from a task's mailbox.
///
/// This macro must be used with `.await` as it performs an asynchronous operation.
/// It is a convenient wrapper around the [`Task::recv()`](crate::task::Task::recv)
/// method.
///
/// Returns a [`RecvResult`](crate::core::errors::RecvResult).
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
/// impl Runnable<Signal> for Worker {
///     async fn start(&self) {
///         let msg = recv!(self).unwrap();
///         
///         // Equivalent to:
///         // let msg = self.recv().await.unwrap();
///     }
/// }
/// ```
#[macro_export]
macro_rules! recv {
    ($ident:ident) => {
        $ident.recv().await
    };
}
