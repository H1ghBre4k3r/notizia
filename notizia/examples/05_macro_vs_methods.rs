//! Comparison of macro vs method-based API.
//!
//! This example demonstrates both ways to:
//! - Spawn tasks
//! - Send messages
//! - Receive messages
//!
//! Both styles are functionally equivalent - choose based on preference.

use notizia::prelude::*;

#[derive(Debug, Clone)]
enum Message {
    Echo(String),
    Stop,
}

#[derive(Task)]
#[task(message = Message)]
struct MacroStyleTask {
    name: String,
}

impl Runnable<Message> for MacroStyleTask {
    async fn start(&self) {
        println!("[Macro Style] {} started", self.name);

        loop {
            // Using macro style
            match recv!(self) {
                Ok(Message::Echo(text)) => {
                    println!("[Macro Style] {}: {}", self.name, text);
                }
                Ok(Message::Stop) => break,
                Err(_) => break,
            }
        }
    }
}

#[derive(Task)]
#[task(message = Message)]
struct MethodStyleTask {
    name: String,
}

impl Runnable<Message> for MethodStyleTask {
    async fn start(&self) {
        println!("[Method Style] {} started", self.name);

        loop {
            // Using method style
            match self.recv().await {
                Ok(Message::Echo(text)) => {
                    println!("[Method Style] {}: {}", self.name, text);
                }
                Ok(Message::Stop) => break,
                Err(_) => break,
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Macro vs Method API Comparison ===\n");

    println!("1. Spawning Tasks\n");

    // Macro style
    let task1 = MacroStyleTask {
        name: "MacroTask".to_string(),
    };
    let handle1 = spawn!(task1); // Using spawn! macro

    // Method style (equivalent)
    let task2 = MethodStyleTask {
        name: "MethodTask".to_string(),
    };
    let handle2 = task2.spawn(); // Using .spawn() method

    println!("\n2. Sending Messages\n");

    // Both use the same API for sending
    send!(handle1, Message::Echo("Hello from macro!".to_string())).unwrap();
    handle2
        .send(Message::Echo("Hello from method!".to_string()))
        .unwrap();

    println!("\n3. Message Reception (check the impl above)\n");
    println!("  - Macro style: recv!(self)");
    println!("  - Method style: self.recv().await");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("\n4. Getting TaskRef\n");

    // Both styles
    let _ref1 = handle1.this(); // Method style
    let _ref2 = handle2.this(); // Method style (macros don't provide alternative here)

    println!("  Both use: handle.this()");

    // Cleanup
    handle1.send(Message::Stop).unwrap();
    handle2.send(Message::Stop).unwrap();

    let _ = handle1.join().await;
    let _ = handle2.join().await;

    println!("\nSummary:");
    println!("  - spawn!(task) ≡ task.spawn() ≡ task.run()");
    println!("  - send!(handle, msg) could be used but handle.send(msg) is idiomatic");
    println!("  - recv!(self) ≡ self.recv().await");
    println!("\nChoose the style that fits your preferences!");
}
