use notizia::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::time::{Duration, sleep};

// Test with struct message type
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct StructMsg {
    value: u32,
    text: String,
}

#[derive(Task)]
#[task(message = StructMsg)]
struct StructTask {
    received: Arc<AtomicU32>,
}

impl Runnable<StructMsg> for StructTask {
    async fn start(&self) {
        while let Ok(msg) = recv!(self) {
            self.received.fetch_add(msg.value, Ordering::SeqCst);
        }
    }
}

#[tokio::test]
async fn derive_macro_works_with_struct_messages() {
    let received = Arc::new(AtomicU32::new(0));
    let task = StructTask {
        received: received.clone(),
    };
    let handle = spawn!(task);

    handle
        .send(StructMsg {
            value: 10,
            text: "test".to_string(),
        })
        .unwrap();

    sleep(Duration::from_millis(10)).await;

    assert_eq!(received.load(Ordering::SeqCst), 10);

    drop(handle);
}

// Test with enum message type
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum EnumMsg {
    Increment,
    Decrement,
    Reset,
    Stop,
}

#[derive(Task)]
#[task(message = EnumMsg)]
struct EnumTask {
    value: Arc<AtomicU32>,
}

impl Runnable<EnumMsg> for EnumTask {
    async fn start(&self) {
        let mut count = 0u32;
        loop {
            match recv!(self) {
                Ok(EnumMsg::Increment) => count += 1,
                Ok(EnumMsg::Decrement) => count = count.saturating_sub(1),
                Ok(EnumMsg::Reset) => count = 0,
                Ok(EnumMsg::Stop) => {
                    self.value.store(count, Ordering::SeqCst);
                    break;
                }
                Err(_) => break,
            }
        }
    }
}

#[tokio::test]
async fn derive_macro_works_with_enum_messages() {
    let value = Arc::new(AtomicU32::new(0));
    let task = EnumTask {
        value: value.clone(),
    };
    let handle = spawn!(task);

    handle.send(EnumMsg::Increment).unwrap();
    handle.send(EnumMsg::Increment).unwrap();
    handle.send(EnumMsg::Increment).unwrap();
    handle.send(EnumMsg::Decrement).unwrap();
    handle.send(EnumMsg::Stop).unwrap();

    let _ = handle.join().await;

    assert_eq!(value.load(Ordering::SeqCst), 2);
}

// Test with unit struct
#[derive(Task)]
#[task(message = UnitMsg)]
struct UnitTask;

#[derive(Debug, Clone)]
struct UnitMsg;

impl Runnable<UnitMsg> for UnitTask {
    async fn start(&self) {
        // Just receive one message and terminate
        let _ = recv!(self);
    }
}

#[tokio::test]
async fn derive_macro_works_with_unit_struct() {
    let task = UnitTask;
    let handle = spawn!(task);

    handle.send(UnitMsg).unwrap();
    let _ = handle.join().await;
}

// Test with struct with multiple fields
#[derive(Task)]
#[task(message = MultiFieldMsg)]
struct MultiFieldTask {
    field1: u32,
    field2: String,
    field3: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
enum MultiFieldMsg {
    Check,
    Stop,
}

impl Runnable<MultiFieldMsg> for MultiFieldTask {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(MultiFieldMsg::Check) => {
                    if self.field1 == 42 && self.field2 == "test" {
                        self.field3.store(true, Ordering::SeqCst);
                    }
                }
                Ok(MultiFieldMsg::Stop) => break,
                Err(_) => break,
            }
        }
    }
}

#[tokio::test]
async fn derive_macro_works_with_multi_field_struct() {
    let flag = Arc::new(AtomicBool::new(false));
    let task = MultiFieldTask {
        field1: 42,
        field2: "test".to_string(),
        field3: flag.clone(),
    };
    let handle = spawn!(task);

    handle.send(MultiFieldMsg::Check).unwrap();

    sleep(Duration::from_millis(10)).await;

    assert!(flag.load(Ordering::SeqCst));

    handle.send(MultiFieldMsg::Stop).unwrap();
    let _ = handle.join().await;
}

