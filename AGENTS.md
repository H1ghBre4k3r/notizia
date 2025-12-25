# Agent Guide: notizia

## Project Overview

This is a Rust library implementing message passing between threads using channels. It provides a custom task abstraction with mailbox-based communication and macro-based task spawning.

**Project Type**: Rust Library (Cargo, Edition 2024)

**Library Name**: `notizia`

## Essential Commands

```bash
# Build the library
cargo build

# Run examples
cargo run --example simple

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
│   └── lib.rs          # Library code
└── examples/
    └── simple.rs       # Example usage
```

The library code is in `src/lib.rs`. See `examples/simple.rs` for usage examples.

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

### Functions

**`spawn_task<M, R, Func>(func: Func) -> Task<M, R>`** - Low-level task spawning
- `M`: Message type (must be `Send + 'static`)
- `R`: Return type (must be `Send + 'static`)
- `Func`: Closure taking `Receiver<M>` and returning `R` (must be `FnOnce + Send + 'static`)

## Code Patterns

### Task Spawning

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

### Task Communication Pattern

Tasks communicate via message passing using channels:
1. Spawner uses `task.send()` to send messages
2. Task uses `recv!()` macro to receive messages
3. Task computes a return value
4. Spawner calls `task.join()` to get the result

## Naming Conventions

- **Structs**: PascalCase (`Mailbox`, `Task`, `SpawnedTask`)
- **Variables**: snake_case (`counter`, `receiver`, `mailbox`, `handle`)
- **Functions**: snake_case (`spawn_task`)
- **Macros**: snake_case with `!` suffix (`recv!`, `proc!`)
- **Type Parameters**: Single uppercase letters (`T`, `M`, `R`)

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

### Type Constraints

**Message type must implement `Clone`**: When defining `Task<M, R>`, the message type `M` must implement `Clone` because `Mailbox<M>` derives `Clone`.

**Thread safety requirements**: The generic constraints require types to be:
- `Send` - Can be transferred between threads
- `'static` - Has no borrowed references (owned data only)

### Macro Scope

The `recv!()` macro is only available inside `proc!` blocks. It defines a local macro within the spawned closure.

## Library Usage

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

## Dependencies

Currently the project has no external dependencies (the `[dependencies]` section in `Cargo.toml` is empty). All functionality is built on Rust's standard library:
- `std::sync::mpsc` - Multi-producer, single-consumer channels
- `std::sync::Arc` - Atomic reference counting
- `std::thread` - Thread spawning and management
