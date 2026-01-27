use notizia::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::time::{Duration, sleep};

// Message types and tasks for basic lifecycle tests
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum LifecycleMsg {
    Ping,
    Stop,
}

#[derive(Task)]
#[task(message = LifecycleMsg)]
struct BasicTask {
    started: Arc<AtomicBool>,
}

impl Runnable<LifecycleMsg> for BasicTask {
    async fn start(&self) {
        self.started.store(true, Ordering::SeqCst);
        loop {
            match recv!(self) {
                Ok(LifecycleMsg::Ping) => {}
                Ok(LifecycleMsg::Stop) => break,
                Err(_) => break,
            }
        }
    }
}

#[derive(Task)]
#[task(message = LifecycleMsg)]
struct LongRunningTask {
    counter: Arc<AtomicU32>,
}

impl Runnable<LifecycleMsg> for LongRunningTask {
    async fn start(&self) {
        loop {
            self.counter.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(10)).await;

            // Check for messages without blocking
            match self.recv().await {
                Ok(LifecycleMsg::Stop) => break,
                Ok(LifecycleMsg::Ping) => {}
                Err(_) => break,
            }
        }
    }
}

// Message types and tasks for task_with_state_maintains_state
#[derive(Debug, Clone)]
enum CountMsg {
    Increment,
    GetAndStop,
}

#[derive(Task)]
#[task(message = CountMsg)]
struct StatefulTask {
    initial_value: u32,
    result: Arc<AtomicU32>,
}

impl Runnable<CountMsg> for StatefulTask {
    async fn start(&self) {
        let mut count = self.initial_value;
        loop {
            match recv!(self) {
                Ok(CountMsg::Increment) => {
                    count += 1;
                }
                Ok(CountMsg::GetAndStop) => {
                    self.result.store(count, Ordering::SeqCst);
                    break;
                }
                Err(_) => break,
            }
        }
    }
}

// Tests

#[tokio::test]
async fn task_can_be_spawned_and_started() {
    let started = Arc::new(AtomicBool::new(false));
    let task = BasicTask {
        started: started.clone(),
    };

    let handle = spawn!(task);

    // Give task time to start
    sleep(Duration::from_millis(10)).await;

    assert!(started.load(Ordering::SeqCst), "Task should have started");

    handle.send(LifecycleMsg::Stop).unwrap();
    handle.join().await;
}

#[tokio::test]
async fn task_can_be_joined() {
    let started = Arc::new(AtomicBool::new(false));
    let task = BasicTask {
        started: started.clone(),
    };

    let handle = spawn!(task);
    handle.send(LifecycleMsg::Stop).unwrap();

    // Join should wait for task to complete
    handle.join().await;

    assert!(started.load(Ordering::SeqCst));
}

#[tokio::test]
async fn task_can_be_killed() {
    let counter = Arc::new(AtomicU32::new(0));
    let task = LongRunningTask {
        counter: counter.clone(),
    };

    let handle = spawn!(task);

    // Let it run for a bit
    sleep(Duration::from_millis(25)).await;

    // Kill it abruptly
    handle.kill();

    // Give time for abort to take effect
    sleep(Duration::from_millis(10)).await;

    let final_count = counter.load(Ordering::SeqCst);

    // Counter should have stopped incrementing
    sleep(Duration::from_millis(30)).await;
    assert_eq!(
        counter.load(Ordering::SeqCst),
        final_count,
        "Counter should not increment after kill"
    );
}

#[tokio::test]
async fn multiple_tasks_can_run_concurrently() {
    let started1 = Arc::new(AtomicBool::new(false));
    let started2 = Arc::new(AtomicBool::new(false));
    let started3 = Arc::new(AtomicBool::new(false));

    let task1 = BasicTask {
        started: started1.clone(),
    };
    let task2 = BasicTask {
        started: started2.clone(),
    };
    let task3 = BasicTask {
        started: started3.clone(),
    };

    let handle1 = spawn!(task1);
    let handle2 = spawn!(task2);
    let handle3 = spawn!(task3);

    sleep(Duration::from_millis(10)).await;

    assert!(started1.load(Ordering::SeqCst));
    assert!(started2.load(Ordering::SeqCst));
    assert!(started3.load(Ordering::SeqCst));

    handle1.send(LifecycleMsg::Stop).unwrap();
    handle2.send(LifecycleMsg::Stop).unwrap();
    handle3.send(LifecycleMsg::Stop).unwrap();

    handle1.join().await;
    handle2.join().await;
    handle3.join().await;
}

#[tokio::test]
async fn task_with_state_maintains_state() {
    let result = Arc::new(AtomicU32::new(0));
    let task = StatefulTask {
        initial_value: 10,
        result: result.clone(),
    };

    let handle = spawn!(task);

    handle.send(CountMsg::Increment).unwrap();
    handle.send(CountMsg::Increment).unwrap();
    handle.send(CountMsg::Increment).unwrap();
    handle.send(CountMsg::GetAndStop).unwrap();

    handle.join().await;

    assert_eq!(result.load(Ordering::SeqCst), 13);
}

#[tokio::test]
async fn task_handles_channel_closure_gracefully() {
    let started = Arc::new(AtomicBool::new(false));
    let task = BasicTask {
        started: started.clone(),
    };

    let handle = spawn!(task);

    sleep(Duration::from_millis(10)).await;
    assert!(started.load(Ordering::SeqCst));

    // Drop the handle, which closes the channel
    drop(handle);

    // Task should terminate gracefully
    sleep(Duration::from_millis(20)).await;
}
