//! Error handling patterns example.
//!
//! This example demonstrates:
//! - Graceful error handling
//! - Channel closure detection
//! - Using Result with the ? operator

use notizia::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone)]
enum Message {
    Process(u32),
    Shutdown,
}

#[derive(Task)]
#[task(message = Message)]
struct ResilientTask {
    error_occurred: Arc<AtomicBool>,
}

impl Runnable<Message> for ResilientTask {
    async fn start(&self) {
        println!("ResilientTask started");

        loop {
            // Using ? operator with RecvResult
            match self.recv().await {
                Ok(Message::Process(value)) => {
                    if value == 0 {
                        println!("Warning: Received zero value (division by zero would occur)");
                        self.error_occurred.store(true, Ordering::SeqCst);
                        continue;
                    }

                    let result = 100 / value;
                    println!("Processed: 100 / {} = {}", value, result);
                }
                Ok(Message::Shutdown) => {
                    println!("Graceful shutdown requested");
                    break;
                }
                Err(e) => {
                    println!("Channel error: {}", e);
                    break;
                }
            }
        }

        println!("ResilientTask finished");
    }
}

#[derive(Task)]
#[task(message = Message)]
struct QuickTask;

impl Runnable<Message> for QuickTask {
    async fn start(&self) {
        println!("QuickTask started and immediately finishing");
        // Terminates immediately
    }
}

#[tokio::main]
async fn main() {
    println!("=== Error Handling Example ===\n");

    // Example 1: Handling errors within task
    println!("Example 1: Error handling within task\n");
    let error_flag = Arc::new(AtomicBool::new(false));
    let task = ResilientTask {
        error_occurred: error_flag.clone(),
    };
    let handle = spawn!(task);

    handle.send(Message::Process(10)).unwrap();
    handle.send(Message::Process(5)).unwrap();
    handle.send(Message::Process(0)).unwrap(); // This will trigger error handling
    handle.send(Message::Process(2)).unwrap();
    handle.send(Message::Shutdown).unwrap();

    let _ = handle.join().await;

    if error_flag.load(Ordering::SeqCst) {
        println!("Task encountered an error but recovered\n");
    }

    // Example 2: Sending to terminated task
    println!("Example 2: Sending to terminated task\n");
    let quick_task = QuickTask;
    let quick_handle = spawn!(quick_task);
    let task_ref = quick_handle.this();

    // Wait for task to finish
    let _ = quick_handle.join().await;

    // Try to send to terminated task
    match task_ref.send(Message::Process(42)) {
        Ok(_) => println!("Message sent successfully"),
        Err(e) => println!("Expected error: {}\n", e),
    }

    // Example 3: Channel closure detection
    println!("Example 3: Channel closure\n");
    let task2 = ResilientTask {
        error_occurred: Arc::new(AtomicBool::new(false)),
    };
    let handle2 = spawn!(task2);

    handle2.send(Message::Process(4)).unwrap();

    // Drop handle to close channel
    drop(handle2);

    // Task should detect channel closure and exit gracefully
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("All examples completed!");
}
