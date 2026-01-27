//! Call/Cast request-response patterns example.
//!
//! This example demonstrates:
//! - Synchronous request-response with call!()
//! - Asynchronous fire-and-forget with cast!()
//! - Timeout handling
//! - Multiple concurrent callers
//! - Error handling patterns

use notizia::prelude::*;
use notizia::{call, cast};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};

/// Message protocol for our counter service
#[derive(Debug)]
enum CounterMsg {
    // Synchronous operations (require response)
    GetCount { reply_to: oneshot::Sender<u32> },
    GetStats {
        reply_to: oneshot::Sender<CounterStats>,
    },

    // Asynchronous operations (no response)
    Increment,
    Decrement,
    Add(u32),
    Reset,
    Stop,
}

/// Statistics about the counter
#[derive(Debug, Clone)]
struct CounterStats {
    current: u32,
    total_operations: u32,
}

/// A simple counter service
#[derive(Task)]
#[task(message = CounterMsg)]
struct Counter {
    count: Arc<AtomicU32>,
    operations: Arc<AtomicU32>,
}

impl Runnable<CounterMsg> for Counter {
    async fn start(&self) {
        println!("Counter service started");

        loop {
            match recv!(self) {
                // Synchronous operations - send response back
                Ok(CounterMsg::GetCount { reply_to }) => {
                    let count = self.count.load(Ordering::SeqCst);
                    if let Err(_) = reply_to.send(count) {
                        eprintln!("Failed to send count response");
                    }
                }

                Ok(CounterMsg::GetStats { reply_to }) => {
                    let stats = CounterStats {
                        current: self.count.load(Ordering::SeqCst),
                        total_operations: self.operations.load(Ordering::SeqCst),
                    };
                    if let Err(_) = reply_to.send(stats) {
                        eprintln!("Failed to send stats response");
                    }
                }

                // Asynchronous operations - no response
                Ok(CounterMsg::Increment) => {
                    self.count.fetch_add(1, Ordering::SeqCst);
                    self.operations.fetch_add(1, Ordering::SeqCst);
                }

                Ok(CounterMsg::Decrement) => {
                    self.count.fetch_sub(1, Ordering::SeqCst);
                    self.operations.fetch_add(1, Ordering::SeqCst);
                }

                Ok(CounterMsg::Add(amount)) => {
                    self.count.fetch_add(amount, Ordering::SeqCst);
                    self.operations.fetch_add(1, Ordering::SeqCst);
                }

                Ok(CounterMsg::Reset) => {
                    self.count.store(0, Ordering::SeqCst);
                    self.operations.fetch_add(1, Ordering::SeqCst);
                }

                Ok(CounterMsg::Stop) => {
                    println!("Counter service stopping");
                    break;
                }

                Err(_) => break,
            }
        }
    }

