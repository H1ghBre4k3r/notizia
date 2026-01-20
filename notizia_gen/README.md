# Notizia Gen

This crate defines the procedural macros used by `notizia`.

It exports the `#[Proc]` attribute, which is responsible for analyzing your struct and message enum to generate:

1. The `Task` trait implementation.
2. The `Mailbox` management logic.
3. The setup code that bridges the Tokio `mpsc` channel with your `Runnable` implementation.

You generally should not use this crate directly. Instead, depend on `notizia`, which re-exports functionality from this crate.
