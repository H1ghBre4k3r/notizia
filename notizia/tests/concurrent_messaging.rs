use notizia::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

// Message types and tasks for multiple_senders_to_one_receiver
#[derive(Debug, Clone)]
enum ConcurrentMsg {
    Value(u32),
    Stop,
}

#[derive(Task)]
#[task(message = ConcurrentMsg)]
struct ConcurrentReceiver {
    received: Arc<Mutex<Vec<u32>>>,
}

impl Runnable<ConcurrentMsg> for ConcurrentReceiver {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(ConcurrentMsg::Value(v)) => {
                    self.received.lock().await.push(v);
                }
                Ok(ConcurrentMsg::Stop) => break,
                Err(_) => break,
            }
        }
    }
}

// Message types and tasks for high_volume_messaging
#[derive(Debug, Clone)]
struct CountMsg;

#[derive(Task)]
#[task(message = CountMsg)]
struct CounterTask {
    count: Arc<AtomicU32>,
}

impl Runnable<CountMsg> for CounterTask {
    async fn start(&self) {
        while recv!(self).is_ok() {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }
}

// Message types and tasks for concurrent_receivers_independent
#[derive(Debug, Clone)]
struct UniqueMsg;

#[derive(Task)]
#[task(message = UniqueMsg)]
#[allow(dead_code)]
struct UniqueReceiver {
    id: u32,
    count: Arc<AtomicU32>,
}

impl Runnable<UniqueMsg> for UniqueReceiver {
    async fn start(&self) {
        while recv!(self).is_ok() {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }
}

// Message types and tasks for burst_messaging_no_loss
#[derive(Debug, Clone)]
struct BurstMsg;

#[derive(Task)]
#[task(message = BurstMsg)]
struct BurstTask {
    count: Arc<AtomicU32>,
}

impl Runnable<BurstMsg> for BurstTask {
    async fn start(&self) {
        while recv!(self).is_ok() {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }
}

// Message types and tasks for concurrent_send_and_receive
#[derive(Debug, Clone)]
struct ResponseMsg;

#[derive(Debug, Clone)]
enum EchoMsg {
    Ping(TaskRef<ResponseMsg>),
    Stop,
}

#[derive(Task)]
#[task(message = EchoMsg)]
struct EchoTask;

impl Runnable<EchoMsg> for EchoTask {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(EchoMsg::Ping(sender)) => {
                    sender.send(ResponseMsg).unwrap();
                }
                Ok(EchoMsg::Stop) => break,
                Err(_) => break,
            }
        }
    }
}

#[derive(Task)]
#[task(message = ResponseMsg)]
struct ResponseTask {
    received: Arc<AtomicU32>,
}

impl Runnable<ResponseMsg> for ResponseTask {
    async fn start(&self) {
        while recv!(self).is_ok() {
            self.received.fetch_add(1, Ordering::SeqCst);
        }
    }
}

// Tests

