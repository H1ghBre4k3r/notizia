use notizia_gen::message;
enum SimpleMsg {
    Increment,
    Decrement,
    Stop,
}
#[automatically_derived]
impl ::core::fmt::Debug for SimpleMsg {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                SimpleMsg::Increment => "Increment",
                SimpleMsg::Decrement => "Decrement",
                SimpleMsg::Stop => "Stop",
            },
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for SimpleMsg {
    #[inline]
    fn clone(&self) -> SimpleMsg {
        match self {
            SimpleMsg::Increment => SimpleMsg::Increment,
            SimpleMsg::Decrement => SimpleMsg::Decrement,
            SimpleMsg::Stop => SimpleMsg::Stop,
        }
    }
}
fn main() {}