    async fn terminate(&self, reason: TerminateReason) {
        match reason {
            TerminateReason::Normal => {
                let final_count = self.count.load(Ordering::SeqCst);
                let total_ops = self.operations.load(Ordering::SeqCst);
                println!(
                    "Counter service shut down gracefully. Final count: {}, Total operations: {}",
                    final_count, total_ops
                );
            }
            TerminateReason::Panic(msg) => {
                eprintln!("Counter service panicked: {}", msg);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Call/Cast Request-Response Example ===\n");

    // Create and start the counter service
    let counter = Counter {
        count: Arc::new(AtomicU32::new(0)),
        operations: Arc::new(AtomicU32::new(0)),
    };
    let handle = spawn!(counter);

    // Give the service time to start
    sleep(Duration::from_millis(50)).await;

    // =========================================================================
    // Asynchronous operations with cast! (fire-and-forget)
    // =========================================================================
    println!("1. Fire-and-forget operations with cast!\n");

    println!("   Sending async operations...");
    cast!(handle, CounterMsg::Increment).expect("cast failed");
    cast!(handle, CounterMsg::Increment).expect("cast failed");
    cast!(handle, CounterMsg::Increment).expect("cast failed");
    cast!(handle, CounterMsg::Add(5)).expect("cast failed");
    cast!(handle, CounterMsg::Decrement).expect("cast failed");

    println!("   ✓ All cast operations sent (non-blocking)\n");

    // Give time for async operations to process
    sleep(Duration::from_millis(100)).await;

    // =========================================================================
    // Synchronous operations with call! (request-response)
    // =========================================================================
    println!("2. Synchronous request-response with call!\n");

    // Query current count with default timeout (5 seconds)
    println!("   Calling GetCount...");
    match call!(handle, |tx| CounterMsg::GetCount { reply_to: tx }).await {
        Ok(count) => println!("   ✓ Current count: {}\n", count),
        Err(e) => println!("   ✗ Call failed: {:?}\n", e),
    }

    // Query stats with custom timeout (1 second)
    println!("   Calling GetStats with 1s timeout...");
    match call!(handle, |tx| CounterMsg::GetStats { reply_to: tx }, timeout = 1000).await {
        Ok(stats) => {
            println!("   ✓ Stats retrieved:");
            println!("     - Current: {}", stats.current);
            println!("     - Total operations: {}\n", stats.total_operations);
        }
        Err(e) => println!("   ✗ Call failed: {:?}\n", e),
    }

    // =========================================================================
    // Multiple concurrent callers
    // =========================================================================
    println!("3. Multiple concurrent callers\n");

    let handle_arc = Arc::new(handle);
    let mut tasks = vec![];

    println!("   Spawning 5 concurrent tasks...");
    for task_id in 0..5 {
        let handle_clone = handle_arc.clone();
        let task = tokio::spawn(async move {
            // Each task does some operations
            cast!(handle_clone, CounterMsg::Increment).ok();
            cast!(handle_clone, CounterMsg::Increment).ok();

            // Then queries the count
            sleep(Duration::from_millis(50)).await;
            let result = call!(handle_clone, |tx| CounterMsg::GetCount { reply_to: tx }).await;

            (task_id, result)
        });
        tasks.push(task);
    }

    for task in tasks {
        let (task_id, result) = task.await.expect("task panicked");
        match result {
            Ok(count) => println!("   Task {} saw count: {}", task_id, count),
            Err(e) => println!("   Task {} failed: {:?}", task_id, e),
        }
    }
    println!();

    // =========================================================================
    // Error handling
    // =========================================================================
    println!("4. Error handling patterns\n");

    // Reset counter
    cast!(handle_arc, CounterMsg::Reset).ok();
    sleep(Duration::from_millis(50)).await;

    println!("   Pattern 1: Using ? operator for error propagation");
    async fn get_count_or_fail(handle: &TaskHandle<CounterMsg>) -> Result<u32, CallError> {
        let count = call!(handle, |tx| CounterMsg::GetCount { reply_to: tx }).await?;
        Ok(count)
    }

    match get_count_or_fail(&handle_arc).await {
        Ok(count) => println!("   ✓ Count via ? operator: {}\n", count),
        Err(e) => println!("   ✗ Error: {:?}\n", e),
    }

    println!("   Pattern 2: Match on specific error types");
    // This will succeed
    match call!(handle_arc, |tx| CounterMsg::GetCount { reply_to: tx }).await {
        Ok(count) => println!("   ✓ Retrieved count: {}", count),
        Err(CallError::Timeout) => println!("   ✗ Request timed out"),
        Err(CallError::ChannelClosed) => println!("   ✗ Service dropped the response channel"),
        Err(CallError::SendError) => println!("   ✗ Service is not running"),
    }
    println!();

    // =========================================================================
    // Graceful shutdown
    // =========================================================================
    println!("5. Graceful shutdown\n");

    println!("   Sending stop message...");
    cast!(handle_arc, CounterMsg::Stop).expect("stop failed");

    // Wait for the service to shut down
    // Extract handle from Arc (this consumes the last reference)
    let handle = Arc::try_unwrap(handle_arc).unwrap_or_else(|_| {
        panic!("Failed to unwrap Arc - still has references");
    });
    match handle.join().await {
        Ok(TerminateReason::Normal) => println!("   ✓ Service stopped gracefully\n"),
        Ok(TerminateReason::Panic(msg)) => println!("   ✗ Service panicked: {}\n", msg),
        Err(e) => println!("   ✗ Join error: {:?}\n", e),
    }

    println!("=== Example Complete ===");
}
