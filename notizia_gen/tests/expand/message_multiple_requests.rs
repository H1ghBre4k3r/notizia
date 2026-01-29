use notizia_gen::message;

#[derive(Debug, Clone)]
struct CounterStats {
    count: u32,
    operations: u64,
}

#[message]
#[derive(Debug)]
enum CounterMsg {
    #[request(reply = u32)]
    GetCount,

    #[request(reply = CounterStats)]
    GetStats,

    Increment,
    Decrement,
    Add(u32),
}

fn main() {}
