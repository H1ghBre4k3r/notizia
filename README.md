# Notizia

Notizia is a lightweight, type-safe message passing system built on top of Tokio. It provides an ergonomic abstraction for spawning and communicating with asynchronous tasks using an actor-like model.

The goal is to reduce the boilerplate associated with setting up unbounded channels, managing task handles, and synchronizing state in async Rust applications.

## Workspace Structure

This project is organized as a Cargo workspace with two members:

- **notizia**: The primary library containing the runtime primitives, traits, and interaction macros.
- **notizia_gen**: A procedural macro crate that generates the necessary glue code for message handling.

## Getting Started

To use Notizia, you define a struct to hold your state and an enum to represent the messages it can handle. The `#[Task]` attribute macro (re-exported from `notizia_gen`) implements the `Task` trait and handles the implementation details, allowing you to focus on the `start` logic.

See the `notizia` crate documentation for specific usage examples.
