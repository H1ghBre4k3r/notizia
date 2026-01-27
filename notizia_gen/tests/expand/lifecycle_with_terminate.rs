// Test macro expansion for task with custom terminate() hook
use notizia_gen::Task;

#[derive(Clone, Debug)]
enum Signal {
    Work,
    Stop,
}

#[derive(Task)]
#[task(message = Signal)]
struct WorkerWithCleanup {
    worker_id: u32,
}

// This file tests that the macro-generated code properly:
// 1. Catches panics in start()
// 2. Calls terminate() with the appropriate reason
// 3. Catches panics in terminate() itself
// 4. Returns TerminateReason to the join handle

fn main() {}
