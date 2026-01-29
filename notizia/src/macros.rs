//! Convenience macros for task operations.
//!
//! This module provides ergonomic macros for common task operations:
//! - [`spawn!`] - Spawn a task
//! - [`send!`] / [`cast!`] - Send a message to a task (fire-and-forget)
//! - [`call!`] - Call a task and wait for response (request-response)
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
/// Call a task and wait for synchronous response with timeout.
///
/// This macro performs a synchronous request-response interaction with a task,
/// blocking until a reply is received or the timeout expires. It automatically
/// creates a oneshot channel for the reply.
///
/// # Timeout
///
/// The timeout parameter is optional and defaults to 5000ms (5 seconds).
/// Specify a custom timeout with `timeout = <millis>`.
///
/// # Errors
///
/// Returns [`CallError::Timeout`] if no response within deadline.
/// Returns [`CallError::ChannelClosed`] if task drops reply channel.
/// Returns [`CallError::SendError`] if task mailbox is closed.
///
/// # Example
///
/// ```no_run
/// # use notizia::prelude::*;
/// # use notizia::{call, message};
/// # use tokio::sync::oneshot;
/// # #[message]
/// # #[derive(Debug)]
/// # enum Msg {
/// #     #[request(reply = u32)]
/// #     GetStatus,
/// # }
/// # #[derive(Task)]
/// # #[task(message = Msg)]
/// # struct Worker;
/// # impl Runnable<Msg> for Worker { async fn start(&self) {} }
/// # #[tokio::main]
/// # async fn main() -> Result<(), CallError> {
/// # let worker = Worker;
/// # let handle = spawn!(worker);
///
/// // Simple syntax for request variants without additional data
/// let status = call!(handle, Msg::GetStatus).await?;
///
/// // With custom timeout (1 second)
/// let status = call!(handle, Msg::GetStatus, timeout = 1000).await?;
///
/// // For variants with additional data, use closure syntax:
/// // call!(handle, |tx| Msg::Echo { id: 42, reply_to: tx }).await?;
/// # Ok(())
/// # }
#[macro_export]
macro_rules! call {
    // Pattern 1: Closure syntax with timeout (implementation)
    // e.g., call!(handle, |tx| Msg::Echo { id: 42, reply_to: tx }, timeout = 1000)
    ($task:expr, |$tx:ident| $msg:expr, timeout = $timeout:expr) => {{
        async {
            let ($tx, rx) = $crate::tokio::sync::oneshot::channel();
            let msg = $msg;
            $task
                .send(msg)
                .map_err(|_| $crate::core::errors::CallError::SendError)?;

            $crate::tokio::time::timeout(std::time::Duration::from_millis($timeout), rx)
                .await
                .map_err(|_| $crate::core::errors::CallError::Timeout)?
                .map_err(|_| $crate::core::errors::CallError::ChannelClosed)
        }
    }};

    // Pattern 2: Closure syntax without timeout
    // e.g., call!(handle, |tx| Msg::Echo { id: 42, reply_to: tx })
    ($task:expr, |$tx:ident| $msg:expr) => {
        call!($task, |$tx| $msg, timeout = 5000)
    };

    // Pattern 3: Simple variant path with timeout (new ergonomic syntax)
    // Match using token trees to detect :: pattern
    // e.g., call!(handle, CounterMsg::GetStatus, timeout = 1000)
    ($task:expr, $first:ident :: $($rest:tt)::+, timeout = $timeout:expr) => {
        call!($task, |__notizia_tx| $first :: $($rest)::+ { reply_to: __notizia_tx }, timeout = $timeout)
    };

    // Pattern 4: Simple variant path without timeout
    // e.g., call!(handle, CounterMsg::GetStatus)
    ($task:expr, $first:ident :: $($rest:tt)::+) => {
        call!($task, $first :: $($rest)::+, timeout = 5000)
    };
}

/// Cast a message to a task (fire-and-forget, asynchronous).
///
/// This is an alias for [`send!`] that matches GenServer/Erlang naming conventions.
/// Cast operations are asynchronous and do not wait for a response.
///
/// # Example
///
/// ```no_run
/// # use notizia::prelude::*;
/// # use notizia::cast;
/// # #[derive(Clone)]
/// # enum Signal { Increment }
/// # #[derive(Task)]
/// # #[task(message = Signal)]
/// # struct Worker;
/// # impl Runnable<Signal> for Worker { async fn start(&self) {} }
/// # #[tokio::main]
/// # async fn main() {
/// # let worker = Worker;
/// let handle = spawn!(worker);
/// cast!(handle, Signal::Increment).expect("cast failed");
/// # }
#[macro_export]
macro_rules! cast {
    ($task:expr, $msg:expr) => {
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
