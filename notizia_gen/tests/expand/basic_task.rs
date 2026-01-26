use notizia_gen::Task;

#[derive(Clone, Debug)]
struct PingMessage;

#[Task(PingMessage)]
struct PingTask;

fn main() {}
