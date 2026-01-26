use notizia::{
    Runnable, Task,
    core::errors::{RecvError, SendError},
    spawn,
};

#[Task(TestMsg)]
struct TestTask;

#[derive(Debug, Clone)]
enum TestMsg {
    Ping,
    Stop,
}

impl Runnable<TestMsg> for TestTask {
    async fn start(&self) {
        while let Ok(msg) = self.recv().await {
            match msg {
                TestMsg::Ping => {}
                TestMsg::Stop => break,
            }
        }
    }
}

#[tokio::test]
async fn recv_returns_closed_error_when_channel_closed() {
    let mailbox = notizia::Mailbox::<TestMsg>::new();

    // Create a channel directly
    let (sender, receiver) = notizia::tokio::sync::mpsc::unbounded_channel();

    // Set the receiver
    mailbox.set_receiver(receiver).await;

    // Drop the sender to close the channel
    drop(sender);

    // This should return RecvError::Closed when we try to recv
    let result = mailbox.recv().await;
    assert!(matches!(result, Err(RecvError::Closed)));
}

#[tokio::test]
async fn send_returns_disconnected_when_receiver_dropped() {
    // Create a channel directly
    let (sender, receiver) = notizia::tokio::sync::mpsc::unbounded_channel();

    // Drop the receiver immediately
    drop(receiver);

    // Try to send a message - should fail
    let result = sender.send(TestMsg::Ping);

    assert!(matches!(result, Err(SendError(_))));
}

#[tokio::test]
async fn error_types_can_be_propagated_with_question_mark() {
    async fn try_send(handle: &notizia::TaskHandle<TestMsg>) -> Result<(), SendError<TestMsg>> {
        handle.send(TestMsg::Ping)?;
        Ok(())
    }

    let task = TestTask;
    let handle = spawn!(task);

    // Test that we can propagate errors with ?
    let result = try_send(&handle).await;
    assert!(result.is_ok());

    // Kill the task
    handle.kill();
}

#[tokio::test]
async fn recv_returns_poisoned_error_when_receiver_not_set() {
    let mailbox = notizia::Mailbox::<TestMsg>::new();

    // Try to receive without setting the receiver
    let result = mailbox.recv().await;
    assert!(matches!(result, Err(RecvError::Poisoned)));
}

#[tokio::test]
async fn mailbox_can_reuse_receiver_after_close() {
    let task = TestTask;
    let handle = spawn!(task);

    // Send a message successfully
    assert!(handle.send(TestMsg::Stop).is_ok());

    // Let the task process the message
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // After the task completes, sends should fail
    let result = handle.send(TestMsg::Ping);
    assert!(matches!(result, Err(SendError(_))));
}

#[tokio::test]
async fn send_error_returns_original_message() {
    // Create a channel directly
    let (sender, receiver) = notizia::tokio::sync::mpsc::unbounded_channel();

    // Drop the receiver immediately
    drop(receiver);

    // Try to send a message
    let original_msg = TestMsg::Ping;
    let result = sender.send(original_msg.clone());

    // Verify the error contains the original message
    assert!(matches!(result, Err(SendError(_))));

    // We can extract the message from the error
    if let Err(SendError(msg)) = result {
        match msg {
            TestMsg::Ping => {}
            TestMsg::Stop => panic!("Wrong message returned"),
        }
    }
}
