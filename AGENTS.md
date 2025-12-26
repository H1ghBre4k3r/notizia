# Agent Guide: notizia

## Project Overview

This is a Rust library implementing message passing between threads using channels. It provides a custom task abstraction with mailbox-based communication and macro-based task spawning.

The library supports both synchronous (blocking) and asynchronous (tokio-based) task execution.

**Project Type**: Rust Library (Cargo, Edition 2024)

**Library Name**: `notizia`

## Essential Commands

```bash
# Build the library
cargo build

# Run examples
cargo run --example simple

# Run async example (requires tokio feature)
cargo run --example async --features tokio

# Check for errors without building
cargo check

# Run tests
cargo test

# Run the linter
cargo clippy

# Format code
cargo fmt

# Build for release
cargo build --release

# Generate documentation
cargo doc --open
```

## Code Organization

```
notizia/
├── Cargo.toml          # Project configuration
├── .gitignore         # Git ignore (ignores /target)
├── src/
│   ├── lib.rs          # Library entry point (re-exports std_impl and tokio_impl)
│   ├── std_impl.rs     # Synchronous implementation (Task, proc!, recv!)
│   └── tokio_impl.rs   # Async implementation with tokio (AsyncTask, async_proc!, recv!)
└── examples/
    ├── simple.rs       # Synchronous example
    └── async.rs        # Async example (requires tokio feature)
```

The library provides two implementations:
- **std_impl.rs**: Synchronous task spawning using `std::thread`
- **tokio_impl.rs**: Asynchronous task spawning using `tokio` (requires the `tokio` feature)

## Core Components

### Structs

**`Mailbox<T>`** - Wrapper around a channel sender
- Derives `Clone`
- Contains `Sender<T>`

**`Task<M, R>`** - Represents a spawned task with communication capability
- `M`: Message type (must implement `Clone`)
- `R`: Return type of the task
- Contains `Mailbox<M>` for sending messages to the task
- Contains `Arc<JoinHandle<R>>` for the thread handle
- Methods:
  - `send(&self, payload: T)` - Send message to the task
  - `join(self) -> R` - Join the task and get its return value (takes ownership)

**`SpawnedTask<T>`** - Internal struct for holding the receiver side
- Contains `Receiver<T>`
- Used internally by `spawn_task`

### Async Structs (requires `tokio` feature)

**`AsyncTask<M, R>`** - Async version of `Task` for tokio-based task spawning
- `M`: Message type (must implement `Clone`)
- `R`: Return type of the task
- Contains `AsyncMailbox<M>` for sending messages to the task
- Contains `JoinHandle<R>` for the tokio task handle
- Methods:
  - `async fn send(&self, payload: T)` - Async send message to the task
  - `async fn join(self) -> R` - Async join the task and get its return value (takes ownership)

**`AsyncMailbox<T>`** - Internal async wrapper around tokio channel sender
- Derives `Clone`
- Contains `tokio::sync::mpsc::Sender<T>`

### Macros

**`proc!`** - Spawns a task and returns a `Task<M, R>`
```rust
let task: Task<u32, u32> = proc! {
    // Task code here
    // Use recv!() to receive messages
    // Return value becomes the R type
};
```

**`recv!()`** - Receives a message from the task's mailbox
- Can only be used inside `proc!` blocks
- Blocks until a message is available
- Returns the message of type `M`

**`async_proc!`** - Async version of `proc!` for spawning async tasks
- Requires `tokio` feature
- Returns an `AsyncTask<M, R>`
- Can use `await` expressions inside the task body
```rust
let task: AsyncTask<u32, u32> = async_proc! {
    let mut counter = 0;
    for _ in 0..10 {
        let val = recv!();
        counter += val;
    }
    counter
};
```

Inside `async_proc!`, the `recv!()` macro is redefined to be async:
```rust
macro_rules! recv {
    () => { _receiver.recv().await.unwrap() }
}
```

### Functions

**`spawn_task<M, R, Func>(func: Func) -> Task<M, R>`** - Low-level task spawning
- `M`: Message type (must be `Send + 'static`)
- `R`: Return type (must be `Send + 'static`)
- `Func`: Closure taking `Receiver<M>` and returning `R` (must be `FnOnce + Send + 'static`)

**`spawn_async_task<M, R, Output, Func>(func: Func) -> AsyncTask<M, Output>`** - Low-level async task spawning (requires tokio feature)
- `M`: Message type (must be `Send + 'static`)
- `R`: Future type (must be `Send + 'static + Future<Output = Output>`)
- `Output`: The output type of the future (must be `Send + 'static`)
- `Func`: Closure taking `Receiver<M>` and returning a Future `R` (must be `FnOnce + Send + 'static`)
- Uses a tokio channel with buffer size of 64

## Code Patterns

### Task Spawning

**Synchronous:**
```rust
// Spawn a task that receives u32 values and returns u32
let task: Task<u32, u32> = proc! {
    let mut accumulator = 0;
    for _ in 0..10 {
        let value = recv!();
        accumulator += value;
    }
    accumulator
};

// Send messages to the task
task.send(5);
task.send(10);

// Join the task to get result
let result = task.join();
```

