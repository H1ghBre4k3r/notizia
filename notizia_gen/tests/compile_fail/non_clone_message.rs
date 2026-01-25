// This file tests that non-Clone messages cause compile errors
// This is a compile-fail test

use notizia_gen::Task;

#[Task(NonCloneMsg)]
struct TestTask;

#[derive(Debug)]
struct NonCloneMsg; // This does not implement Clone

// This should fail to compile because the message type needs to be Clone
// for use with the Task macro (it's used in send operations)
fn main() {
    let _ = TestTask;
}
