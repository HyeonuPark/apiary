use proc_macro2::TokenStream;
use proc_macro_error::{emit_call_site_error, emit_error};
use syn::parse_quote;
use syn::punctuated::Punctuated;

mod extract;
mod fixture;
mod parse;

mod server;

use fixture::Fixture;
use parse::{ParamSrc, Parsed};

pub fn process(args: syn::AttributeArgs, mut input_trait: syn::ItemTrait) -> Option<TokenStream> {
    let fixture = Fixture::new();
    if async_trait_expanded_before(&input_trait, &fixture) {
        emit_call_site_error!("#[api] should be placed above the #[async_trait]");
        return None;
    }

    let mut server_args = None;

    for arg in args {
        let (path, args) = match arg {
            syn::NestedMeta::Lit(_) | syn::NestedMeta::Meta(syn::Meta::NameValue(_)) => {
                emit_error!(arg, "Invalid parameter");
                continue;
            }
            syn::NestedMeta::Meta(syn::Meta::Path(path)) => (path, Punctuated::new()),
            syn::NestedMeta::Meta(syn::Meta::List(list)) => (list.path, list.nested),
        };

        if fixture.is_server(&path) {
            if server_args.is_some() {
                emit_error!(path, "Duplicated server parameter");
            } else {
                server_args = server::parse_args(args);
            }
        } else {
            emit_error!(path, "Invalid parameter");
        }
    }

    let extracted = extract::extract(&mut input_trait, &fixture)?;
    let parsed = parse::parse(&extracted, &fixture)?;

    let mut generated = vec![syn::Item::Trait(input_trait)];

    if let Some(args) = server_args {
        generated.append(&mut server::codegen(args, &parsed));
    }

    Some(quote::quote! {
        #(#generated)*
    })
}

fn async_trait_expanded_before(input_trait: &syn::ItemTrait, fixture: &Fixture) -> bool {
    input_trait
        .items
        .iter()
        .filter_map(|item| match item {
            syn::TraitItem::Method(meth) => Some(meth),
            _ => None,
        })
        .flat_map(|meth| &meth.sig.generics.params)
        .any(|p| fixture.is_async_trait_param(p))
}

pub fn _codegen(parsed: &Parsed) -> syn::TraitItemMethod {
    let body: Vec<syn::Stmt> = parsed
        .handlers
        .iter()
        .map(|handler| {
            let method = handler.http_method.ident();
            let method: syn::Expr = parse_quote!(apiary::Method::#method);
            let handler_name = handler.name.clone();
            let path_len = handler.path.len();

            let decl_fields: Vec<syn::Field> = handler
                .params
                .iter()
                .map(|param| syn::Field {
                    attrs: vec![],
                    vis: syn::Visibility::Inherited,
                    ident: Some(param.name.clone()),
                    colon_token: Some(Default::default()),
                    ty: param.ty.clone(),
                })
                .collect();

            let mut conditions: Vec<syn::Expr> = vec![
                parse_quote!(req.method() != #method),
                parse_quote!(path.len() != #path_len),
            ];
            conditions.extend(
                handler
                    .path
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, seg)| Some((idx, seg.as_deref()?)))
                    .map(|(idx, seg)| parse_quote!(path[#idx] != #seg)),
            );
            let conditions = conditions.into_iter().rev().fold(
                parse_quote!(true),
                |acc: syn::Expr, new: syn::Expr| parse_quote!(#new && #acc),
            );

            let parse_fields: Vec<syn::Stmt> = handler
                .params
                .iter()
                .map(|param| {
                    let name = param.name.clone();
                    let ty = param.ty.clone();

                    match &param.src {
                        ParamSrc::Path { idx } => parse_quote! {
                            let #name: #ty = path[#idx].parse().ok()?;
                        },
                    }
                })
                .collect();

            let field_names: Vec<syn::Ident> = handler
                .params
                .iter()
                .map(|param| param.name.clone())
                .collect();

            parse_quote! {{
                struct Params {
                    #(#decl_fields,)*
                }

                fn parse(req: &apiary::Request<String>, path: &[&str]) -> Option<Params> {
                    if #conditions {
                        return None;
                    }

                    #(#parse_fields)*

                    Some(Params {
                        #(#field_names),*
                    })
                }

                if let Some(Params { #(#field_names),* }) = parse(&req, &path) {
                    return tokio::spawn(async move {
                        let res = this.#handler_name( #(#field_names),* ).await;
                        Ok(apiary::Response::new(res))
                    })
                }
            }}
        })
        .collect();

    parse_quote! {
        fn router(self: Arc<Self>) -> apiary::Router<Self> {
            apiary::Router {
                app: self,
                handler: |this, req, closed| -> apiary::router::HandlerResult {
                    let path: Vec<_> = req
                        .uri()
                        .path()
                        .strip_prefix("/")
                        .expect("Path doesn't starts with `/`")
                        .split('/')
                        .collect();
                    #(#body)*
                    tokio::spawn(apiary::default_404_not_found(req))
                },
            }
        }
    }
}
