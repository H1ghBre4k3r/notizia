# Notizia

[![codecov](https://codecov.io/gh/H1ghBre4k3r/notizia/branch/main/graph/badge.svg)](https://codecov.io/gh/H1ghBre4k3r/notizia)
[![CI](https://github.com/H1ghBre4k3r/notizia/workflows/CI/badge.svg)](https://github.com/H1ghBre4k3r/notizia/actions)

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
use notizia::prelude::*;

// 1. Define your message protocol
// Clone is required for messages passed through channels
#[derive(Debug, Clone)]
enum Signal {
    Ping,
    Pong,
}

// 2. Define your state and attach the Task capability
#[derive(Task)]
#[task(message = Signal)]
struct Worker {
    id: usize,
}

// 3. Implement the logic
impl Runnable<Signal> for Worker {
    async fn start(&self) {
        loop {
            // Type-safe message receiving
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

    handle.send(Signal::Ping).expect("failed to send");
    
    handle.join().await;
}
```

## Examples

The `notizia/examples/` directory contains complete, runnable examples demonstrating various patterns:

- **`01_basic_task.rs`** - Simple message passing and graceful shutdown
- **`02_bidirectional.rs`** - Request-response pattern between tasks
- **`03_supervisor.rs`** - Supervisor managing a worker pool
- **`04_error_handling.rs`** - Error recovery and channel closure handling
- **`05_macro_vs_methods.rs`** - Comparison of macro and method-based API styles

Run examples from the workspace root:
```bash
cargo run -p notizia --example 01_basic_task
cargo run -p notizia --example 02_bidirectional
cargo run -p notizia --example 03_supervisor
```

Or from the `notizia/` directory:
```bash
cd notizia
cargo run --example 01_basic_task
```

## Error Handling

Notizia provides explicit, type-safe error handling for all messaging operations. Instead of panicking on channel failures, operations return `Result` types that you can handle gracefully.

**Return Types:**
- `recv!()` returns `Result<T, RecvError>`
- `send!()` returns `Result<(), SendError<T>>`

**RecvError Variants:**
- `Closed` - The channel has been closed
- `Poisoned` - The channel is in an invalid state
- `Timeout` - A receive operation timed out

### Pattern 1: Simple Error Handling with `.unwrap()` / `.expect()`

For prototypes or when you want to panic on errors:

```rust
use notizia::prelude::*;

#[derive(Debug, Clone)]
enum Message { Ping }

#[derive(Task)]
#[task(message = Message)]
struct Worker;

impl Runnable<Message> for Worker {
    async fn start(&self) {
        loop {
            let msg = recv!(self).unwrap();  // Panics if channel closed
            println!("Received: {:?}", msg);
        }
    }
}

#[tokio::main]
async fn main() {
    let worker = Worker;
    let handle = spawn!(worker);
    
    handle.send(Message::Ping).expect("failed to send");
}
```

### Pattern 2: Error Propagation with `?`

For composable error handling in async functions:

```rust
use notizia::{RecvError, SendError};

async fn process_messages(worker: &Worker) -> Result<(), RecvError> {
    loop {
        let msg = recv!(worker)?;  // Propagates errors to caller
        println!("Processing: {:?}", msg);
    }
}
```

### Pattern 3: Explicit Error Handling with Pattern Matching

For graceful shutdown and custom error logic:

```rust
impl Runnable<Message> for Worker {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(msg) => {
                    println!("Received: {:?}", msg);
                }
                Err(RecvError::Closed) => {
                    println!("Channel closed, shutting down gracefully");
                    break;
                }
                Err(RecvError::Timeout) => {
                    println!("Receive timeout, retrying...");
                    continue;
                }
                Err(e) => {
                    eprintln!("Unexpected error: {}", e);
                    break;
                }
            }
        }
    }
}
```

This explicit error handling enables production-ready reliability and graceful shutdown behavior.

## Workspace Overview

This repository is organized as a Cargo workspace:

*   **notizia**: The public-facing library. You only need to depend on this.
*   **notizia_gen**: The procedural macro generator. It powers the `#[Task]` attribute but is an internal implementation detail.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
notizia = "0.3"
tokio = { version = "1", features = ["full"] }
```

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests with coverage
make test-coverage

# Generate HTML coverage report
make coverage
```

### Code Coverage

This project uses `cargo-llvm-cov` for code coverage tracking with a target of 90% coverage. Coverage reports are automatically generated in CI and uploaded to [Codecov](https://codecov.io/gh/H1ghBre4k3r/notizia).

**Local Coverage Reports:**

First, install the coverage tool (one-time setup):
```bash
make install-tools
# or manually: cargo install cargo-llvm-cov
```

Then generate coverage reports:
```bash
make coverage          # Generate HTML report and open in browser
make coverage-check    # Check if coverage meets 90% threshold
make coverage-lcov     # Generate LCOV format for tooling
```

Coverage reports are stored in `target/llvm-cov/html/` and can also be downloaded from CI artifacts on each pull request.
