# Notizia

**Frictionless message passing for the Tokio runtime.**

Async Rust is powerful, but managing channels, task handles, and state synchronization often leads to verbose boilerplate. Notizia cuts through the noise. It provides a thin, type-safe layer over Tokio's primitives, offering an actor-like model that feels native to Rust.

The philosophy is simple: **Concurrency shouldn't hurt.**

## Why Notizia?

We built Notizia to solve the "setup tax" of spawning async tasks. Instead of manually wiring `mpsc` channels and managing mutex locks, you define your state and your messages. Notizia generates the rest.

*   **Zero Boilerplate:** The `#[Task]` macro writes the plumbing for you. You focus on the logic.
*   **Type-Safe Mailboxes:** Messages are strictly typed. No dynamic dispatch, no runtime surprises.
*   **Tokio Native:** Built directly on standard `mpsc` channels and `JoinHandle`s. There is no heavy custom runtime, just ergonomic abstractions.
*   **Unified Semantics:** We use a unified naming convention. The `#[Task]` macro implements the `Task` trait. It just works.

## Quick Start

Add Notizia to your project and define your first task in seconds.

```rust
use notizia::{Task, Runnable, spawn, recv, send};

// 1. Define your message protocol
#[derive(Debug, Clone)]
enum Signal {
    Ping,
    Pong,
}

// 2. Define your state and attach the Task capability
#[Task(Signal)]
struct Worker {
    id: usize,
}

// 3. Implement the logic
impl Runnable<Signal> for Worker {
    async fn start(&self) {
        async move {
            loop {
                // Type-safe message receiving
                let msg = recv!(self);
                println!("Worker {} received: {:?}", self.id, msg);
            }
        }
        .await
    }
}

#[tokio::main]
async fn main() {
    // 4. Spawn and enjoy
    let worker = Worker { id: 1 };
    let handle = spawn!(worker);

    let _ = send!(handle, Signal::Ping);
    
    handle.join().await;
}
```

## Workspace Overview

This repository is organized as a Cargo workspace:

*   **notizia**: The public-facing library. You only need to depend on this.
*   **notizia_gen**: The procedural macro generator. It powers the `#[Task]` attribute but is an internal implementation detail.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
notizia = "0.1"
```