#[tokio::test]
async fn multiple_senders_to_one_receiver() {
    let received = Arc::new(Mutex::new(Vec::new()));
    let task = ConcurrentReceiver {
        received: received.clone(),
    };
    let handle = spawn!(task);

    // Get task references for concurrent sending
    let r1 = handle.this();
    let r2 = handle.this();
    let r3 = handle.this();

    let t1 = tokio::spawn(async move {
        for i in 0..10 {
            r1.send(ConcurrentMsg::Value(i)).unwrap();
        }
    });

    let t2 = tokio::spawn(async move {
        for i in 10..20 {
            r2.send(ConcurrentMsg::Value(i)).unwrap();
        }
    });

    let t3 = tokio::spawn(async move {
        for i in 20..30 {
            r3.send(ConcurrentMsg::Value(i)).unwrap();
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    t3.await.unwrap();

    handle.send(ConcurrentMsg::Stop).unwrap();
    handle.join().await;

    let values = received.lock().await;
    assert_eq!(values.len(), 30);

    // All values should be present (order may vary)
    let value_set: HashSet<_> = values.iter().copied().collect();
    for i in 0..30 {
        assert!(value_set.contains(&i), "Missing value: {}", i);
    }
}

#[tokio::test]
async fn high_volume_messaging() {
    let received = Arc::new(AtomicU32::new(0));

    let task = CounterTask {
        count: received.clone(),
    };
    let handle = spawn!(task);

    // Send 1000 messages as fast as possible
    for _ in 0..1000 {
        handle.send(CountMsg).unwrap();
    }

    // Give task time to process all messages
    sleep(Duration::from_millis(100)).await;

    assert_eq!(received.load(Ordering::SeqCst), 1000);

    drop(handle);
}

#[tokio::test]
async fn message_ordering_from_single_sender() {
    let received = Arc::new(Mutex::new(Vec::new()));
    let task = ConcurrentReceiver {
        received: received.clone(),
    };
    let handle = spawn!(task);

    // Send messages in order
    for i in 0..100 {
        handle.send(ConcurrentMsg::Value(i)).unwrap();
    }

    handle.send(ConcurrentMsg::Stop).unwrap();
    handle.join().await;

    let values = received.lock().await;

    // Messages from a single sender should arrive in order
    for (i, &value) in values.iter().enumerate() {
        assert_eq!(value, i as u32, "Message order violated at index {}", i);
    }
}

#[tokio::test]
async fn concurrent_receivers_independent() {
    let count1 = Arc::new(AtomicU32::new(0));
    let count2 = Arc::new(AtomicU32::new(0));
    let count3 = Arc::new(AtomicU32::new(0));

    let task1 = UniqueReceiver {
        id: 1,
        count: count1.clone(),
    };
    let task2 = UniqueReceiver {
        id: 2,
        count: count2.clone(),
    };
    let task3 = UniqueReceiver {
        id: 3,
        count: count3.clone(),
    };

    let h1 = spawn!(task1);
    let h2 = spawn!(task2);
    let h3 = spawn!(task3);

    // Send different amounts to each
    for _ in 0..10 {
        h1.send(UniqueMsg).unwrap();
    }
    for _ in 0..20 {
        h2.send(UniqueMsg).unwrap();
    }
    for _ in 0..30 {
        h3.send(UniqueMsg).unwrap();
    }

    sleep(Duration::from_millis(50)).await;

    assert_eq!(count1.load(Ordering::SeqCst), 10);
    assert_eq!(count2.load(Ordering::SeqCst), 20);
    assert_eq!(count3.load(Ordering::SeqCst), 30);

    drop(h1);
    drop(h2);
    drop(h3);
}

#[tokio::test]
async fn burst_messaging_no_loss() {
    let received = Arc::new(AtomicU32::new(0));

    let task = BurstTask {
        count: received.clone(),
    };
    let handle = spawn!(task);

    // Send in rapid bursts
    for _ in 0..10 {
        for _ in 0..50 {
            handle.send(BurstMsg).unwrap();
        }
        sleep(Duration::from_millis(5)).await;
    }

    // Wait for all messages to be processed
    sleep(Duration::from_millis(100)).await;

    assert_eq!(received.load(Ordering::SeqCst), 500);

    drop(handle);
}

#[tokio::test]
async fn concurrent_send_and_receive() {
    let responses = Arc::new(AtomicU32::new(0));
    let echo_task = EchoTask;
    let echo_handle = spawn!(echo_task);

    let response_task = ResponseTask {
        received: responses.clone(),
    };
    let response_handle = spawn!(response_task);
    let response_ref = response_handle.this();

    // Send 100 pings concurrently
    for _ in 0..100 {
        echo_handle
            .send(EchoMsg::Ping(response_ref.clone()))
            .unwrap();
    }

    sleep(Duration::from_millis(100)).await;

    assert_eq!(responses.load(Ordering::SeqCst), 100);

    echo_handle.send(EchoMsg::Stop).unwrap();
    echo_handle.join().await;

    drop(response_handle);
}
