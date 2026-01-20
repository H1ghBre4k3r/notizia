use mp::{Proc, Runnable, send, spawn};

#[mp_gen::Proc(Bar)]
struct MyProc {}

#[derive(Debug, Clone)]
enum Bar {
    Yes,
    No,
}

impl Runnable<Bar> for MyProc {
    async fn start(&self) -> () {
        async {
            let val = self.recv().await;
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
