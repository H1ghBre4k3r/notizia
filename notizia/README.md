# Notizia

This crate provides the core runtime and traits for the Notizia message passing system. It is designed to make async task communication in Rust feel almost synchronous in its simplicity, while retaining the non-blocking nature of Tokio.

## Core Concepts

- **Task**: A trait implemented via the `#[derive(Task)]` macro that defines a message-handling process. The macro and trait share the same name, following the pattern of `#[derive(Debug)]` for the `Debug` trait.
- **Runnable**: The trait where you define your task's logic.
- **TaskHandle**: A handle for controlling and communicating with a spawned task.
- **TaskRef**: A lightweight reference for sending messages to a task.
- **Mailbox**: Internal state management for the message receiver.
- **Macros**: `spawn!`, `send!`, and `recv!` provide a shorthand DSL for interacting with tasks.

## Usage

Define your state struct and message enum, then implement `Runnable`. The `#[derive(Task)]` macro generates the `Task` trait implementation and necessary boilerplate to wire up the channels.

```rust
use notizia::prelude::*;

// 1. Define your message protocol
// Clone is required for messages passed through channels
#[derive(Debug, Clone)]
enum Signal {
    Ping,
    Pong,
}

// 2. Define your state and derive Task
#[derive(Task)]
#[task(message = Signal)]
struct Worker {
    id: usize,
}

// 3. Implement the logic
impl Runnable<Signal> for Worker {
    async fn start(&self) {
        loop {
            // Type-safe message receiving with error handling
            match recv!(self) {
                Ok(msg) => println!("Worker {} received: {:?}", self.id, msg),
                Err(_) => break,
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // 4. Spawn and enjoy
    let worker = Worker { id: 1 };
    let handle = spawn!(worker);

    // Send a message
    handle.send(Signal::Ping).expect("failed to send");

    // Wait for the task to complete
    handle.join().await;
}
```

## Error Handling

All messaging operations return `Result` types for explicit error handling:

```rust
use notizia::prelude::*;

impl Runnable<Signal> for Worker {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(msg) => {
                    println!("Received: {:?}", msg);
                }
                Err(RecvError::Closed) => {
                    println!("Channel closed, shutting down");
                    break;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    }
}
```

## API Comparison

### Macro Style (Recommended)

```rust
let handle = spawn!(worker);
send!(handle, Signal::Ping)?;
let msg = recv!(worker)?;
```

### Method Style

```rust
let handle = worker.run();  // or worker.spawn()
handle.send(Signal::Ping)?;
let msg = worker.mailbox().recv().await?;
```

Both styles are equivalent—choose what feels most comfortable.
