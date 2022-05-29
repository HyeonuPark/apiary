use proc_macro_error::emit_error;
use syn::parse_quote;
use syn::spanned::Spanned;

use crate::attr_apiary::parse::{Handler, Method, Param, ParamSrc, Parsed};

#[derive(Debug)]
pub struct Args {
    type_name: syn::Ident,
}

pub fn parse_args(args: impl IntoIterator<Item = syn::NestedMeta> + Spanned) -> Option<Args> {
    let span = args.span();
    let mut args = args.into_iter();

    let type_name = match args.next() {
        Some(syn::NestedMeta::Meta(syn::Meta::Path(p))) => p,
        Some(_) => {
            emit_error!(span, "Invalid parameter, requires server type name");
            return None;
        }
        None => {
            emit_error!(span, "Missing server type name");
            return None;
        }
    };
    let type_name = match type_name.get_ident() {
        Some(id) => id.clone(),
        None => {
            emit_error!(span, "Invalid parameter, requires server type name");
            return None;
        }
    };

    if args.next().is_some() {
        emit_error!(span, "Too many attribute parameters, server only takes one");
    }

    Some(Args { type_name })
}

pub fn codegen(args: Args, parsed: &Parsed) -> Vec<syn::Item> {
    let handlers: Vec<_> = parsed.handlers.iter().map(codegen_handler).collect();

    let type_name = args.type_name;
    let trait_name = &parsed.trait_name;
    let type_def: syn::Item = parse_quote! {
        #[derive(Debug)]
        struct #type_name<T: ?Sized>(pub std::sync::Arc<T>);
    };
    let impl_clone: syn::Item = parse_quote! {
        impl<T: ?Sized> std::clone::Clone for #type_name<T> {
            fn clone(&self) -> Self {
                #type_name(Arc::clone(&self.0))
            }
        }
    };

    let impl_server: syn::Item = parse_quote! {
        impl<T: Foo + ?Sized> apiary::server::Server for #type_name<T> {
            fn serve<B>(self, request: apiary::http::Request<B>) -> apiary::server::ServeResult
            where
                B: apiary::http_body::Body,
            {
                Box::pin(async move {
                    #(#handlers)*

                    let fallback = apiary::http::Response::builder()
                        .status(404)
                        .body(apiary::response::Body::once("404 Not Found"))?;

                    Ok(fallback)
                })
            }
        }
    };

    vec![type_def, impl_clone, impl_server]
}

fn codegen_handler(handler: &Handler) -> syn::Stmt {
    syn::parse_quote!(();)
}
