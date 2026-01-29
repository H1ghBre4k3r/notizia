//! Demonstrates the #[message] macro for reducing boilerplate in message enums.
//!
//! The #[message] macro automatically injects reply_to fields for request variants,
//! making message definitions cleaner and more maintainable.

use notizia::message;
use notizia::prelude::*;
use notizia::{call, cast};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

// Custom type for stats response
#[derive(Debug, Clone)]
struct CounterStats {
    current: u32,
    total_operations: u64,
}

// Message enum using the #[message] macro
// The #[request(reply = T)] attribute automatically adds:
//   reply_to: tokio::sync::oneshot::Sender<T>
#[message]
#[derive(Debug)]
enum CounterMsg {
    // Request variants - will have reply_to field injected
    #[request(reply = u32)]
    GetCount,
    
    #[request(reply = CounterStats)]
    GetStats,
    
    // Cast variants - no reply_to field (fire-and-forget)
    Increment,
    Decrement,
    Add(u32),
    Reset,
    Stop,
}

// Counter task that maintains state
#[derive(Task)]
#[task(message = CounterMsg)]
struct Counter {
    count: Arc<AtomicU32>,
    operations: Arc<AtomicU32>,
}

impl Runnable<CounterMsg> for Counter {
    async fn start(&self) {
        println!("Counter task started");
        
        loop {
            match recv!(self) {
                // Handle request messages - send response via reply_to
                Ok(CounterMsg::GetCount { reply_to }) => {
                    let count = self.count.load(Ordering::SeqCst);
                    println!("GetCount request - responding with: {}", count);
                    if reply_to.send(count).is_err() {
                        eprintln!("Failed to send count response");
                    }
                }
                
                Ok(CounterMsg::GetStats { reply_to }) => {
                    let stats = CounterStats {
                        current: self.count.load(Ordering::SeqCst),
                        total_operations: self.operations.load(Ordering::SeqCst) as u64,
                    };
                    println!("GetStats request - responding with: {:?}", stats);
                    if reply_to.send(stats).is_err() {
                        eprintln!("Failed to send stats response");
                    }
                }
                
                // Handle cast messages - no response needed
                Ok(CounterMsg::Increment) => {
                    self.count.fetch_add(1, Ordering::SeqCst);
                    self.operations.fetch_add(1, Ordering::SeqCst);
                    println!("Incremented counter");
                }
                
                Ok(CounterMsg::Decrement) => {
                    self.count.fetch_sub(1, Ordering::SeqCst);
                    self.operations.fetch_add(1, Ordering::SeqCst);
                    println!("Decremented counter");
                }
                
                Ok(CounterMsg::Add(value)) => {
                    self.count.fetch_add(value, Ordering::SeqCst);
                    self.operations.fetch_add(1, Ordering::SeqCst);
                    println!("Added {} to counter", value);
                }
                
                Ok(CounterMsg::Reset) => {
                    self.count.store(0, Ordering::SeqCst);
                    self.operations.fetch_add(1, Ordering::SeqCst);
                    println!("Reset counter to 0");
                }
                
                Ok(CounterMsg::Stop) => {
                    println!("Stop message received, shutting down");
                    break;
                }
                
                Err(_) => {
                    println!("Channel closed, shutting down");
                    break;
                }
            }
        }
        
        println!("Counter task stopped");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Message Macro Demo ===\n");
    
    // Create and spawn counter task
    let counter = Counter {
        count: Arc::new(AtomicU32::new(0)),
        operations: Arc::new(AtomicU32::new(0)),
    };
    
    let handle = spawn!(counter);
    
    // Demonstrate cast operations (fire-and-forget)
    println!("--- Cast Operations (Fire-and-Forget) ---");
    cast!(handle, CounterMsg::Increment)?;
    cast!(handle, CounterMsg::Increment)?;
    cast!(handle, CounterMsg::Add(5))?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Demonstrate call operations (request-response)
    println!("\n--- Call Operations (Request-Response) ---");
    
    // Get current count with default timeout (5 seconds)
    let count = call!(handle, |tx| CounterMsg::GetCount { reply_to: tx }).await?;
    println!("Current count: {}\n", count);
    
    // More operations
    cast!(handle, CounterMsg::Decrement)?;
    cast!(handle, CounterMsg::Add(10))?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Get stats with custom timeout (1 second)
    let stats = call!(
        handle,
        |tx| CounterMsg::GetStats { reply_to: tx },
        timeout = 1000
    ).await?;
    println!("Statistics: {:?}\n", stats);
    
    // Get final count
    let final_count = call!(handle, |tx| CounterMsg::GetCount { reply_to: tx }).await?;
    println!("Final count: {}", final_count);
    
    // Stop the counter
    println!("\n--- Shutting Down ---");
    cast!(handle, CounterMsg::Stop)?;
    
    // Wait for task to complete
    handle.join().await?;
    
    println!("\n=== Demo Complete ===");
    
    Ok(())
}
