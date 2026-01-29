use notizia_gen::message;

#[message]
enum TestMsg {
    #[request(reply = u32)]
    GetValue(String),
}

fn main() {}
