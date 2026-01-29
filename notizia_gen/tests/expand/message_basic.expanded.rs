use notizia_gen::message;
enum CounterMsg {
    GetCount { reply_to: ::notizia::tokio::sync::oneshot::Sender<u32> },
    Increment,
    Decrement,
}
#[automatically_derived]
impl ::core::fmt::Debug for CounterMsg {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            CounterMsg::GetCount { reply_to: __self_0 } => {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "GetCount",
                    "reply_to",
                    &__self_0,
                )
            }
            CounterMsg::Increment => ::core::fmt::Formatter::write_str(f, "Increment"),
            CounterMsg::Decrement => ::core::fmt::Formatter::write_str(f, "Decrement"),
        }
    }
}
fn main() {}
