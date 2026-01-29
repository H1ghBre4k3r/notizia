use notizia_gen::message;
enum EchoMsg {
    Echo { id: u32, reply_to: ::notizia::tokio::sync::oneshot::Sender<u32> },
    Stop,
}
#[automatically_derived]
impl ::core::fmt::Debug for EchoMsg {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            EchoMsg::Echo { id: __self_0, reply_to: __self_1 } => {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "Echo",
                    "id",
                    __self_0,
                    "reply_to",
                    &__self_1,
                )
            }
            EchoMsg::Stop => ::core::fmt::Formatter::write_str(f, "Stop"),
        }
    }
}
fn main() {}
