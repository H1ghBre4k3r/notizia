use quote::{format_ident, quote};
use syn::{ItemEnum, ItemStruct, parse_macro_input};

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

        impl Proc<#item> for #name {
            async fn __setup(&self, receiver: mp::tokio::sync::mpsc::UnboundedReceiver<#item>) {
                let mb = self.mailbox();

                mb.set_receiver(receiver);

                self.start().await
            }

            fn mailbox(&self) -> mp::Mailbox<#item> {
                #mod_name::#mailbox.get()
            }

            fn run(self) -> mp::TaskHandle<#item, impl Future<Output = ()>> {
                let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<#item>();

                let handle = #mod_name::#mailbox.scope(mp::Mailbox::new(), async move {
                    let handle = self.__setup(receiver);
                    handle.await
                });

                mp::TaskHandle::new(sender, handle)
            }
        }


        mod #mod_name{
            use super::*;

            tokio::task_local! {
                pub static #mailbox: mp::Mailbox<#item>;
            }
        }
    };

    generated.into()
}
