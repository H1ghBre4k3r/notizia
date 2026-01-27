//! Core types for message passing.
//!
//! This module contains the fundamental types used for message passing:
//! - [`Mailbox`] - Thread-safe message receiver
//! - [`errors`] - Error types for send and receive operations
//! - [`state`] - Internal task-local state (hidden from docs)

pub mod errors;
pub mod lifecycle;
pub mod mailbox;
pub(crate) mod state;

pub use mailbox::Mailbox;
pub use state::TaskState;
