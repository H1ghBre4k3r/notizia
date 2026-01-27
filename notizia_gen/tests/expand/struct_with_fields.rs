use notizia_gen::Task;

#[derive(Clone, Debug)]
enum TaskMessage {
    Start,
    Process(u32),
    Stop,
}

#[derive(Task)]
#[task(message = TaskMessage)]
struct WorkerTask {
    id: usize,
    name: String,
    config: Config,
}

#[derive(Clone)]
struct Config {
    max_retries: u32,
}

fn main() {}
