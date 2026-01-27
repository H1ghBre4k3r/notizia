//! # Notizia - Frictionless Message Passing
//!
//! **Async Rust actor-like message passing for the Tokio runtime.**
//!
//! Async Rust is powerful, but managing channels, task handles, and state synchronization
//! often leads to verbose boilerplate. Notizia cuts through the noise. It provides a thin,
//! type-safe layer over Tokio's primitives, offering an actor-like model that feels native
//! to Rust.
//!
//! The philosophy is simple: **Concurrency shouldn't hurt.**
//!
//! ## Why Notizia?
//!
//! We built Notizia to solve the "setup tax" of spawning async tasks. Instead of manually
//! wiring `mpsc` channels and managing mutex locks, you define your state and your messages.
//! Notizia generates the rest.
//!
//! - **Zero Boilerplate:** The `#[derive(Task)]` macro writes the plumbing for you.
//! - **Type-Safe Mailboxes:** Messages are strictly typed. No dynamic dispatch.
//! - **Tokio Native:** Built directly on standard `mpsc` channels and `JoinHandle`s.
//! - **Unified Semantics:** Consistent naming and ergonomic APIs.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use notizia::prelude::*;
//!
//! // 1. Define your message protocol
//! #[derive(Debug, Clone)]
//! enum Signal {
//!     Ping,
//!     Pong,
//! }
//!
//! // 2. Define your state and derive Task
//! #[derive(Task)]
//! #[task(message = Signal)]
//! struct Worker {
//!     id: usize,
//! }
//!
//! // 3. Implement the logic
//! impl Runnable<Signal> for Worker {
//!     async fn start(&self) {
//!         loop {
//!             match recv!(self) {
//!                 Ok(msg) => println!("Worker {} received: {:?}", self.id, msg),
//!                 Err(_) => break,
//!             }
//!         }
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     // 4. Spawn and enjoy
//!     let worker = Worker { id: 1 };
//!     let handle = spawn!(worker);
//!
//!     send!(handle, Signal::Ping).expect("failed to send");
//!     
//!     handle.join().await;
//! }
//! ```
//!
//! ## Core Concepts
//!
//! ### Tasks
//!
//! A **task** is an independent unit of work that processes messages. Tasks are defined
//! by deriving the [`Task`] trait and implementing [`Runnable`]:
//!
//! ```rust,ignore
//! # TODO: Re-enable once derive macro hygiene is fixed
//! # use notizia::prelude::*;
//! # #[derive(Clone)] enum MyMessage {}
//! #[derive(Task)]
//! #[task(message = MyMessage)]
//! struct MyTask {
//!     // Your state here
//! }
//!
//! impl Runnable<MyMessage> for MyTask {
//!     async fn start(&self) {
//!         // Your task logic here
//!     }
//! }
//! ```
//!
//! ### Messages
//!
//! Messages are strongly-typed values sent between tasks. They must implement `Clone`
//! since messages are passed through unbounded channels:
//!
//! ```rust
//! #[derive(Debug, Clone)]
//! enum MyMessage {
//!     DoWork(String),
//!     Shutdown,
//! }
//! ```
//!
//! ### Handles and References
//!
//! - [`TaskHandle<T>`]: Full control over a task (send, join, kill)
//! - [`TaskRef<T>`]: Lightweight reference for sending messages only
//!
//! ## API Styles
//!
//! Notizia supports both macro and method-based APIs:
//!
//! ### Macro Style (Recommended)
//!
//! ```rust,no_run
//! # use notizia::prelude::*;
//! # #[derive(Clone)] enum Signal { Ping }
//! # #[derive(Task)]
//! # #[task(message = Signal)]
//! # struct Worker;
//! # impl Runnable<Signal> for Worker { async fn start(&self) {} }
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let worker = Worker;
//! let handle = spawn!(worker);
//! send!(handle, Signal::Ping)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Method Style (Alternative)
//!
//! ```rust,no_run
//! # use notizia::prelude::*;
//! # #[derive(Clone)] enum Signal { Ping }
//! # #[derive(Task)]
//! # #[task(message = Signal)]
//! # struct Worker;
//! # impl Runnable<Signal> for Worker { async fn start(&self) {} }
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let worker = Worker;
//! let handle = worker.run();  // or worker.spawn()
//! handle.send(Signal::Ping)?;
//! # Ok(())
//! # }
//! ```
//!
//! Both styles are equivalent—choose what feels most comfortable.
//!
//! ## Error Handling
//!
//! Notizia provides explicit, type-safe error handling for all messaging operations:
//!
//! - [`recv()`](Task::recv) returns [`RecvResult<T>`](core::errors::RecvResult)
//! - [`send()`](TaskHandle::send) returns [`SendResult<T>`](core::errors::SendResult)
//!
//! ### Pattern 1: Unwrap for Prototypes
//!
//! ```rust,ignore
//! # TODO: Re-enable once derive macro hygiene is fixed
//! # use notizia::prelude::*;
//! # #[derive(Clone)] enum Signal {}
//! # #[derive(Task)]
//! # #[task(message = Signal)]
//! # struct Worker;
//! impl Runnable<Signal> for Worker {
//!     async fn start(&self) {
//!         loop {
//!             let msg = recv!(self).unwrap();  // Panics on error
//!             // Process message...
//!         }
//!     }
//! }
//! ```
//!
//! ### Pattern 2: Error Propagation with `?`
//!
//! ```rust,ignore
//! # TODO: Re-enable once derive macro hygiene is fixed
//! # use notizia::prelude::*;
//! # use notizia::core::errors::RecvError;
//! # #[derive(Clone)] enum Signal {}
//! # #[derive(Task)]
//! # #[task(message = Signal)]
//! # struct Worker;
//! # impl Runnable<Signal> for Worker {
//! #     async fn start(&self) {
//! #         let _ = self.process().await;
//! #     }
//! # }
//! # impl Worker {
//! async fn process(&self) -> Result<(), RecvError> {
//!     loop {
//!         let msg = recv!(self)?;  // Propagates errors
//!         // Process message...
//!     }
//! }
//! # }
//! ```
//!
//! ### Pattern 3: Explicit Handling
//!
//! ```rust,ignore
//! # TODO: Re-enable once derive macro hygiene is fixed
//! # use notizia::prelude::*;
//! # #[derive(Clone)] enum Signal {}
//! # #[derive(Task)]
//! # #[task(message = Signal)]
//! # struct Worker;
//! impl Runnable<Signal> for Worker {
//!     async fn start(&self) {
//!         loop {
//!             match recv!(self) {
//!                 Ok(msg) => { /* Handle message */ }
//!                 Err(RecvError::Closed) => {
//!                     println!("Channel closed, shutting down gracefully");
//!                     break;
//!                 }
//!                 Err(e) => {
//!                     eprintln!("Error: {}", e);
//!                     break;
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`core`] - Core types (mailbox, errors, internal state)
//! - [`task`] - Task traits and handles
//! - [`prelude`] - Common imports for convenience
//!
//! ## Re-exports
//!
//! Notizia re-exports key types at the crate root for convenience:

pub mod core;
#[doc(hidden)]
pub mod macros;
pub mod prelude;
pub mod task;

// Re-export core types at crate root
pub use crate::core::Mailbox;
pub use crate::core::errors::{RecvError, RecvResult, SendResult};

// Re-export task types at crate root
pub use crate::task::{Runnable, Task, TaskHandle, TaskRef};

// Re-export lifecycle types at crate root
pub use crate::core::lifecycle::{ShutdownError, ShutdownResult, TerminateReason};

// Note: Macros (spawn!, send!, recv!) are already at crate root via #[macro_export]
// They don't need to be re-exported here

// Re-export procedural macro
// Note: We keep the original name 'Task' for the attribute macro
// until we migrate to derive macro syntax
#[doc(inline)]
pub use notizia_gen::Task;

// Re-export Tokio for macro usage (hidden from docs)
#[doc(hidden)]
pub use tokio;

#[doc(hidden)]
pub use futures;

// Internal types (hidden from docs)
#[doc(hidden)]
pub use crate::core::state::TaskState;
