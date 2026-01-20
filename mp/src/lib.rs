use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;

use tokio::sync::mpsc::error::SendError;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::futures::TaskLocalFuture,
};

pub use tokio;

#[derive(Clone)]
pub struct Mailbox<T> {
    pub receiver: Rc<RefCell<Option<UnboundedReceiver<T>>>>,
}

impl<T> Default for Mailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Mailbox<T> {
    pub fn new() -> Self {
        Mailbox {
            receiver: Rc::new(RefCell::new(None)),
        }
    }

    pub fn set_receiver(&self, receiver: UnboundedReceiver<T>) {
        *self.receiver.borrow_mut() = Some(receiver);
    }

    pub async fn recv(&self) -> T {
        // Take the receiver out
        let mut receiver = {
            let mut slot = self.receiver.borrow_mut();
            slot.take().expect("receiver not set")
        };

        // Await without holding the RefCell borrow
        let value = receiver.recv().await.unwrap();

        // Put it back
        *self.receiver.borrow_mut() = Some(receiver);

        value
    }
}

pub trait Runnable<T> {
    async fn start(&self);
}

pub trait Proc<T>: Runnable<T> {
    async fn __setup(&self, receiver: UnboundedReceiver<T>);

    fn mailbox(&self) -> Mailbox<T>;

    fn run(self) -> TaskHandle<T, impl Future<Output = ()>>;
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
