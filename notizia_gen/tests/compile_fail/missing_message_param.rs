use notizia_gen::Task;

#[derive(Clone, Debug)]
struct Message;

// Test wrong parameter name - should fail with "Expected 'message' parameter"
#[derive(Task)]
#[task(msg = Message)]
struct MyTask;

fn main() {}
