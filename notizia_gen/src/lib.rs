use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemEnum, ItemStruct};

use proc_macro::TokenStream;

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn Proc(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(attrs as syn::Path);
    let ast: syn::Item =
        syn::parse(input.clone()).expect("#[token] currently only works for items!");

    let name = match &ast {
        syn::Item::Enum(ItemEnum { ident, .. }) => ident,
        syn::Item::Struct(ItemStruct { ident, .. }) => ident,
        _ => todo!(),
    };

    let mailbox = format_ident!("{name}Mailbox");

    let mod_name = format_ident!("__{name}_gen");

    let generated = quote! {
        #ast

        impl notizia::Proc<#item> for #name {
            fn __setup(&self, receiver: notizia::tokio::sync::mpsc::UnboundedReceiver<#item>) -> impl std::future::Future<Output = ()> + Send {
                async move {
                    let mb = self.mailbox();

                    mb.set_receiver(receiver).await;

                    self.start().await
                }
            }

            fn mailbox(&self) -> notizia::Mailbox<#item> {
                #mod_name::#mailbox.get()
            }

            fn run(self) -> notizia::TaskHandle<#item, impl Future<Output = ()>> {
                let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<#item>();

                let handle = #mod_name::#mailbox.scope(notizia::Mailbox::new(), async move {
                    let handle = self.__setup(receiver);
                    handle.await
                });

                notizia::TaskHandle::new(sender, handle)
            }
        }


        mod #mod_name{
            use super::*;

            tokio::task_local! {
                pub static #mailbox: notizia::Mailbox<#item>;
            }
        }
    };

    generated.into()
}
