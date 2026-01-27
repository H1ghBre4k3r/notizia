use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, DeriveInput, Error, Meta, MetaNameValue, Result, Type};

/// Derive macro for implementing the Task trait.
///
/// This macro requires a `#[task(message = T)]` attribute to specify the message type.
///
/// # Example
///
/// ```rust,ignore
/// # TODO: Re-enable once derive macro hygiene is fixed
/// use notizia::prelude::*;
///
/// #[derive(Task)]
/// #[task(message = MyMessage)]
/// struct MyTask {
///     id: usize,
/// }
///
/// impl Runnable<MyMessage> for MyTask {
///     async fn start(&self) {
///         // Task logic here
///     }
/// }
/// # #[derive(Clone)]
/// # enum MyMessage {}
/// ```
#[proc_macro_derive(Task, attributes(task))]
pub fn derive_task(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_task_derive(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_task_derive(input: &DeriveInput) -> Result<quote::__private::TokenStream> {
    let name = &input.ident;

    // Parse the #[task(message = T)] attribute
    let message_type = parse_task_attribute(&input.attrs)?;

    // Generate the module name for task-local storage
    let mod_name = format_ident!("__{name}_gen");
    let task_state = format_ident!("{name}State");

    // Generate the Task trait implementation
    let generated = quote! {
        impl notizia::Task<#message_type> for #name {
            fn __setup(&self, receiver: notizia::tokio::sync::mpsc::UnboundedReceiver<#message_type>) -> impl std::future::Future<Output = ()> + Send {
                async move {
                    let mb = self.mailbox();

                    mb.set_receiver(receiver).await;

                    self.start().await
                }
            }

            fn mailbox(&self) -> notizia::Mailbox<#message_type> {
                #mod_name::#task_state.get().mailbox
            }

            fn run(self) -> notizia::TaskHandle<#message_type> {
                let (sender, receiver) = notizia::tokio::sync::mpsc::unbounded_channel::<#message_type>();

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

            fn this(&self) -> notizia::TaskRef<#message_type> {
                notizia::TaskRef::new(#mod_name::#task_state.get().sender)
            }
        }

        mod #mod_name {
            use super::*;

            tokio::task_local! {
                pub static #task_state: notizia::TaskState<#message_type>;
            }
        }
    };

    Ok(generated)
}

/// Parse the #[task(message = T)] attribute to extract the message type.
fn parse_task_attribute(attrs: &[Attribute]) -> Result<Type> {
    // Find the #[task(...)] attribute
    let task_attr = attrs
        .iter()
        .find(|attr| attr.path().is_ident("task"))
        .ok_or_else(|| {
            Error::new_spanned(
                &attrs.first(),
                "Missing #[task(message = T)] attribute. \
                 The Task derive macro requires specifying the message type.\n\
                 Example: #[task(message = MyMessage)]",
            )
        })?;

    // Parse the attribute as a list: #[task(message = T)]
    let meta = &task_attr.meta;

    match meta {
        Meta::List(list) => {
            // Parse the nested meta items
            let nested: MetaNameValue = syn::parse2(list.tokens.clone()).map_err(|_| {
                Error::new_spanned(
                    meta,
                    "Expected #[task(message = Type)].\n\
                     The task attribute must be in the form: #[task(message = YourMessageType)]",
                )
            })?;

            // Check that the name is "message"
            if !nested.path.is_ident("message") {
                return Err(Error::new_spanned(
                    &nested.path,
                    "Expected 'message' parameter.\n\
                     Use: #[task(message = YourMessageType)]",
                ));
            }

            // Extract the type from the value
            match &nested.value {
                syn::Expr::Path(expr_path) => Ok(Type::Path(syn::TypePath {
                    qself: None,
                    path: expr_path.path.clone(),
                })),
                _ => Err(Error::new_spanned(
                    &nested.value,
                    "Expected a type for the message parameter.\n\
                     Example: #[task(message = MyMessage)]",
                )),
            }
        }
        Meta::Path(_) => Err(Error::new_spanned(
            meta,
            "The #[task] attribute requires parameters.\n\
             Use: #[task(message = YourMessageType)]",
        )),
        Meta::NameValue(_) => Err(Error::new_spanned(
            meta,
            "Invalid task attribute format.\n\
             Use: #[task(message = YourMessageType)]",
        )),
    }
}
