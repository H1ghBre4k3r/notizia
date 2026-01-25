// This file tests that missing #[Task] attribute causes compile errors
// This is a compile-fail test

use notizia::Runnable;

// This struct does NOT have the #[Task] attribute
struct NoAttributeTask;

#[derive(Clone, Debug)]
struct Msg;

impl Runnable<Msg> for NoAttributeTask {
    async fn start(&self) {}
}

// This should fail to compile because NoAttributeTask does not implement
// the Task trait (it's missing the #[Task] macro)
fn main() {
    let task = NoAttributeTask;
    // This line should fail because run() is not defined
    let _ = task.run();
}
