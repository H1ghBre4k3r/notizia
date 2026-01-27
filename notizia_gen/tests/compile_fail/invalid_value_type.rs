use notizia_gen::Task;

// Test string literal instead of type - should fail with "Expected a type"
#[derive(Task)]
#[task(message = "StringLiteral")]
struct MyTask;

fn main() {}
