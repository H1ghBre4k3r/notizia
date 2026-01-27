use notizia_gen::Task;

#[derive(Clone, Debug)]
struct PingMessage;

#[derive(Task)]
#[task(message = PingMessage)]
struct PingTask;

fn main() {}
