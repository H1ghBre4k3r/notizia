//! Mailbox for receiving messages.

use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

use super::errors::{RecvError, RecvResult};

/// A thread-safe mailbox for receiving messages.
///
/// The mailbox provides a safe way to receive messages from other tasks.
/// It wraps an `UnboundedReceiver` and manages its lifecycle using Arc and Mutex
/// to enable the take-recv-put pattern required for async receiving without
/// holding locks.
#[derive(Clone)]
pub struct Mailbox<T> {
    pub(crate) receiver: Arc<Mutex<Option<UnboundedReceiver<T>>>>,
}

impl<T> Default for Mailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Mailbox<T> {
    /// Create a new empty mailbox.
    ///
    /// The receiver must be set using [`set_receiver`](Self::set_receiver) before
    /// messages can be received.
    pub fn new() -> Self {
        Mailbox {
            receiver: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the receiver for this mailbox.
    ///
    /// This is typically called during task setup by the generated code.
    pub async fn set_receiver(&self, receiver: UnboundedReceiver<T>) {
        *self.receiver.lock().await = Some(receiver);
    }

    /// Receive a message from the mailbox.
    ///
    /// This method will await until a message is available. It uses a take-recv-put
    /// pattern to avoid holding the Mutex lock while awaiting.
    ///
    /// # Errors
    ///
    /// Returns [`RecvError::Closed`] if the channel has been closed.
    /// Returns [`RecvError::Poisoned`] if the receiver has not been set or was
    /// taken and not returned.
    pub async fn recv(&self) -> RecvResult<T> {
        // Take the receiver out
        let mut receiver = {
            let mut slot = self.receiver.lock().await;
            slot.take().ok_or(RecvError::Poisoned)?
        };

        // Await without holding the Mutex lock
        let value = receiver.recv().await.ok_or(RecvError::Closed)?;

        // Put it back
        *self.receiver.lock().await = Some(receiver);

        Ok(value)
    }
}
