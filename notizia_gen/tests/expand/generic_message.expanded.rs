use notizia_gen::Task;
enum GenericMessage<T> {
    Data(T),
    Done,
}
#[automatically_derived]
impl<T: ::core::clone::Clone> ::core::clone::Clone for GenericMessage<T> {
    #[inline]
    fn clone(&self) -> GenericMessage<T> {
        match self {
            GenericMessage::Data(__self_0) => {
                GenericMessage::Data(::core::clone::Clone::clone(__self_0))
            }
            GenericMessage::Done => GenericMessage::Done,
        }
    }
}
#[automatically_derived]
impl<T: ::core::fmt::Debug> ::core::fmt::Debug for GenericMessage<T> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            GenericMessage::Data(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Data", &__self_0)
            }
            GenericMessage::Done => ::core::fmt::Formatter::write_str(f, "Done"),
        }
    }
}
#[task(message = GenericMessage<String>)]
struct ProcessorTask;
fn main() {}
