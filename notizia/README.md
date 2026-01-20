# Notizia

This crate provides the core runtime and traits for the Notizia message passing system. It is designed to make async task communication in Rust feel almost synchronous in its simplicity, while retaining the non-blocking nature of Tokio.

## Core Concepts

- **Proc**: A trait (typically implemented via macro) that defines a message-handling process.
- **Runnable**: The trait where you define your process's logic.
- **Mailbox**: Internal state management for the message receiver.
- **Macros**: `spawn!`, `send!`, and `recv!` provide a shorthand DSL for interacting with processes.

## Usage

Define your state struct and message enum, then implement `Runnable`. The `#[Proc]` attribute generates the necessary boilerplate to wire up the channels.

```rust
use notizia::{Proc, Runnable, recv, send, spawn};

// The procedural macro generates the mailbox and trait implementations
#[notizia_gen::Proc(Message)]
struct MyProcess {}

#[derive(Debug, Clone)]
enum Message {
    Ping,
    Pong,
}

impl Runnable<Message> for MyProcess {
    async fn start(&self) {
        // This block runs in the spawned task
        async move {
            loop {
                let msg = recv!(self);
                println!("Received: {:?}", msg);
                // Handle exit conditions or logic here
            }
        }
        .await
    }
}

#[tokio::main]
async fn main() {
    let process = MyProcess {};
    
    // Spawns the task and returns a handle
    let handle = spawn!(process);

    // Sends a message to the running task
    if let Err(e) = send!(handle, Message::Ping) {
        eprintln!("Failed to send message: {}", e);
    }

    // Wait for the task to complete
    handle.join().await;
}
```
