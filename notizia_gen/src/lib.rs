use quote::{format_ident, quote};
use syn::{ItemEnum, ItemStruct, parse_macro_input};

use proc_macro::TokenStream;

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn Task(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(attrs as syn::Path);
    let ast: syn::Item =
        syn::parse(input.clone()).expect("#[token] currently only works for items!");

    let name = match &ast {
        syn::Item::Enum(ItemEnum { ident, .. }) => ident,
        syn::Item::Struct(ItemStruct { ident, .. }) => ident,
        _ => todo!(),
    };

    let mod_name = format_ident!("__{name}_gen");

    let task_state = format_ident!("{name}State");

    let generated = quote! {
        #ast

        impl notizia::Task<#item> for #name {
            fn __setup(&self, receiver: notizia::tokio::sync::mpsc::UnboundedReceiver<#item>) -> impl std::future::Future<Output = ()> + Send {
                async move {
                    let mb = self.mailbox();

                    mb.set_receiver(receiver).await;

                    self.start().await
                }
            }

            fn mailbox(&self) -> notizia::Mailbox<#item> {
                #mod_name::#task_state.get().mailbox
            }

            fn run(self) -> notizia::TaskHandle<#item> {
                let (sender, receiver) = notizia::tokio::sync::mpsc::unbounded_channel::<#item>();

                let task = #mod_name::#task_state.scope(notizia::TaskState {
                    mailbox: notizia::Mailbox::new(),
                    sender: sender.clone(),
                }, async move {
                    let handle = self.__setup(receiver);
                    handle.await
                });

                let handle = notizia::tokio::spawn(task);

                notizia::TaskHandle::new(sender, handle)
            }

            fn this(&self) -> notizia::TaskRef<#item> {
                notizia::TaskRef::new(#mod_name::#task_state.get().sender)
            }
        }


        mod #mod_name{
            use super::*;

            tokio::task_local! {
                pub static #task_state: notizia::TaskState<#item>;
            }
        }
    };

    generated.into()
}
