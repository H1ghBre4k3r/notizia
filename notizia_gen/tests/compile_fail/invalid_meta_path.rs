use notizia_gen::Task;

#[derive(Clone, Debug)]
struct Message;

// Test #[task] without parameters - should fail with "requires parameters"
#[derive(Task)]
#[task]
struct MyTask;

fn main() {}