**Asynchronous (requires tokio feature):**
```rust
#[tokio::main]
async fn main() {
    // Spawn an async task that receives u32 values and returns u32
    let task: AsyncTask<u32, u32> = async_proc! {
        let mut accumulator = 0;
        for _ in 0..10 {
            let value = recv!();
            accumulator += value;
        }
        accumulator
    };

    // Send messages to the task (async)
    task.send(5).await;
    task.send(10).await;

    // Join the task to get result (async)
    let result = task.join().await;
}
```

### Task Communication Pattern

**Synchronous:**
Tasks communicate via message passing using channels:
1. Spawner uses `task.send()` to send messages
2. Task uses `recv!()` macro to receive messages
3. Task computes a return value
4. Spawner calls `task.join()` to get the result

**Asynchronous:**
Tasks communicate similarly but with async operations:
1. Spawner uses `task.send().await` to send messages
2. Task uses `recv!()` macro (which internally calls `receiver.recv().await`)
3. Task computes a return value (can use async operations)
4. Spawner calls `task.join().await` to get the result

## Naming Conventions

- **Structs**: PascalCase (`Mailbox`, `Task`, `SpawnedTask`, `AsyncTask`, `AsyncMailbox`)
- **Variables**: snake_case (`counter`, `receiver`, `mailbox`, `handle`)
- **Functions**: snake_case (`spawn_task`, `spawn_async_task`)
- **Macros**: snake_case with `!` suffix (`recv!`, `proc!`, `async_proc!`)
- **Type Parameters**: Single uppercase letters (`T`, `M`, `R`, `Output`)

## Important Gotchas

### Ownership Issues

**`Task::join()` takes ownership**: The method signature is `join(self) -> R`, meaning the task is consumed when you call join. You cannot call `join()` multiple times on the same task.

```rust
// ❌ This will fail if you need the task after joining
let result = task.join();
task.send(1); // Error: task was moved

// ✅ Clone the task if needed
let task_clone = task.clone();
let result = task_clone.join();
task.send(1); // This works
```

**Closure trait bounds**: When using `spawn_task`, the closure implements `FnOnce`, meaning it can only be called once. This allows consuming captured variables.

**Async closure trait bounds**: When using `spawn_async_task`, the closure returns a Future. The async block inside `async_proc!` captures the receiver and returns an async block.

### Type Constraints

**Message type must implement `Clone`**: When defining `Task<M, R>`, the message type `M` must implement `Clone` because `Mailbox<M>` derives `Clone`.

**Thread safety requirements**: The generic constraints require types to be:
- `Send` - Can be transferred between threads
- `'static` - Has no borrowed references (owned data only)

### Macro Scope

The `recv!()` macro is only available inside `proc!` blocks (sync) or `async_proc!` blocks (async). It defines a local macro within the spawned closure/async block.

**Important**: Inside `async_proc!`, `recv!()` uses `.await` internally, so you must be in an async context.

### Async/Await

When using async features:
- All `send()` and `join()` methods on `AsyncTask` are async and require `.await`
- The `recv!()` macro inside `async_proc!` is also async (uses `receiver.recv().await`)
- Tasks can contain arbitrary async code (e.g., HTTP requests, database operations, sleep)

## Library Usage

### Synchronous Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
notizia = "0.1"
```

Then use the library:

```rust
use notizia::{Task, proc};

let task: Task<u32, u32> = proc! {
    let mut total = 0;
    for _ in 0..5 {
        let val = recv!();
        total += val;
    }
    total
};

for i in 1..=5 {
    task.send(i);
}

let result = task.join();
```

### Asynchronous Usage

Add this to your `Cargo.toml` with the `tokio` feature:

```toml
[dependencies]
notizia = { version = "0.1", features = ["tokio"] }
tokio = { version = "1", features = ["full"] }
```

Then use the library in an async context:

```rust
use notizia::{AsyncTask, async_proc};

#[tokio::main]
async fn main() {
    let task: AsyncTask<u32, u32> = async_proc! {
        let mut total = 0;
        for _ in 0..5 {
            let val = recv!();
            total += val;
        }
        total
    };

    for i in 1..=5 {
        task.send(i).await;
    }

    let result = task.join().await;
}
```

## Dependencies

The library has minimal external dependencies:

**Always available:**
- Rust standard library:
  - `std::sync::mpsc` - Multi-producer, single-consumer channels
  - `std::sync::Arc` - Atomic reference counting
  - `std::thread` - Thread spawning and management

**With `tokio` feature (optional):**
- `tokio` version 1.48.0+ with full features:
  - `tokio::sync::mpsc` - Async channels
  - `tokio::task::JoinHandle` - Async task handles
  - `tokio::spawn` - Async task spawning

## Sync vs Async

**Use `Task` (sync) when:**
- Your application doesn't use tokio
- You prefer blocking operations for simplicity
- You're building CLI tools or synchronous services

**Use `AsyncTask` (async) when:**
- Your application already uses tokio
- You need to perform async operations inside tasks (HTTP, database, etc.)
- You want non-blocking concurrent operations
- You're building high-throughput network services

**Key Differences:**
| Aspect | Task (Sync) | AsyncTask (Async) |
|--------|-------------|-------------------|
| Runtime | `std::thread` | tokio runtime |
| Channel | `std::sync::mpsc` | `tokio::sync::mpsc` |
| `send()` | Blocking | Async (returns Future) |
| `join()` | Blocking | Async (returns Future) |
| `recv!()` | Blocking | Async (uses `.await`) |
| Feature | Always available | Requires `tokio` feature |
