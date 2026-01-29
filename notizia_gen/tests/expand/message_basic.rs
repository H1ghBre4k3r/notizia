use notizia_gen::message;

#[message]
#[derive(Debug)]
enum CounterMsg {
    #[request(reply = u32)]
    GetCount,

    Increment,
    Decrement,
}

fn main() {}
