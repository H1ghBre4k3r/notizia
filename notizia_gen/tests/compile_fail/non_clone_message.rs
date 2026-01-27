use notizia_gen::Task;

/// This should fail because the message type doesn't implement Clone
#[derive(Debug)]
struct NonCloneMessage {
    data: String,
}

#[derive(Task)]
#[task(message = NonCloneMessage)]
struct MyTask;

fn main() {}
