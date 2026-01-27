//! Common imports for Notizia.
//!
//! The prelude provides convenient access to the most commonly used types and macros.
//! It's designed to be imported with a glob import for quick setup:
//!
//! ```
//! use notizia::prelude::*;
//! ```
//!
//! This brings into scope:
//! - Core types: [`Mailbox`], error types ([`RecvError`], [`RecvResult`], [`SendResult`])
//! - Task types: [`Task`], [`Runnable`], [`TaskHandle`], [`TaskRef`]
//! - Macros: [`spawn!`], [`send!`], [`recv!`]
//! - Derive macro: [`Task`] (for `#[derive(Task)]`)

pub use crate::core::Mailbox;
pub use crate::core::errors::{CallError, CallResult, RecvError, RecvResult, SendResult};
pub use crate::core::lifecycle::{ShutdownError, ShutdownResult, TerminateReason};
pub use crate::task::{Runnable, Task, TaskHandle, TaskRef};

// Macros are already exported at crate root via #[macro_export]
// They're automatically available when you use notizia::prelude::*
pub use crate::{recv, send, spawn};

// Re-export the attribute macro (will become derive macro in next phase)
#[doc(inline)]
pub use notizia_gen::Task;
