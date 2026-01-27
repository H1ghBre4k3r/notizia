//! Task lifecycle types.
//!
//! This module contains types related to task lifecycle management,
//! including graceful shutdown and termination handling.

use std::fmt;

/// Reason why a task's terminate() hook is being called.
///
/// This is passed to the task's [`Runnable::terminate`] method when
/// the task is shutting down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminateReason {
    /// Task completed normally (start() returned without panic)
    Normal,
    /// Task panicked during execution
    Panic(String),
}

impl fmt::Display for TerminateReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminateReason::Normal => write!(f, "normal termination"),
            TerminateReason::Panic(msg) => write!(f, "panicked: {}", msg),
        }
    }
}

/// Errors that can occur during graceful shutdown.
#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {
    /// The terminate() hook exceeded the timeout limit
    #[error("shutdown timeout exceeded")]
    Timeout,
    /// Unexpected join error
    #[error("task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

/// Result type for shutdown operations.
pub type ShutdownResult = Result<TerminateReason, ShutdownError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminate_reason_implements_debug_clone() {
        let reason = TerminateReason::Normal;
        assert_eq!(format!("{:?}", reason), "Normal");
        let _ = reason.clone();
    }

    #[test]
    fn shutdown_error_implements_std_error() {
        fn assert_is_error<E: std::error::Error + 'static>() {}
        assert_is_error::<ShutdownError>();
    }

    #[test]
    fn error_display_messages_are_user_friendly() {
        assert_eq!(
            format!("{}", ShutdownError::Timeout),
            "shutdown timeout exceeded"
        );
    }
}
