# Notizia Roadmap

This document outlines the development plan for Notizia v0.3 and beyond. The primary goal is to bring Elixir/OTP-inspired concurrency patterns to Rust while maintaining a minimal, Tokio-native architecture.

## Design Philosophy

Notizia aims to provide the ergonomics and safety of Elixir's actor model without introducing a custom runtime. All abstractions compile down to standard Tokio primitives. We favor compile-time guarantees over runtime flexibility, and explicit error handling over silent failures.

The v0.3 release represents a significant step toward feature parity with Elixir/OTP while keeping the codebase minimal and focused on message-passing fundamentals.

## Current State (v0.2.0)

The existing implementation provides:
- Procedural macro-based task definition (`#[Task]`)
- Type-safe message passing via unbounded channels
- Basic task spawning and lifecycle management
- Macros for ergonomic message sending and receiving

### Known Limitations

1. **Error handling**: All channel operations use `unwrap()`, causing panics on failure
2. **No supervision**: Tasks do not restart on crash
3. **No request/response**: Only async fire-and-forget messaging
4. **No process discovery**: Tasks must be manually tracked via handles
5. **Limited lifecycle control**: No graceful shutdown hooks
6. **Isolation**: No crash propagation or monitoring between tasks

## Version 0.3.0: Foundation

