use notizia_gen::message;

#[message]
#[derive(Debug, Clone)]
enum SimpleMsg {
    Increment,
    Decrement,
    Stop,
}

fn main() {}
