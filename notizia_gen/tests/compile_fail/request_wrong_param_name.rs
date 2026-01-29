use notizia_gen::message;

#[message]
enum TestMsg {
    #[request(response = u32)]
    GetValue,
}

fn main() {}
