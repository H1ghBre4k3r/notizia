use std::fmt;

pub type RecvResult<T> = Result<T, RecvError>;
pub type SendResult<T> = Result<(), SendError<T>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecvError {
    Closed,
    Poisoned,
    Timeout,
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecvError::Closed => write!(f, "channel closed"),
            RecvError::Poisoned => write!(f, "channel poisoned"),
            RecvError::Timeout => write!(f, "receive timeout"),
        }
    }
}

impl std::error::Error for RecvError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendError<T> {
    Disconnected(T),
    Full(T),
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendError::Disconnected(_) => write!(f, "channel disconnected"),
            SendError::Full(_) => write!(f, "channel full"),
        }
    }
}

impl<T: std::fmt::Debug> std::error::Error for SendError<T> {}

impl<T> SendError<T> {
    pub fn into_inner(self) -> T {
        match self {
            SendError::Disconnected(msg) => msg,
            SendError::Full(msg) => msg,
        }
    }
}

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

        assert_eq!(
            format!("{}", SendError::<i32>::Disconnected(42)),
            "channel disconnected"
        );
        assert_eq!(format!("{}", SendError::<i32>::Full(42)), "channel full");
    }

    #[test]
    fn send_error_into_inner() {
        let err = SendError::Disconnected(42);
        assert_eq!(err.into_inner(), 42);

        let err = SendError::Full(100);
        assert_eq!(err.into_inner(), 100);
    }

    #[test]
    fn recv_error_variants_are_distinct() {
        assert_ne!(RecvError::Closed, RecvError::Poisoned);
        assert_ne!(RecvError::Closed, RecvError::Timeout);
        assert_ne!(RecvError::Poisoned, RecvError::Timeout);
    }

    #[test]
    fn send_error_variants_are_distinct() {
        assert_ne!(SendError::Disconnected(1), SendError::Full(1));
    }

    #[test]
    fn send_error_equality_considers_message() {
        assert_eq!(SendError::Disconnected(1), SendError::Disconnected(1));
        assert_ne!(SendError::Disconnected(1), SendError::Disconnected(2));
    }

    #[test]
    fn recv_error_debug_formatting() {
        assert_eq!(format!("{:?}", RecvError::Closed), "Closed");
        assert_eq!(format!("{:?}", RecvError::Poisoned), "Poisoned");
        assert_eq!(format!("{:?}", RecvError::Timeout), "Timeout");
    }

    #[test]
    fn send_error_debug_formatting() {
        assert_eq!(
            format!("{:?}", SendError::Disconnected(42)),
            "Disconnected(42)"
        );
        assert_eq!(format!("{:?}", SendError::Full(42)), "Full(42)");
    }
}
