# Notizia Gen

This crate defines the procedural macros used by `notizia`.

It exports the `Task` derive macro, which is responsible for analyzing your struct and generating:

1. The `Task` trait implementation.
2. The `Mailbox` management logic.
3. Task-local state storage using `tokio::task_local!`.
4. The setup code that bridges the Tokio `mpsc` channel with your `Runnable` implementation.

## Usage

The macro is used as a derive macro with an explicit message type attribute:

```rust
use notizia_gen::Task;

#[derive(Clone)]
enum MyMessage {
    Start,
    Stop,
}

#[derive(Task)]
#[task(message = MyMessage)]
struct MyTask {
    id: usize,
}
```

## Attribute Format

The `#[task(message = T)]` attribute is required and specifies the message type for the task:

- **Correct**: `#[task(message = MyMessage)]`
- **Incorrect**: `#[Task(MyMessage)]` (old v0.2.0 syntax)
- **Incorrect**: `#[task]` (missing message parameter)
- **Incorrect**: `#[task(msg = MyMessage)]` (wrong parameter name)

The macro will provide helpful error messages if the attribute is malformed.

## Direct Usage

You generally should not use this crate directly. Instead, depend on `notizia`, which re-exports the `Task` macro along with all necessary traits and types:

```toml
[dependencies]
notizia = "0.3"
```

Then import from the prelude:

```rust
use notizia::prelude::*;
```
