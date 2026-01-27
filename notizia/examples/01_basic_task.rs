//! Basic task example demonstrating simple message passing.
//!
//! This example shows how to:
//! - Define a message type
//! - Create a task using the derive macro
//! - Spawn and communicate with the task
//! - Gracefully shut down

use notizia::prelude::*;

#[derive(Debug, Clone)]
enum Message {
    Print(String),
    Stop,
}

#[derive(Task)]
#[task(message = Message)]
struct Printer {
    id: usize,
}

impl Runnable<Message> for Printer {
    async fn start(&self) {
        println!("Printer {} started!", self.id);

        loop {
            match recv!(self) {
                Ok(Message::Print(text)) => {
                    println!("[Printer {}] {}", self.id, text);
                }
                Ok(Message::Stop) => {
                    println!("Printer {} stopping...", self.id);
                    break;
                }
                Err(_) => {
                    println!("Printer {} channel closed", self.id);
                    break;
                }
            }
        }

        println!("Printer {} finished!", self.id);
    }
}

#[tokio::main]
async fn main() {
    println!("=== Basic Task Example ===\n");

    // Create and spawn a task
    let printer = Printer { id: 1 };
    let handle = spawn!(printer);

    // Send some messages
    handle
        .send(Message::Print("Hello, World!".to_string()))
        .unwrap();
    handle
        .send(Message::Print("This is Notizia!".to_string()))
        .unwrap();
    handle
        .send(Message::Print("Actor model in Rust.".to_string()))
        .unwrap();

    // Graceful shutdown
    handle.send(Message::Stop).unwrap();
    let _ = handle.join().await;

    println!("\nAll tasks completed!");
}
