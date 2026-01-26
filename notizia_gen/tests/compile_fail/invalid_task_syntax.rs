use notizia_gen::Task;

#[derive(Clone, Debug)]
struct Message;

// Missing the message type parameter - should fail
#[Task]
struct MyTask;

fn main() {}
