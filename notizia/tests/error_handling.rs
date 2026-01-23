use notizia::{Runnable, Task, RecvError, SendError, spawn, recv, Mailbox};

#[Task(TestMsg)]
struct TestTask;

#[derive(Debug, Clone, PartialEq)]
enum TestMsg {
    Ping,
    Close,
}

impl Runnable<TestMsg> for TestTask {
    async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let msg = recv!(self);
            match msg {
                TestMsg::Ping => {
                    println!("Received Ping");
                }
                TestMsg::Close => break,
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_closed_channel_recv() {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<TestMsg>();
    let mailbox = Mailbox::new();
    mailbox.set_receiver(receiver).await;
    drop(sender);

    let result = mailbox.recv().await;

    assert!(matches!(result, Err(RecvError::Closed)));
}

#[tokio::test]
async fn test_disconnected_channel_send() {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let task_ref = TestRef::new(sender);
    drop(receiver);

    let result = task_ref.send(TestMsg::Ping);

    assert!(matches!(result, Err(SendError::Disconnected(_))));
}

#[derive(Debug, Clone)]
struct TestRef;
impl TestRef {
    fn new<T>(sender: tokio::sync::mpsc::UnboundedSender<T>) -> notizia::TaskRef<T> {
        notizia::TaskRef::new(sender)
    }
}

#[tokio::test]
async fn test_successful_send() {
    let task = TestTask;
    let handle = spawn!(task);

    let result = handle.send(TestMsg::Close);

    assert!(result.is_ok());

    handle.join().await;
}

#[tokio::test]
async fn test_successful_recv() {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<TestMsg>();
    let mailbox = Mailbox::new();
    mailbox.set_receiver(receiver).await;

    sender.send(TestMsg::Close).unwrap();
    let result = mailbox.recv().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TestMsg::Close);
}
