pub mod core;

use std::future::Future;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

pub use notizia_gen::Task;

pub use tokio;

use crate::core::errors::{RecvError, RecvResult, SendResult};

#[derive(Clone)]
pub struct Mailbox<T> {
    pub receiver: Arc<Mutex<Option<UnboundedReceiver<T>>>>,
}

impl<T> Default for Mailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Mailbox<T> {
    pub fn new() -> Self {
        Mailbox {
            receiver: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_receiver(&self, receiver: UnboundedReceiver<T>) {
        *self.receiver.lock().await = Some(receiver);
    }

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

pub trait Runnable<T>: Send + Sync {
    fn start(&self) -> impl Future<Output = ()> + Send;
}

pub trait Task<T>: Runnable<T>
where
    T: Send,
{
    fn __setup(&self, receiver: UnboundedReceiver<T>) -> impl Future<Output = ()> + Send;

    fn mailbox(&self) -> Mailbox<T>;

    fn run(self) -> TaskHandle<T>;

    fn recv(&self) -> impl Future<Output = RecvResult<T>> + Send {
        async move { self.mailbox().recv().await }
    }

    fn this(&self) -> TaskRef<T>;
}
pub struct TaskHandle<T>
where
    T: 'static,
{
    sender: UnboundedSender<T>,
    handle: JoinHandle<()>,
}

impl<T> TaskHandle<T>
where
    T: 'static,
{
    pub fn new(sender: UnboundedSender<T>, handle: JoinHandle<()>) -> Self {
        TaskHandle { sender, handle }
    }

    pub async fn join(self) {
        let _ = self.handle.await;
    }

    pub fn send(&self, msg: T) -> SendResult<T> {
        self.sender.send(msg)
    }

    pub fn kill(self) {
        self.handle.abort();
    }
}

#[derive(Debug, Clone)]
pub struct TaskRef<T> {
    sender: UnboundedSender<T>,
}

impl<T> TaskRef<T> {
    pub fn new(sender: UnboundedSender<T>) -> Self {
        TaskRef { sender }
    }

    pub fn send(&self, msg: T) -> SendResult<T> {
        self.sender.send(msg)
    }
}

#[derive(Clone)]
pub struct TaskState<T> {
    pub mailbox: Mailbox<T>,
    pub sender: UnboundedSender<T>,
}

/// Spawn a task which implements `notizia::Runnable`.
#[macro_export]
macro_rules! spawn {
    ($ident:ident) => {
        $ident.run()
    };
}

/// Send a message to a task which what spawned by `notizia::spawn!()`.
#[macro_export]
macro_rules! send {
    ($task:ident, $msg:expr) => {
        $task.send($msg)
    };
}

#[macro_export]
macro_rules! recv {
    ($ident:ident) => {
        $ident.recv().await
    };
}
