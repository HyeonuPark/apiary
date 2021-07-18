use proc_macro2::TokenStream;
use proc_macro_error::{emit_call_site_error, emit_error, ResultExt};
use syn::parse_quote;

mod extract;
mod fixture;
mod parse;

use fixture::Fixture;
use parse::{ParamSrc, Parsed};

pub fn process(_param: TokenStream, item: TokenStream) -> Option<TokenStream> {
    let mut input_trait: syn::ItemTrait =
        syn::parse2(item).expect_or_abort("#[api] should be placed on the trait definition");

    let fixture = Fixture::new();
    if !is_valid(&input_trait, &fixture) {
        return None;
    }
    let extracted = extract::extract(&mut input_trait, &fixture)?;
    let parsed = parse::parse(&extracted, &fixture)?;
    let gen_method = codegen(&parsed);
    input_trait.items.push(syn::TraitItem::Method(gen_method));

    Some(quote::quote! {
        #input_trait
    })
}

fn is_valid(input_trait: &syn::ItemTrait, fixture: &Fixture) -> bool {
    for item in &input_trait.items {
        if let syn::TraitItem::Method(method) = item {
            if fixture.is_fn_router(&method.sig) {
                emit_error!(
                    method.sig.ident,
                    "method name conflict with the one generated from the #[api] attribute"
                )
            }

            for param in &method.sig.generics.params {
                if fixture.is_async_trait_param(param) {
                    emit_call_site_error!("#[api] should be placed above the #[async_trait]");
                    return false;
                }
            }
        }
    }

    true
}

pub fn codegen(parsed: &Parsed) -> syn::TraitItemMethod {
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
                        ParamSrc::Path {
                            idx,
                            is_result: true,
                        } => parse_quote! {
                            let #name: #ty = path[#idx].parse().map_err(From::from);
                        },
                        ParamSrc::Path {
                            idx,
                            is_result: false,
                        } => parse_quote! {
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
