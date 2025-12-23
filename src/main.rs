use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Clone)]
struct Mailbox<T>(Sender<T>);

#[derive(Clone)]
struct Task<M> {
    mailbox: Mailbox<M>,
}

impl<T> Task<T>
where
    T: Clone,
{
    pub fn send(&self, payload: T) {
        self.mailbox.0.send(payload).unwrap()
    }
}

struct SpawnedTask<T> {
    receiver: Receiver<T>,
}

macro_rules! proc {
    ($($content:tt)*) => {
        spawn_task(|receiver| {
            let recv = || receiver.recv();
            $($content)*
        })
    };
}

fn spawn_task<T, Func>(func: Func) -> Task<T>
where
    T: Send + 'static,
    Func: Fn(Receiver<T>) -> () + Send + 'static,
{
    let (sender, receiver) = channel::<T>();
    let mb = Mailbox(sender);

    let task = Task { mailbox: mb };

    std::thread::spawn(move || func(receiver));

    task
}

fn main() {
    let task: Task<u32> = proc! {
        recv();
    };
}