// TODO: Test with generic message type
// Currently commented out because the derive macro parser doesn't support
// generic types in #[task(message = ...)] attribute yet.
//
// #[derive(Debug, Clone)]
// struct GenericMsg<T: Clone> {
//     data: T,
// }
//
// #[derive(Task)]
// #[task(message = GenericMsg<u32>)]
// struct ConcreteGenericTask {
//     sum: Arc<AtomicU32>,
// }
//
// impl Runnable<GenericMsg<u32>> for ConcreteGenericTask {
//     async fn start(&self) {
//         loop {
//             match recv!(self) {
//                 Ok(msg) => {
//                     self.sum.fetch_add(msg.data, Ordering::SeqCst);
//                 }
//                 Err(_) => break,
//             }
//         }
//     }
// }
//
// #[tokio::test]
// async fn derive_macro_works_with_generic_messages() {
//     let sum = Arc::new(AtomicU32::new(0));
//     let task = ConcreteGenericTask { sum: sum.clone() };
//     let handle = spawn!(task);
//
//     handle.send(GenericMsg { data: 5 }).unwrap();
//     handle.send(GenericMsg { data: 10 }).unwrap();
//     handle.send(GenericMsg { data: 15 }).unwrap();
//
//     sleep(Duration::from_millis(10)).await;
//
//     assert_eq!(sum.load(Ordering::SeqCst), 30);
//
//     drop(handle);
// }

// Test with nested enum
#[derive(Debug, Clone)]
enum OuterMsg {
    Inner(InnerMsg),
    Direct(u32),
    Stop,
}

#[derive(Debug, Clone)]
enum InnerMsg {
    Value(u32),
}

#[derive(Task)]
#[task(message = OuterMsg)]
struct NestedTask {
    count: Arc<AtomicU32>,
}

impl Runnable<OuterMsg> for NestedTask {
    async fn start(&self) {
        loop {
            match recv!(self) {
                Ok(OuterMsg::Inner(InnerMsg::Value(v))) => {
                    self.count.fetch_add(v, Ordering::SeqCst);
                }
                Ok(OuterMsg::Direct(v)) => {
                    self.count.fetch_add(v * 2, Ordering::SeqCst);
                }
                Ok(OuterMsg::Stop) => break,
                Err(_) => break,
            }
        }
    }
}

#[tokio::test]
async fn derive_macro_works_with_nested_enums() {
    let count = Arc::new(AtomicU32::new(0));
    let task = NestedTask {
        count: count.clone(),
    };
    let handle = spawn!(task);

    handle.send(OuterMsg::Inner(InnerMsg::Value(5))).unwrap();
    handle.send(OuterMsg::Direct(3)).unwrap(); // 3 * 2 = 6

    sleep(Duration::from_millis(10)).await;

    assert_eq!(count.load(Ordering::SeqCst), 11); // 5 + 6

    handle.send(OuterMsg::Stop).unwrap();
    let _ = handle.join().await;
}

// Test multiple tasks with different message types
#[tokio::test]
async fn multiple_task_types_can_coexist() {
    let struct_received = Arc::new(AtomicU32::new(0));
    let enum_value = Arc::new(AtomicU32::new(0));

    let struct_task = StructTask {
        received: struct_received.clone(),
    };
    let enum_task = EnumTask {
        value: enum_value.clone(),
    };

    let h1 = spawn!(struct_task);
    let h2 = spawn!(enum_task);

    h1.send(StructMsg {
        value: 100,
        text: "test".to_string(),
    })
    .unwrap();

    h2.send(EnumMsg::Increment).unwrap();
    h2.send(EnumMsg::Increment).unwrap();
    h2.send(EnumMsg::Stop).unwrap();

    let _ = h2.join().await;

    sleep(Duration::from_millis(10)).await;

    assert_eq!(struct_received.load(Ordering::SeqCst), 100);
    assert_eq!(enum_value.load(Ordering::SeqCst), 2);

    drop(h1);
}

// Test with tuple struct message
#[derive(Debug, Clone)]
struct TupleMsg(u32, String);

#[derive(Task)]
#[task(message = TupleMsg)]
struct TupleTask {
    sum: Arc<AtomicU32>,
}

impl Runnable<TupleMsg> for TupleTask {
    async fn start(&self) {
        while let Ok(TupleMsg(value, _text)) = recv!(self) {
            self.sum.fetch_add(value, Ordering::SeqCst);
        }
    }
}

#[tokio::test]
async fn derive_macro_works_with_tuple_struct() {
    let sum = Arc::new(AtomicU32::new(0));
    let task = TupleTask { sum: sum.clone() };
    let handle = spawn!(task);

    handle.send(TupleMsg(10, "first".to_string())).unwrap();
    handle.send(TupleMsg(20, "second".to_string())).unwrap();

    sleep(Duration::from_millis(10)).await;

    assert_eq!(sum.load(Ordering::SeqCst), 30);

    drop(handle);
}
