use notizia::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::time::{Duration, sleep};

// Message types and tasks for task_ref_can_send_messages
#[derive(Debug, Clone)]
struct PingMsg {
    sender: TaskRef<PongMsg>,
    count: u32,
}

#[derive(Debug, Clone)]
struct PongMsg {
    count: u32,
}

#[derive(Task)]
#[task(message = PingMsg)]
struct PingTask {
    max_count: u32,
    final_count: Arc<AtomicU32>,
}

impl Runnable<PingMsg> for PingTask {
    async fn start(&self) {
        while let Ok(msg) = recv!(self) {
            if msg.count >= self.max_count {
                self.final_count.store(msg.count, Ordering::SeqCst);
                break;
            }
            // Send back incremented count
            msg.sender
                .send(PongMsg {
                    count: msg.count + 1,
                })
                .unwrap();
        }
    }
}

#[derive(Task)]
#[task(message = PongMsg)]
struct PongTask {
    ping_ref: Option<TaskRef<PingMsg>>,
}

impl Runnable<PongMsg> for PongTask {
    async fn start(&self) {
        while let Ok(msg) = recv!(self) {
            if let Some(ref ping) = self.ping_ref {
                ping.send(PingMsg {
                    sender: self.this(),
                    count: msg.count + 1,
                })
                .unwrap();
            }
        }
    }
}

// Message types and tasks for task_ref_can_be_cloned
#[derive(Debug, Clone)]
struct SimpleMsg;

#[derive(Task)]
#[task(message = SimpleMsg)]
struct SimpleTask {
    received: Arc<AtomicU32>,
}

impl Runnable<SimpleMsg> for SimpleTask {
    async fn start(&self) {
        while recv!(self).is_ok() {
            self.received.fetch_add(1, Ordering::SeqCst);
        }
    }
}

// Message types and tasks for task_ref_can_be_passed_to_multiple_tasks
#[derive(Debug, Clone)]
enum CollectorMsg {
    Value(u32),
    Stop,
}

#[derive(Task)]
#[task(message = CollectorMsg)]
struct CollectorTask {
    count: Arc<AtomicU32>,
}

impl Runnable<CollectorMsg> for CollectorTask {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(CollectorMsg::Value(v)) => {
                    self.count.fetch_add(v, Ordering::SeqCst);
                }
                Ok(CollectorMsg::Stop) => break,
                Err(_) => break,
            }
        }
    }
}

#[derive(Debug, Clone)]
enum SenderMsg {
    Send,
    Stop,
}

#[derive(Task)]
#[task(message = SenderMsg)]
struct SenderTask {
    id: u32,
    collector: TaskRef<CollectorMsg>,
}

impl Runnable<SenderMsg> for SenderTask {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(SenderMsg::Send) => {
                    self.collector.send(CollectorMsg::Value(self.id)).unwrap();
                }
                Ok(SenderMsg::Stop) => break,
                Err(_) => break,
            }
        }
    }
}

// Message types and tasks for task_ref_this_returns_working_reference
#[derive(Debug, Clone)]
struct SelfMsg;

#[derive(Task)]
#[task(message = SelfMsg)]
struct SelfReferencingTask {
    sent_to_self: Arc<AtomicBool>,
}

impl Runnable<SelfMsg> for SelfReferencingTask {
    async fn start(&self) {
        // Send message to self using this()
        let my_ref = self.this();
        my_ref.send(SelfMsg).unwrap();

        // Receive the message
        if recv!(self).is_ok() {
            self.sent_to_self.store(true, Ordering::SeqCst);
        }
    }
}

// Message types and tasks for task_ref_send_fails_when_task_terminated
#[derive(Debug, Clone)]
struct QuickMsg;

#[derive(Task)]
#[task(message = QuickMsg)]
struct QuickTask;

impl Runnable<QuickMsg> for QuickTask {
    async fn start(&self) {
        // Terminate immediately
    }
}

// Tests

#[tokio::test]
async fn task_ref_can_send_messages() {
    let final_count = Arc::new(AtomicU32::new(0));
    let ping = PingTask {
        max_count: 5,
        final_count: final_count.clone(),
    };
    let ping_handle = spawn!(ping);

    let pong = PongTask {
        ping_ref: Some(ping_handle.this()),
    };
    let pong_handle = spawn!(pong);

    // Start the ping-pong
    ping_handle
        .send(PingMsg {
            sender: pong_handle.this(),
            count: 0,
        })
        .unwrap();

    // Wait for completion
    ping_handle.join().await;

    // Ping-pong goes: 0->1->2->3->4->5->6, and ping stops at 6 (>= 5)
    assert_eq!(final_count.load(Ordering::SeqCst), 6);

    pong_handle.kill();
}

#[tokio::test]
async fn task_ref_can_be_cloned() {
    let received = Arc::new(AtomicU32::new(0));
    let task = SimpleTask {
        received: received.clone(),
    };
    let handle = spawn!(task);

    // Clone the TaskRef
    let ref1 = handle.this();
    let ref2 = ref1.clone();
    let ref3 = ref2.clone();

    // All refs should work
    ref1.send(SimpleMsg).unwrap();
    ref2.send(SimpleMsg).unwrap();
    ref3.send(SimpleMsg).unwrap();

    sleep(Duration::from_millis(10)).await;

    assert_eq!(received.load(Ordering::SeqCst), 3);

    handle.kill();
}

#[tokio::test]
async fn task_ref_can_be_passed_to_multiple_tasks() {
    let count = Arc::new(AtomicU32::new(0));
    let collector = CollectorTask {
        count: count.clone(),
    };
    let collector_handle = spawn!(collector);
    let collector_ref = collector_handle.this();

    // Create multiple senders with the same TaskRef
    let sender1 = SenderTask {
        id: 1,
        collector: collector_ref.clone(),
    };
    let sender2 = SenderTask {
        id: 2,
        collector: collector_ref.clone(),
    };
    let sender3 = SenderTask {
        id: 3,
        collector: collector_ref.clone(),
    };

    let h1 = spawn!(sender1);
    let h2 = spawn!(sender2);
    let h3 = spawn!(sender3);

    // Each sender sends once
    h1.send(SenderMsg::Send).unwrap();
    h2.send(SenderMsg::Send).unwrap();
    h3.send(SenderMsg::Send).unwrap();

    sleep(Duration::from_millis(10)).await;

    assert_eq!(count.load(Ordering::SeqCst), 6); // 1 + 2 + 3

    h1.send(SenderMsg::Stop).unwrap();
    h2.send(SenderMsg::Stop).unwrap();
    h3.send(SenderMsg::Stop).unwrap();
    collector_handle.send(CollectorMsg::Stop).unwrap();

    h1.join().await;
    h2.join().await;
    h3.join().await;
    collector_handle.join().await;
}

#[tokio::test]
async fn task_ref_this_returns_working_reference() {
    let sent = Arc::new(AtomicBool::new(false));
    let task = SelfReferencingTask {
        sent_to_self: sent.clone(),
    };
    let handle = spawn!(task);

    handle.join().await;

    assert!(
        sent.load(Ordering::SeqCst),
        "Task should have sent message to itself"
    );
}

#[tokio::test]
async fn task_ref_send_fails_when_task_terminated() {
    let task = QuickTask;
    let handle = spawn!(task);
    let task_ref = handle.this();

    // Wait for task to complete
    handle.join().await;

    // Sending should fail
    assert!(task_ref.send(QuickMsg).is_err());
}
