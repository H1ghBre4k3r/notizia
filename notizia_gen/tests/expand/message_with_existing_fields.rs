use notizia_gen::message;

#[message]
#[derive(Debug)]
enum EchoMsg {
    #[request(reply = u32)]
    Echo {
        id: u32,
    },

    Stop,
}

fn main() {}
