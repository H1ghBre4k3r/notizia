use notizia_gen::message;

#[message]
enum TestMsg {
    #[request]
    GetValue,
}

fn main() {}
