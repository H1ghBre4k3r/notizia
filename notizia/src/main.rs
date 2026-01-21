use notizia::{Runnable, Task, recv, send, spawn};

#[Task(Bar)]
struct MyProc {}

#[derive(Debug, Clone)]
enum Bar {
    Yes,
    No,
}

impl Runnable<Bar> for MyProc {
    async fn start(&self) {
        async move {
            let val = recv!(self);
            println!("received {val:?}")
        }
        .await
    }
}

#[tokio::main]
async fn main() {
    let task = MyProc {};

    let handle = spawn!(task);

    if let Err(e) = send!(handle, Bar::Yes) {
        eprintln!("sending failed: {e}")
    }

    handle.join().await
}
