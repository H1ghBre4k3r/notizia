//! Integration tests for the #[message] macro.
//!
//! This test suite validates the #[message] attribute macro that automatically
//! injects reply_to fields for request variants.

use notizia::message;
use tokio::sync::oneshot;

#[test]
fn message_macro_injects_reply_to_field() {
    #[message]
    #[derive(Debug)]
    #[allow(dead_code)]
    enum TestMsg {
        #[request(reply = u32)]
        GetValue,

        Increment,
    }

    // This would not compile if reply_to wasn't injected
    let (tx, _rx) = oneshot::channel();
    let _msg = TestMsg::GetValue { reply_to: tx };
}

#[test]
fn message_macro_preserves_cast_variants() {
    #[message]
    #[derive(Debug)]
    #[allow(dead_code)]
    enum TestMsg {
        #[request(reply = String)]
        GetStatus,

        Increment,
        Decrement,
        Stop,
    }

    // Cast variants should work as before
    let _msg1 = TestMsg::Increment;
    let _msg2 = TestMsg::Decrement;
    let _msg3 = TestMsg::Stop;
}

#[test]
fn message_macro_works_with_existing_fields() {
    #[message]
    #[derive(Debug)]
    #[allow(dead_code)]
    enum TestMsg {
        #[request(reply = u32)]
        Echo {
            id: u32,
        },

        Stop,
    }

    // Should inject reply_to alongside existing fields
    let (tx, _rx) = oneshot::channel();
    let _msg = TestMsg::Echo {
        id: 42,
        reply_to: tx,
    };
}

#[test]
fn message_macro_works_with_multiple_requests() {
    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct Stats {
        count: u32,
    }

    #[message]
    #[derive(Debug)]
    #[allow(dead_code)]
    enum TestMsg {
        #[request(reply = u32)]
        GetCount,

        #[request(reply = Stats)]
        GetStats,

        #[request(reply = String)]
        GetStatus,

        Increment,
    }

    let (tx1, _rx1) = oneshot::channel();
    let _msg1 = TestMsg::GetCount { reply_to: tx1 };

    let (tx2, _rx2) = oneshot::channel();
    let _msg2 = TestMsg::GetStats { reply_to: tx2 };

    let (tx3, _rx3) = oneshot::channel();
    let _msg3 = TestMsg::GetStatus { reply_to: tx3 };
}

#[test]
fn message_macro_works_with_tuple_cast_variants() {
    #[message]
    #[derive(Debug)]
    #[allow(dead_code)]
    enum TestMsg {
        #[request(reply = u32)]
        GetValue,

        Add(u32),
        Process(String),
    }

    let _msg1 = TestMsg::Add(10);
    let _msg2 = TestMsg::Process("test".to_string());
}

#[test]
fn issue5_exact_syntax_works() {
    // This is the exact syntax from issue #5
    // Note: Can't derive Clone because oneshot::Sender doesn't implement Clone
    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct StatusReply {
        status: String,
    }

    #[message]
    #[derive(Debug)]
    #[allow(dead_code)]
    enum Msg {
        #[request(reply = StatusReply)]
        GetStatus,

        Increment,
    }

    // Verify it compiles and works
    let (tx, _rx) = oneshot::channel();
    let _msg = Msg::GetStatus { reply_to: tx };
    let _msg2 = Msg::Increment;
}
