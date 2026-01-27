// Test macro expansion for basic task with panic catching in __setup
use notizia_gen::Task;

#[derive(Clone, Debug)]
struct Message;

#[derive(Task)]
#[task(message = Message)]
struct BasicLifecycleTask {
    id: usize,
}

fn main() {}
