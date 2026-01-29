use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Attribute, DeriveInput, Error, Expr, Field, Fields, ItemEnum, Meta,
    MetaNameValue, Result, Type, Variant,
};

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
            fn __setup(
                &self,
                receiver: notizia::tokio::sync::mpsc::UnboundedReceiver<#message_type>,
            ) -> impl std::future::Future<Output = notizia::TerminateReason> + Send {
                async move {
                    // Set up mailbox
                    let mb = self.mailbox();
                    mb.set_receiver(receiver).await;

                    // Execute start() and catch panics
                    let start_result = notizia::futures::FutureExt::catch_unwind(
                        std::panic::AssertUnwindSafe(self.start())
                    ).await;

                    // Determine termination reason
                    let reason = match start_result {
                        Ok(()) => notizia::TerminateReason::Normal,
                        Err(panic_payload) => {
                            // Extract panic message
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

                    // Call terminate hook, also catch panics
                    let terminate_result  = notizia::futures::FutureExt::catch_unwind(
                        std::panic::AssertUnwindSafe(self.terminate(reason.clone()))
                    ).await;

                    // Log if terminate() panicked
                    if let Err(terminate_panic) = terminate_result {
                        let msg = if let Some(s) = terminate_panic.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = terminate_panic.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "unknown panic".to_string()
                        };
                        eprintln!("Warning: terminate() hook panicked: {}", msg);
                    }

                    // Return the original termination reason
                    reason
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
                attrs.first(),
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

/// Attribute macro for message enums that automatically injects reply_to fields.
///
/// This macro allows marking enum variants with `#[request(reply = T)]` to automatically
/// inject a `reply_to: tokio::sync::oneshot::Sender<T>` field into the variant.
///
/// # Example
///
/// ```rust,ignore
/// use notizia::prelude::*;
/// use notizia_gen::message;
///
/// #[message]
/// #[derive(Debug)]
/// enum CounterMsg {
///     #[request(reply = u32)]
///     GetCount,
///     
///     #[request(reply = String)]
///     GetStatus,
///     
///     Increment,
///     Decrement,
/// }
/// ```
///
/// This expands to:
///
/// ```rust,ignore
/// #[derive(Debug)]
/// enum CounterMsg {
///     GetCount { reply_to: tokio::sync::oneshot::Sender<u32> },
///     GetStatus { reply_to: tokio::sync::oneshot::Sender<String> },
///     Increment,
///     Decrement,
/// }
/// ```
#[proc_macro_attribute]
pub fn message(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);

    match impl_message_macro(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_message_macro(input: &ItemEnum) -> Result<quote::__private::TokenStream> {
    let enum_name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let generics = &input.generics;

    // Process each variant
    let variants = input
        .variants
        .iter()
        .map(|variant| process_variant(variant))
        .collect::<Result<Vec<_>>>()?;

    // Generate the enum
    let generated = quote! {
        #(#attrs)*
        #vis enum #enum_name #generics {
            #(#variants),*
        }
    };

    Ok(generated)
}

/// Process a single enum variant, checking for #[request(reply = T)] attribute
fn process_variant(variant: &Variant) -> Result<quote::__private::TokenStream> {
    let variant_name = &variant.ident;
    let variant_attrs: Vec<_> = variant
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("request"))
        .collect();

    // Check for #[request(reply = T)] attribute
    if let Some(reply_type) = parse_request_attribute(&variant.attrs)? {
        // Inject reply_to field
        let fields = inject_reply_field(variant, &reply_type)?;

        Ok(quote! {
            #(#variant_attrs)*
            #variant_name #fields
        })
    } else {
        // Leave variant unchanged
        let discriminant = &variant.discriminant;
        let fields = &variant.fields;

        let disc_tokens = if let Some((eq, expr)) = discriminant {
            quote! { #eq #expr }
        } else {
            quote! {}
        };

        Ok(quote! {
            #(#variant_attrs)*
            #variant_name #fields #disc_tokens
        })
    }
}

/// Parse the #[request(reply = T)] attribute to extract the reply type.
fn parse_request_attribute(attrs: &[Attribute]) -> Result<Option<Type>> {
    // Find the #[request(...)] attribute
    let request_attr = attrs.iter().find(|attr| attr.path().is_ident("request"));

    let Some(request_attr) = request_attr else {
        return Ok(None);
    };

    let meta = &request_attr.meta;

    match meta {
        Meta::List(list) => {
            // Parse the nested meta items: #[request(reply = T)]
            let nested: MetaNameValue = syn::parse2(list.tokens.clone()).map_err(|_| {
                Error::new_spanned(
                    meta,
                    "Expected #[request(reply = Type)].\n\
                     The request attribute must be in the form: #[request(reply = YourReplyType)]",
                )
            })?;

            // Check that the name is "reply"
            if !nested.path.is_ident("reply") {
                return Err(Error::new_spanned(
                    &nested.path,
                    "Expected 'reply' parameter.\n\
                     Use: #[request(reply = YourReplyType)]",
                ));
            }

            // Extract the type from the value
            match &nested.value {
                Expr::Path(expr_path) => Ok(Some(Type::Path(syn::TypePath {
                    qself: None,
                    path: expr_path.path.clone(),
                }))),
                _ => Err(Error::new_spanned(
                    &nested.value,
                    "Expected a type for the reply parameter.\n\
                     Example: #[request(reply = u32)]",
                )),
            }
        }
        Meta::Path(_) => Err(Error::new_spanned(
            meta,
            "The #[request] attribute requires parameters.\n\
             Use: #[request(reply = YourReplyType)]",
        )),
        Meta::NameValue(_) => Err(Error::new_spanned(
            meta,
            "Invalid request attribute format.\n\
             Use: #[request(reply = YourReplyType)]",
        )),
    }
}

/// Inject reply_to field into the variant
fn inject_reply_field(
    variant: &Variant,
    reply_type: &Type,
) -> Result<quote::__private::TokenStream> {
    match &variant.fields {
        Fields::Named(fields) => {
            // Add reply_to to existing named fields
            let mut new_fields = fields.named.clone();

            let reply_field: Field = syn::parse_quote! {
                reply_to: ::notizia::tokio::sync::oneshot::Sender<#reply_type>
            };

            new_fields.push(reply_field);

            Ok(quote! {
                { #new_fields }
            })
        }
        Fields::Unit => {
            // Convert unit variant to struct variant with single field
            Ok(quote! {
                { reply_to: ::notizia::tokio::sync::oneshot::Sender<#reply_type> }
            })
        }
        Fields::Unnamed(_) => {
            // Error: Can't inject into tuple variant
            Err(Error::new_spanned(
                variant,
                "Cannot apply #[request] to tuple variants.\n\
                 Convert to a struct variant or unit variant.\n\
                 Example: Instead of `GetCount(u32)`, use `GetCount { id: u32 }` or just `GetCount`",
            ))
        }
    }
}
