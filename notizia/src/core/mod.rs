use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecvError {
    Closed,
    Poisoned,
    Timeout,
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecvError::Closed => write!(f, "Channel closed"),
            RecvError::Poisoned => write!(f, "Channel poisoned"),
            RecvError::Timeout => write!(f, "Receive timeout"),
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
            SendError::Disconnected(_) => write!(f, "Channel disconnected"),
            SendError::Full(_) => write!(f, "Channel full"),
        }
    }
}

impl<T: fmt::Debug> std::error::Error for SendError<T> {}
