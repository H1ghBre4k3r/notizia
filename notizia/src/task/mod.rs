//! Task types and traits.
//!
//! This module contains the core abstractions for working with tasks:
//! - [`Task`] - Trait automatically implemented by `#[derive(Task)]`
//! - [`Runnable`] - User-facing trait for task logic
//! - [`TaskHandle`] - Handle for controlling spawned tasks
//! - [`TaskRef`] - Lightweight reference for sending messages

pub mod handle;
pub mod reference;
pub mod traits;

pub use handle::TaskHandle;
pub use reference::TaskRef;
pub use traits::{Runnable, Task};
