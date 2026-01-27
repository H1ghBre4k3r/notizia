use notizia_gen::Task;
struct PingMessage;
#[automatically_derived]
impl ::core::clone::Clone for PingMessage {
    #[inline]
    fn clone(&self) -> PingMessage {
        PingMessage
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for PingMessage {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "PingMessage")
    }
}
#[task(message = PingMessage)]
struct PingTask;
impl notizia::Task<PingMessage> for PingTask {
    fn __setup(
        &self,
        receiver: notizia::tokio::sync::mpsc::UnboundedReceiver<PingMessage>,
    ) -> impl std::future::Future<Output = notizia::TerminateReason> + Send {
        async move {
            let mb = self.mailbox();
            mb.set_receiver(receiver).await;
            let start_result = notizia::futures::FutureExt::catch_unwind(
                    std::panic::AssertUnwindSafe(self.start()),
                )
                .await;
            let reason = match start_result {
                Ok(()) => notizia::TerminateReason::Normal,
                Err(panic_payload) => {
                    let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    notizia::TerminateReason::Panic(msg)
                }
            };
            let terminate_result = notizia::futures::FutureExt::catch_unwind(
                    std::panic::AssertUnwindSafe(self.terminate(reason.clone())),
                )
                .await;
            if let Err(terminate_panic) = terminate_result {
                let msg = if let Some(s) = terminate_panic.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = terminate_panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                {
                    ::std::io::_eprint(
                        format_args!("Warning: terminate() hook panicked: {0}\n", msg),
                    );
                };
            }
            reason
        }
    }
    fn mailbox(&self) -> notizia::Mailbox<PingMessage> {
        __PingTask_gen::PingTaskState.get().mailbox
    }
    fn run(self) -> notizia::TaskHandle<PingMessage> {
        let (sender, receiver) = notizia::tokio::sync::mpsc::unbounded_channel::<
            PingMessage,
        >();
        let task = __PingTask_gen::PingTaskState
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
    fn this(&self) -> notizia::TaskRef<PingMessage> {
        notizia::TaskRef::new(__PingTask_gen::PingTaskState.get().sender)
    }
}
mod __PingTask_gen {
    use super::*;
}
fn main() {}
