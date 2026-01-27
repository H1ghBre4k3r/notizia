use notizia_gen::Task;
enum TaskMessage {
    Start,
    Process(u32),
    Stop,
}
#[automatically_derived]
impl ::core::clone::Clone for TaskMessage {
    #[inline]
    fn clone(&self) -> TaskMessage {
        match self {
            TaskMessage::Start => TaskMessage::Start,
            TaskMessage::Process(__self_0) => {
                TaskMessage::Process(::core::clone::Clone::clone(__self_0))
            }
            TaskMessage::Stop => TaskMessage::Stop,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for TaskMessage {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            TaskMessage::Start => ::core::fmt::Formatter::write_str(f, "Start"),
            TaskMessage::Process(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Process",
                    &__self_0,
                )
            }
            TaskMessage::Stop => ::core::fmt::Formatter::write_str(f, "Stop"),
        }
    }
}
#[task(message = TaskMessage)]
struct WorkerTask {
    id: usize,
    name: String,
    config: Config,
}
impl notizia::Task<TaskMessage> for WorkerTask {
    fn __setup(
        &self,
        receiver: notizia::tokio::sync::mpsc::UnboundedReceiver<TaskMessage>,
    ) -> impl std::future::Future<Output = ()> + Send {
        async move {
            let mb = self.mailbox();
            mb.set_receiver(receiver).await;
            self.start().await
        }
    }
    fn mailbox(&self) -> notizia::Mailbox<TaskMessage> {
        __WorkerTask_gen::WorkerTaskState.get().mailbox
    }
    fn run(self) -> notizia::TaskHandle<TaskMessage> {
        let (sender, receiver) = notizia::tokio::sync::mpsc::unbounded_channel::<
            TaskMessage,
        >();
        let task = __WorkerTask_gen::WorkerTaskState
            .scope(
                notizia::TaskState {
                    mailbox: notizia::Mailbox::new(),
                    sender: sender.clone(),
                },
                async move {
                    let handle = self.__setup(receiver);
                    handle.await
                },
            );
        let handle = notizia::tokio::spawn(task);
        notizia::TaskHandle::new(sender, handle)
    }
    fn this(&self) -> notizia::TaskRef<TaskMessage> {
        notizia::TaskRef::new(__WorkerTask_gen::WorkerTaskState.get().sender)
    }
}
mod __WorkerTask_gen {
    use super::*;
}
struct Config {
    max_retries: u32,
}
#[automatically_derived]
impl ::core::clone::Clone for Config {
    #[inline]
    fn clone(&self) -> Config {
        Config {
            max_retries: ::core::clone::Clone::clone(&self.max_retries),
        }
    }
}
fn main() {}
