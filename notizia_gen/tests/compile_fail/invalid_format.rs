use notizia_gen::Task;

#[derive(Clone, Debug)]
struct Message;

// Test malformed attribute syntax - should fail during parsing
#[derive(Task)]
#[task(message)]
struct MyTask;

fn main() {}