**Target Timeline:** 12 weeks from start of development
**Milestone:** [v0.3.0](https://github.com/H1ghBre4k3r/notizia/milestone/1)

### Phase 1: Error Handling and Lifecycle (Weeks 1-2)

**Issues:** [Result-based send/recv error handling](https://github.com/H1ghBre4k3r/notizia/issues/3), [Graceful shutdown and lifecycle hooks](https://github.com/H1ghBre4k3r/notizia/issues/4)

#### Result-Based Error Propagation

Replace all `unwrap()` calls with proper error types:

```rust
pub enum RecvError {
    Closed,
    Poisoned,
    Timeout,
}

pub enum SendError<T> {
    Disconnected(T),
    Full(T),
}
```

The `recv!` and `send!` macros will return `Result` types. This is a breaking change but necessary for production reliability.

#### Graceful Shutdown

Add lifecycle hooks to the `Runnable` trait:

```rust
pub trait Runnable<T>: Send + Sync {
    async fn start(&self);
    
    async fn terminate(&self, reason: ShutdownReason) {
        // Default no-op implementation
    }
}

pub enum ShutdownReason {
    Normal,
    Killed,
    Panic(String),
    Timeout,
}
```

Tasks will have two shutdown methods:
- `shutdown()`: Sends shutdown signal, waits for `terminate()` completion
- `kill()`: Immediate abort (existing behavior)

**Deliverables:**
- New error types in `src/core/errors.rs`
- Updated `Mailbox::recv()` signature
- `ShutdownReason` enum and `terminate()` hook
- Migration guide for v0.2 users
- Updated examples

### Phase 2: GenServer Pattern (Weeks 3-4)

**Issues:** [Call/cast request-response API](https://github.com/H1ghBre4k3r/notizia/issues/5), [Named process registry and discovery macros](https://github.com/H1ghBre4k3r/notizia/issues/6)

#### Call/Cast Message Semantics

Implement synchronous request/response alongside async messaging:

```rust
#[derive(Clone)]
enum Msg {
    #[request(reply = StatusReply)]
    GetStatus { reply_to: oneshot::Sender<StatusReply> },
    
    Increment,  // Regular async message
}
```

New macro syntax:

```rust
let status = call!(worker, Msg::GetStatus, timeout = 5000).await?;
```

Implementation uses `tokio::sync::oneshot` channels with `tokio::time::timeout` for deadline enforcement.

#### Named Process Registry

Global process registration and discovery:

```rust
pub struct Registry<T> {
    tasks: DashMap<String, TaskRef<T>>,
}

impl<T> Registry<T> {
    pub fn register(&self, name: impl Into<String>, task: TaskRef<T>) 
        -> Result<(), RegistryError>;
    pub fn whereis(&self, name: &str) -> Option<TaskRef<T>>;
    pub fn unregister(&self, name: &str) -> Option<TaskRef<T>>;
}
```

Macro support:

```rust
spawn!(worker, name = "worker_1")?;
send_to!("worker_1", Msg::Ping)?;
let task = whereis!("worker_1")?;
```

The registry uses type erasure internally with `downcast` for type safety. Tasks are automatically unregistered on death via monitoring.

**Deliverables:**
- `call!` macro with timeout support
- Registry implementation with `DashMap`
- Extended `spawn!` macro for named registration
- `send_to!` and `whereis!` macros
- Tests for registry edge cases (double registration, stale entries)

### Phase 3: Linking and Monitoring (Weeks 5-6)

**Issues:** [Process linking with crash propagation](https://github.com/H1ghBre4k3r/notizia/issues/7), [Process monitoring API and callbacks](https://github.com/H1ghBre4k3r/notizia/issues/8)

#### Process Linking

Bidirectional crash propagation:

```rust
pub struct Link {
    task: TaskRef<AnyMsg>,
    bidirectional: bool,
}
```

When a linked task crashes, all linked peers receive shutdown signals.

```rust
let worker2 = spawn_link!(Worker2, link_to = worker1)?;
```

#### Process Monitoring

One-way death notifications:

```rust
pub struct Monitor {
    task_id: TaskId,
    on_exit: Box<dyn Fn(ShutdownReason) + Send>,
}
```

Monitors invoke callbacks on task death without affecting the monitoring task:

```rust
monitor!(worker, on_exit = |reason| {
    error!("Task died: {:?}", reason);
});
```

**Deliverables:**
- `Link` and `Monitor` types
- `spawn_link!` and `monitor!` macros
- Crash propagation logic
- Tests for linking scenarios (cascading failures, cycles)
- Documentation on proper linking patterns

### Phase 4: Selective Receive (Week 7)

**Issue:** [Selective receive mailbox and recv_match macro](https://github.com/H1ghBre4k3r/notizia/issues/9)

#### Pattern-Based Message Consumption

Tokio's `mpsc` channels do not support message peeking. We implement selective receive via buffering:

```rust
pub struct SelectiveMailbox<T> {
    pending: VecDeque<T>,
    receiver: UnboundedReceiver<T>,
}

impl<T> SelectiveMailbox<T> {
    pub async fn recv_match<F>(&mut self, matcher: F) -> T
    where
        F: Fn(&T) -> bool
    {
        // Check buffered messages first
        if let Some(idx) = self.pending.iter().position(|msg| matcher(msg)) {
            return self.pending.remove(idx).unwrap();
        }
        
        // Pull messages until match found
        loop {
            let msg = self.receiver.recv().await.unwrap();
            if matcher(&msg) {
                return msg;
            }
            self.pending.push_back(msg);
        }
    }
}
```

Macro usage:

```rust
recv_match!(self, |msg| matches!(msg, Msg::Ping | Msg::Pong))
```

This feature is opt-in via `#[Task(Msg, selective = true)]` due to buffering overhead.

**Deliverables:**
- `SelectiveMailbox` implementation
- `recv_match!` macro
- Benchmarks comparing overhead vs standard `recv`
- Documentation on performance implications
- Example demonstrating use case

### Phase 5: Supervision Trees (Weeks 8-10)

**Issues:** [Supervision core: trait, strategies, restart tracking](https://github.com/H1ghBre4k3r/notizia/issues/10), [SupervisorTree builder and #[Supervisor] macro](https://github.com/H1ghBre4k3r/notizia/issues/11)

#### Supervisor Trait

```rust
pub trait Supervisor {
    type Child: Task<ChildMsg>;
    
    fn strategy(&self) -> SupervisionStrategy;
    fn children(&self) -> Vec<Self::Child>;
}

pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}
```

Strategies:
- **OneForOne**: Restart only the crashed child
- **OneForAll**: Restart all children if any crashes
- **RestForOne**: Restart crashed child and all siblings started after it

#### Restart Policies

Supervisors track restart attempts within a time window:

```rust
#[Supervisor(
    strategy = OneForOne,
    max_restarts = 3,
    max_seconds = 5
)]
struct AppSupervisor {
    workers: Vec<Worker>,
}
```

If restart limits are exceeded, the supervisor escalates the failure to its parent.

#### Supervision Tree Builder

Declarative tree construction API:

```rust
SupervisorTree::new()
    .child(Worker1, name = "w1")
    .child(Worker2, name = "w2")
    .supervisor(
        Supervisor::new(strategy = OneForAll)
            .child(Worker3, name = "w3")
            .child(Worker4, name = "w4")
    )
    .start()?;
```

**Deliverables:**
- `Supervisor` trait and strategies
- Restart tracking (count and time window)
- `#[Supervisor]` procedural macro
- `SupervisorTree` builder API
- Comprehensive tests (restart scenarios, escalation)
- Example: HTTP server with supervised worker pool

### Phase 6: Observability (Week 11)

**Issues:** [Task metrics and inspect macro](https://github.com/H1ghBre4k3r/notizia/issues/12), [Tracing integration and observability validation](https://github.com/H1ghBre4k3r/notizia/issues/13)

#### Task Metrics

```rust
pub struct TaskMetrics {
    pub messages_received: AtomicU64,
    pub messages_sent: AtomicU64,
    pub restarts: AtomicU32,
    pub state: TaskState,
}
```

Inspection API:

```rust
let metrics = inspect!(worker)?;
println!("Received: {}", metrics.messages_received);
```

#### Tracing Integration

Automatic instrumentation of message passing:

```rust
// Auto-emits tracing spans
recv!(self);  // span: "recv", msg_type = "Ping"
send!(worker, Msg::Pong);  // span: "send", target = "worker"
```

Uses the `tracing` crate for structured logging integration.

**Deliverables:**
- `TaskMetrics` struct and `inspect!` macro
- `tracing` integration for recv/send operations
- Example with metrics dashboard (CLI or web)
- Performance impact analysis

### Phase 7: Documentation (Week 12)

**Issues:** [Rustdoc coverage and API documentation](https://github.com/H1ghBre4k3r/notizia/issues/14), [Examples suite for v0.3](https://github.com/H1ghBre4k3r/notizia/issues/15), [Migration guide from v0.2 to v0.3](https://github.com/H1ghBre4k3r/notizia/issues/16), [Comparison documentation and v0.3 announcement](https://github.com/H1ghBre4k3r/notizia/issues/17)

#### API Documentation

Complete rustdoc coverage for all public APIs with:
- Usage examples for each function/macro
- Common pitfalls and best practices
- Cross-references between related concepts

#### Examples

Minimum 5 comprehensive examples:
1. HTTP server with supervision
2. Worker pool with backpressure
3. Request/response pattern (call/cast)
4. Process registry and discovery
5. Supervision tree with multiple strategies

#### Migration Guide

Step-by-step v0.2 to v0.3 migration:
- Error handling changes
- API signature updates
- New features and how to adopt them
- Common migration issues

#### Comparison Documentation

Technical comparison with:
- Actix actors
- Raw Tokio channels
- Elixir/OTP

Include benchmark results and use case recommendations.

**Deliverables:**
- Complete rustdoc coverage
- 5+ working examples
- Migration guide
- Comparison documentation
- Blog post announcing v0.3

## Project Structure

**Issue:** [Align module layout with v0.3 structure](https://github.com/H1ghBre4k3r/notizia/issues/19)

Proposed module organization for v0.3:

```
notizia/
├── src/
│   ├── lib.rs
│   ├── core/
│   │   ├── mailbox.rs
│   │   ├── task.rs
│   │   ├── errors.rs
│   │   └── lifecycle.rs
│   ├── messaging/
│   │   ├── call_cast.rs
│   │   └── selective_recv.rs
│   ├── registry/
│   │   ├── registry.rs
│   │   └── macros.rs
│   ├── supervision/
│   │   ├── supervisor.rs
│   │   ├── monitor.rs
│   │   └── tree.rs
│   ├── observability/
│   │   ├── metrics.rs
│   │   └── tracing.rs
│   └── macros.rs
│
notizia_gen/
└── src/
    ├── lib.rs
    ├── task.rs
    ├── supervisor.rs
    └── codegen.rs
```

## Breaking Changes in v0.3

Since the project is pre-1.0, we accept breaking changes for better long-term ergonomics:

1. `recv!` returns `Result<T, RecvError>` instead of `T`
2. `send!` returns `Result<(), SendError<T>>` instead of `Result<(), SendError<T>>`
3. `Runnable::start()` signature changes from `impl Future` to `async fn`
4. Minimum Rust version may increase for async trait support

The migration path is straightforward: add `?` operators and handle errors appropriately.

## Open Questions

**Tracking issue:** [Resolve v0.3 open design questions](https://github.com/H1ghBre4k3r/notizia/issues/18)

### Type Erasure in Registry

**Decision needed:** Generic registry vs type-erased registry

- **Generic**: `Registry<Msg>` provides type safety but requires separate registries per message type
- **Type-erased**: `Registry<dyn Any>` allows a global registry but requires runtime downcasting
- **Hybrid**: Generic API with internal type erasure

Current recommendation is the hybrid approach, but this needs validation with real-world use cases.

### Selective Receive Performance

Buffering messages in `VecDeque` adds overhead. Should this be:
- Always enabled (simpler API)
- Opt-in via `#[Task(Msg, selective = true)]` (current recommendation)
- Separate `SelectiveTask` trait

Performance benchmarks will inform the final decision.

### Supervision Escalation

What happens when a top-level supervisor exceeds restart limits?

Options:
1. Panic the entire application (fail-fast)
2. Log error and stop supervision (graceful degradation)
3. Invoke a global panic handler hook (configurable)

Current recommendation is option 3 for flexibility.

## Success Criteria

Version 0.3 is considered complete when:

- All Elixir/OTP patterns have equivalent Notizia APIs
- Zero `unwrap()` calls in public APIs
- Test coverage exceeds 90%
- At least 10 examples demonstrate real-world patterns
- Performance within 10% of raw Tokio channels (measured via criterion benchmarks)
- Complete rustdoc coverage
- Migration guide published

## Future Directions (Post-0.3)

Potential features for v0.4 and beyond:

### Bounded Channels

Support for backpressure via bounded mailboxes:

```rust
#[Task(Msg, capacity = 100)]
struct Worker;
```

### Distributed Messaging

Bridge to external message brokers:

```rust
#[Task(Msg, transport = "nats://localhost:4222")]
struct RemoteWorker;
```

### Hot Code Reloading

Limited dynamic dispatch for behavior updates without restarting tasks. This is challenging in Rust but worth exploring for long-running services.

### Application Framework

Higher-level abstractions for common patterns:

```rust
Application::new()
    .worker_pool::<HttpHandler>(size = 10)
    .supervisor(strategy = OneForOne)
    .start()?;
```

### Performance Optimizations

- Zero-copy message passing for large payloads
- Lock-free mailbox implementations
- SIMD-based message filtering for selective receive

## Contributing

This roadmap is a living document. Feedback, suggestions, and contributions are welcome. Please open an issue or pull request to discuss changes to the roadmap.

For implementation priorities, see the GitHub project board and milestone tracking.

## Timeline Summary

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Foundation | 2 weeks | Error handling, lifecycle hooks |
| GenServer | 2 weeks | Call/cast, registry |
| Linking | 2 weeks | Links, monitors |
| Selective Receive | 1 week | Pattern matching mailbox |
| Supervision | 3 weeks | Supervisor trait, strategies, trees |
| Observability | 1 week | Metrics, tracing |
| Documentation | 1 week | Rustdoc, examples, guides |
| **Total** | **12 weeks** | Production-ready v0.3.0 |

Development will be tracked via GitHub milestones and issues. Community input is encouraged throughout the process.
