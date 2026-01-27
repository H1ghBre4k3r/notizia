use notizia_gen::Task;

#[derive(Clone, Debug)]
struct CounterMsg(u32);

#[derive(Task)]
#[task(message = CounterMsg)]
struct CounterTask(usize, String);

fn main() {}
