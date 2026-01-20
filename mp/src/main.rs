use mp::{Proc, Runnable};

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
            let val = self.mailbox().recv().await;
            println!("received {val:?}")
        }
        .await
    }
}

impl Proc<Bar> for MyProc {
    async fn __setup(&self, receiver: mp::tokio::sync::mpsc::UnboundedReceiver<Bar>) {
        let mb = self.mailbox();

        mb.set_receiver(receiver);

        self.start().await
    }

    fn mailbox(&self) -> mp::Mailbox<Bar> {
        __MyProc_gen::MyProcMailbox.get()
    }

    fn run(self) -> mp::TaskHandle<Bar, impl Future<Output = ()>> {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<Bar>();

        let handle = __MyProc_gen::MyProcMailbox.scope(mp::Mailbox::new(), async move {
            let handle = self.__setup(receiver);
            handle.await
        });

        mp::TaskHandle::new(sender, handle)
    }
}

#[tokio::main]
async fn main() {
    let task = MyProc {};

    let handle = task.run();

    handle.send(Bar::Yes);

    handle.join().await
}
