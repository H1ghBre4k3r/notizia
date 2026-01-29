use notizia_gen::message;
struct CounterStats {
    count: u32,
    operations: u64,
}
#[automatically_derived]
impl ::core::fmt::Debug for CounterStats {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "CounterStats",
            "count",
            &self.count,
            "operations",
            &&self.operations,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for CounterStats {
    #[inline]
    fn clone(&self) -> CounterStats {
        CounterStats {
            count: ::core::clone::Clone::clone(&self.count),
            operations: ::core::clone::Clone::clone(&self.operations),
        }
    }
}
enum CounterMsg {
    GetCount { reply_to: ::notizia::tokio::sync::oneshot::Sender<u32> },
    GetStats { reply_to: ::notizia::tokio::sync::oneshot::Sender<CounterStats> },
    Increment,
    Decrement,
    Add(u32),
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
            CounterMsg::GetStats { reply_to: __self_0 } => {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "GetStats",
                    "reply_to",
                    &__self_0,
                )
            }
            CounterMsg::Increment => ::core::fmt::Formatter::write_str(f, "Increment"),
            CounterMsg::Decrement => ::core::fmt::Formatter::write_str(f, "Decrement"),
            CounterMsg::Add(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Add", &__self_0)
            }
        }
    }
}
fn main() {}
