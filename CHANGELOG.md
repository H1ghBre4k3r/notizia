# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-01-27

### Breaking Changes

#### Macro Syntax Change
The Task macro has been changed from an attribute macro to a derive macro with explicit message type specification.

**Old syntax (v0.2.0):**
```rust
#[Task(Message)]
struct MyTask { }
```

**New syntax (v0.3.0):**
```rust
#[derive(Task)]
#[task(message = Message)]
struct MyTask { }
```

**Migration guide:**
1. Replace `#[Task(MessageType)]` with `#[derive(Task)]`
2. Add `#[task(message = MessageType)]` attribute below the derive
3. Update imports to use `prelude::*` for convenience

#### Error Handling Changes
- `recv!()` now returns `Result<T, RecvError>` instead of panicking
- `Mailbox::recv()` returns `Result<T, RecvError>` instead of unwrapping
- All messaging operations now use explicit error handling

**Migration:**
```rust
// Old: panics on error
let msg = recv!(self);

// New: returns Result
let msg = recv!(self)?;
// or
match recv!(self) {
    Ok(msg) => { /* handle message */ }
    Err(e) => { /* handle error */ }
}
```

### Added

- **Error Types**: Introduced `RecvError` and `SendError` for explicit error handling
  - `RecvError::Closed` - Channel closed
  - `RecvError::Poisoned` - Channel in invalid state
  - `RecvError::Timeout` - Receive operation timed out

- **TaskRef**: New lightweight reference type for sending messages without full handle ownership
  - `TaskRef::send()` - Send messages to a task
  - `TaskRef::clone()` - Clone the reference for sharing across tasks
  - `Task::this()` - Get a reference to the current task from within

- **Comprehensive Examples**: Added 5 complete example programs
  - `01_basic_task.rs` - Simple message passing and graceful shutdown
  - `02_bidirectional.rs` - Request-response pattern between tasks
  - `03_supervisor.rs` - Supervisor managing a worker pool
  - `04_error_handling.rs` - Error recovery and channel closure handling
  - `05_macro_vs_methods.rs` - Comparison of macro and method-based API styles

- **Prelude Module**: Convenient `prelude::*` import for common types and traits
  - Exports `Task`, `Runnable`, `TaskHandle`, `TaskRef`
  - Exports macros: `spawn!`, `send!`, `recv!`
  - Exports error types: `RecvError`, `SendError`

- **Test Coverage**: Comprehensive test suite with 93.94% code coverage
  - 40 integration tests covering concurrent messaging, error handling, macro integration
  - Compile-fail tests for macro error cases
  - Expansion tests for macro code generation

- **API Methods**: Additional convenience methods on `TaskHandle`
  - `TaskHandle::this()` - Get a `TaskRef` to the task
  - `TaskHandle::kill()` - Abort the task

### Changed

- **Improved Documentation**: Enhanced inline documentation with examples
- **Better Error Messages**: Macro errors now provide helpful guidance with examples
- **Task Trait**: Refined trait implementation with better ergonomics

### Fixed

- **Channel Lifecycle**: Proper handling of channel closure and cleanup
- **State Management**: Improved task-local state handling with tokio::task_local
- **Error Propagation**: Consistent error propagation throughout the API

### Infrastructure

- **CI/CD**: Added comprehensive GitHub Actions workflows
  - Multi-platform testing (Ubuntu, Windows, macOS)
  - Code coverage tracking with Codecov
  - Clippy linting with strict warnings
  - Rustfmt formatting checks
  - PR template validation

- **Code Coverage**: Achieved 93.94% line coverage with 90% threshold enforcement
- **Dependabot**: Automated dependency updates for GitHub Actions

## [0.2.0] - 2024-01-20

### Added

- Initial public release
- Basic `#[Task]` attribute macro
- `TaskHandle` for managing spawned tasks
- `Mailbox` for receiving messages
- `Runnable` trait for task logic
- Basic macros: `spawn!`, `send!`, `recv!`
- Tokio integration with unbounded channels

### Notes

Version 0.2.0 was the first tagged release. Earlier versions (0.1.x) were experimental and not published to crates.io.

---

## Migration Guide: v0.2.0 → v0.3.0

### Step 1: Update Dependencies

```toml
[dependencies]
notizia = "0.3"
tokio = { version = "1", features = ["full"] }
```

### Step 2: Update Macro Syntax

Replace all `#[Task(MessageType)]` with the new derive syntax:

```diff
- #[Task(Signal)]
+ #[derive(Task)]
+ #[task(message = Signal)]
  struct Worker {
      id: usize,
  }
```

### Step 3: Handle Errors Explicitly

Update message receiving to handle errors:

```diff
  impl Runnable<Signal> for Worker {
      async fn start(&self) {
          loop {
-             let msg = recv!(self);
+             match recv!(self) {
+                 Ok(msg) => {
                      println!("Received: {:?}", msg);
+                 }
+                 Err(_) => break,
+             }
          }
      }
  }
```

### Step 4: Update Imports (Optional but Recommended)

Use the prelude for convenience:

```diff
- use notizia::{Task, Runnable, spawn, send, recv};
+ use notizia::prelude::*;
```

### Step 5: Test Your Application

Run your test suite to ensure everything compiles and runs correctly:

```bash
cargo test
cargo clippy
```

---

[0.3.0]: https://github.com/H1ghBre4k3r/notizia/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/H1ghBre4k3r/notizia/releases/tag/v0.2.0
