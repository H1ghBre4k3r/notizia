//! Tests for the simplified call! macro syntax with #[message] macro.
//!
//! This test suite validates that the call! macro works with simple variant paths
//! when using the #[message] macro to define request variants.

use notizia::prelude::*;
use notizia::{call, cast, message};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

// =============================================================================
// Test 1: Simple call syntax with unit-like request variant
// =============================================================================

#[message]
#[derive(Debug)]
enum CounterMsg {
    #[request(reply = u32)]
    GetCount,

    Increment,
}

#[derive(Task)]
#[task(message = CounterMsg)]
struct Counter {
    count: Arc<AtomicU32>,
}

impl Runnable<CounterMsg> for Counter {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(CounterMsg::GetCount { reply_to }) => {
                    let count = self.count.load(Ordering::SeqCst);
                    let _ = reply_to.send(count);
                }
                Ok(CounterMsg::Increment) => {
                    self.count.fetch_add(1, Ordering::SeqCst);
                }
                Err(_) => break,
            }
        }
    }
}

#[tokio::test]
async fn simple_call_syntax_with_default_timeout() {
    let count = Arc::new(AtomicU32::new(0));
    let counter = Counter {
        count: count.clone(),
    };
    let handle = spawn!(counter);

    // Increment a few times
    cast!(handle, CounterMsg::Increment).expect("cast failed");
    cast!(handle, CounterMsg::Increment).expect("cast failed");

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // New simple syntax - no closure needed!
    let result = call!(handle, CounterMsg::GetCount).await;

    assert!(result.is_ok(), "Call should succeed");
    assert_eq!(result.unwrap(), 2, "Counter should be 2");
}

#[tokio::test]
async fn simple_call_syntax_with_custom_timeout() {
    let count = Arc::new(AtomicU32::new(10));
    let counter = Counter { count };
    let handle = spawn!(counter);

    // Simple syntax with custom timeout
    let result = call!(handle, CounterMsg::GetCount, timeout = 1000).await;

    assert!(result.is_ok(), "Call should succeed");
    assert_eq!(result.unwrap(), 10, "Counter should be 10");
}

// =============================================================================
// Test 2: Multiple request variants with simple syntax
// =============================================================================

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Stats {
    count: u32,
    operations: u32,
}

#[message]
#[derive(Debug)]
#[allow(dead_code)]
enum MultiMsg {
    #[request(reply = u32)]
    GetCount,

    #[request(reply = Stats)]
    GetStats,

    #[request(reply = String)]
    GetStatus,

    Increment,
}

#[derive(Task)]
#[task(message = MultiMsg)]
struct MultiCounter {
    count: Arc<AtomicU32>,
    ops: Arc<AtomicU32>,
}

impl Runnable<MultiMsg> for MultiCounter {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(MultiMsg::GetCount { reply_to }) => {
                    let _ = reply_to.send(self.count.load(Ordering::SeqCst));
                }
                Ok(MultiMsg::GetStats { reply_to }) => {
                    let stats = Stats {
                        count: self.count.load(Ordering::SeqCst),
                        operations: self.ops.load(Ordering::SeqCst),
                    };
                    let _ = reply_to.send(stats);
                }
                Ok(MultiMsg::GetStatus { reply_to }) => {
                    let _ = reply_to.send("Running".to_string());
                }
                Ok(MultiMsg::Increment) => {
                    self.count.fetch_add(1, Ordering::SeqCst);
                    self.ops.fetch_add(1, Ordering::SeqCst);
                }
                Err(_) => break,
            }
        }
    }
}

#[tokio::test]
async fn multiple_request_variants_with_simple_syntax() {
    let counter = MultiCounter {
        count: Arc::new(AtomicU32::new(5)),
        ops: Arc::new(AtomicU32::new(10)),
    };
    let handle = spawn!(counter);

    // Test multiple different request types with simple syntax
    let count = call!(handle, MultiMsg::GetCount).await.unwrap();
    assert_eq!(count, 5);

    let stats = call!(handle, MultiMsg::GetStats).await.unwrap();
    assert_eq!(stats.count, 5);
    assert_eq!(stats.operations, 10);

    let status = call!(handle, MultiMsg::GetStatus).await.unwrap();
    assert_eq!(status, "Running");
}

// =============================================================================
// Test 3: Variants with additional fields still use closure syntax
// =============================================================================

#[message]
#[derive(Debug)]
#[allow(dead_code)]
enum EchoMsg {
    #[request(reply = u32)]
    Echo {
        id: u32,
    },

    Stop,
}

#[derive(Task)]
#[task(message = EchoMsg)]
struct EchoServer;

impl Runnable<EchoMsg> for EchoServer {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(EchoMsg::Echo { id, reply_to }) => {
                    let _ = reply_to.send(id * 2);
                }
                Ok(EchoMsg::Stop) => break,
                Err(_) => break,
            }
        }
    }
}

#[tokio::test]
async fn variants_with_fields_use_closure_syntax() {
    let server = EchoServer;
    let handle = spawn!(server);

    // Variants with additional fields require closure syntax
    let result = call!(handle, |tx| EchoMsg::Echo {
        id: 42,
        reply_to: tx
    })
    .await;

    assert!(result.is_ok(), "Call should succeed");
    assert_eq!(result.unwrap(), 84, "Should return id * 2");
}

// =============================================================================
// Test 4: Backwards compatibility - old syntax still works
// =============================================================================

#[tokio::test]
async fn backwards_compatible_closure_syntax_still_works() {
    let count = Arc::new(AtomicU32::new(7));
    let counter = Counter { count };
    let handle = spawn!(counter);

    // Old closure syntax should still work
    let result = call!(handle, |tx| CounterMsg::GetCount { reply_to: tx }).await;

    assert!(result.is_ok(), "Call should succeed");
    assert_eq!(result.unwrap(), 7, "Counter should be 7");
}

#[tokio::test]
async fn backwards_compatible_closure_syntax_with_timeout() {
    let count = Arc::new(AtomicU32::new(15));
    let counter = Counter { count };
    let handle = spawn!(counter);

    // Old closure syntax with timeout should still work
    let result = call!(
        handle,
        |tx| CounterMsg::GetCount { reply_to: tx },
        timeout = 2000
    )
    .await;

    assert!(result.is_ok(), "Call should succeed");
    assert_eq!(result.unwrap(), 15, "Counter should be 15");
}

// =============================================================================
// Test 5: Both syntaxes work in the same code
// =============================================================================

#[tokio::test]
async fn can_mix_both_syntaxes() {
    let counter = MultiCounter {
        count: Arc::new(AtomicU32::new(3)),
        ops: Arc::new(AtomicU32::new(5)),
    };
    let handle = spawn!(counter);

    // Simple syntax for unit-like variants
    let count = call!(handle, MultiMsg::GetCount).await.unwrap();
    assert_eq!(count, 3);

    // Closure syntax also works
    let count2 = call!(handle, |tx| MultiMsg::GetCount { reply_to: tx })
        .await
        .unwrap();
    assert_eq!(count2, 3);

    // Both should give same result
    assert_eq!(count, count2);
}
