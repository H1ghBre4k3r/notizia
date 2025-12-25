#[cfg(not(feature = "tokio"))]
mod std_impl;
#[cfg(not(feature = "tokio"))]
pub use self::std_impl::*;

#[cfg(feature = "tokio")]
pub use tokio;
#[cfg(feature = "tokio")]
mod tokio_impl;
#[cfg(feature = "tokio")]
pub use self::tokio_impl::*;
