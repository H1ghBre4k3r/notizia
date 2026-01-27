pub use tokio::sync::mpsc::error::SendError;

#[derive(Debug, thiserror::Error)]
pub enum RecvError {
    #[error("channel closed")]
    Closed,
    #[error("channel poisoned")]
    Poisoned,
    #[error("receive timeout")]
    Timeout,
}

pub type RecvResult<T> = Result<T, RecvError>;

pub type SendResult<T> = Result<(), SendError<T>>;

#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("call timeout")]
    Timeout,
    #[error("reply channel closed")]
    ChannelClosed,
    #[error("send failed")]
    SendError,
}

pub type CallResult<T> = Result<T, CallError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recv_error_implements_std_error() {
        fn assert_is_error<E: std::error::Error + 'static>() {}
        assert_is_error::<RecvError>();
    }

    #[test]
    fn send_error_implements_std_error() {
        fn assert_is_error<E: std::error::Error + 'static>() {}
        assert_is_error::<SendError<i32>>();
    }

    #[test]
    fn error_display_messages_are_user_friendly() {
        assert_eq!(format!("{}", RecvError::Closed), "channel closed");
        assert_eq!(format!("{}", RecvError::Poisoned), "channel poisoned");
        assert_eq!(format!("{}", RecvError::Timeout), "receive timeout");

        assert_eq!(format!("{}", SendError(42)), "channel closed");

        assert_eq!(format!("{}", CallError::Timeout), "call timeout");
        assert_eq!(
            format!("{}", CallError::ChannelClosed),
            "reply channel closed"
        );
        assert_eq!(format!("{}", CallError::SendError), "send failed");
    }

    #[test]
    fn recv_error_debug_formatting() {
        assert_eq!(format!("{:?}", RecvError::Closed), "Closed");
        assert_eq!(format!("{:?}", RecvError::Poisoned), "Poisoned");
        assert_eq!(format!("{:?}", RecvError::Timeout), "Timeout");
    }

    #[test]
    fn call_error_implements_std_error() {
        fn assert_is_error<E: std::error::Error + 'static>() {}
        assert_is_error::<CallError>();
    }
}
