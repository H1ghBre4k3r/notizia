use notizia_gen::Task;
struct CounterMsg(u32);
#[automatically_derived]
impl ::core::clone::Clone for CounterMsg {
    #[inline]
    fn clone(&self) -> CounterMsg {
        CounterMsg(::core::clone::Clone::clone(&self.0))
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for CounterMsg {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_tuple_field1_finish(f, "CounterMsg", &&self.0)
    }
}
#[task(message = CounterMsg)]
struct CounterTask(usize, String);
impl notizia::Task<CounterMsg> for CounterTask {
    fn __setup(
        &self,
        receiver: notizia::tokio::sync::mpsc::UnboundedReceiver<CounterMsg>,
    ) -> impl std::future::Future<Output = ()> + Send {
        async move {
            let mb = self.mailbox();
            mb.set_receiver(receiver).await;
            self.start().await
        }
    }
    fn mailbox(&self) -> notizia::Mailbox<CounterMsg> {
        __CounterTask_gen::CounterTaskState.get().mailbox
    }
    fn run(self) -> notizia::TaskHandle<CounterMsg> {
        let (sender, receiver) = notizia::tokio::sync::mpsc::unbounded_channel::<
            CounterMsg,
        >();
        let task = __CounterTask_gen::CounterTaskState
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
    fn this(&self) -> notizia::TaskRef<CounterMsg> {
        notizia::TaskRef::new(__CounterTask_gen::CounterTaskState.get().sender)
    }
}
mod __CounterTask_gen {
    use super::*;
}
fn main() {}
