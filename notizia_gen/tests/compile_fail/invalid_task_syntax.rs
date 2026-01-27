use notizia_gen::Task;

#[derive(Clone, Debug)]
struct Message;

// Missing the #[task(message = T)] attribute - should fail
#[derive(Task)]
struct MyTask;

fn main() {}
