use notizia_gen::Task;

#[derive(Clone, Debug)]
struct Message;

// Test #[task = "value"] format - should fail with "Invalid task attribute format"
#[derive(Task)]
#[task = "Message"]
struct MyTask;

fn main() {}
