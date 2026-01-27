use notizia_gen::Task;

#[derive(Clone, Debug)]
enum GenericMessage<T> {
    Data(T),
    Done,
}

#[derive(Task)]
#[task(message = GenericMessage<String>)]
struct ProcessorTask;

fn main() {}
