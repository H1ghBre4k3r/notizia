use std::future::Future;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::error::SendError;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::futures::TaskLocalFuture,
};

pub use notizia_gen::Task;

pub use tokio;

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

    pub async fn recv(&self) -> T {
        // Take the receiver out
        let mut receiver = {
            let mut slot = self.receiver.lock().await;
            slot.take().expect("receiver not set")
        };

        // Await without holding the Mutex lock
        let value = receiver.recv().await.unwrap();

        // Put it back
        *self.receiver.lock().await = Some(receiver);

        value
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

    fn run(self) -> TaskHandle<T, impl Future<Output = ()>>;

    fn recv(&self) -> impl Future<Output = T> + Send {
        async move { self.mailbox().recv().await }
    }
}
pub struct TaskHandle<T, F>
where
    T: 'static,
    F: Future<Output = ()>,
{
    sender: UnboundedSender<T>,
    handle: TaskLocalFuture<Mailbox<T>, F>,
}

impl<T, F> TaskHandle<T, F>
where
    T: 'static,
    F: Future<Output = ()>,
{
    pub fn new(sender: UnboundedSender<T>, handle: TaskLocalFuture<Mailbox<T>, F>) -> Self {
        TaskHandle { sender, handle }
    }

    pub async fn join(self) {
        self.handle.await
    }

    pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
        self.sender.send(msg)
    }
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
