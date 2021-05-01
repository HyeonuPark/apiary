use std::collections::HashMap;
use std::convert::TryInto;
use std::mem;

use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::{ToTokens, quote_spanned};
use syn::parse_quote;

pub fn process(_param: TokenStream, item: TokenStream) -> Option<TokenStream> {
    let mut input_trait: syn::ItemTrait = syn::parse2(item).expect("some fancy msg");
    // panic!("Received input trait: {:#?}", input_trait);

    let mut handler: syn::ExprBlock = parse_quote! {{
        let (request, body) = request.into_parts();
        let method = request.method.clone();
        let paths = request.path();
        let paths = if paths.starts_with("/") {
            &paths[1..]
        } else {
            paths
        };
        let paths: Vec<_> = paths[1..].split('/').collect();
    }};

    for item in &mut input_trait.items {
        if let Some(expr) = extract_route(item) {
            handler
                .block
                .stmts
                .push(syn::Stmt::Expr(syn::Expr::If(expr)));
        }
    }

    let mut default_routes: Vec<_> = input_trait
        .items
        .iter_mut()
        .filter_map(extract_default_route)
        .collect();

    if default_routes.len() > 1 {
        for (_, attr) in &default_routes {
            emit_error!(attr, "Single trait can take up to single default route");
        }
        return None;
    }

    let default_route = match default_routes.pop() {
        Some((route, _)) => route,
        None => parse_quote! {
            tokio::spawn(apiary::internal_helper::default_404())
        },
    };
    handler.block.stmts.push(syn::Stmt::Expr(default_route));

    let to_router: syn::TraitItemMethod = parse_quote! {
        #[allow(warnings)]
        fn to_router(self: Arc<Self>) -> apiary::Router {
            apiary::Router {
                app: self,
                handler: |app, request, closed| #handler,
            }
        }
    };

    input_trait.items.push(syn::TraitItem::Method(to_router));

    Some(input_trait.to_token_stream())
}

thread_local! {
    static PATHS: HashMap<syn::Path, syn::Path> = {
        let mut map = HashMap::new();
        map.insert(parse_quote!(get), parse_quote!(apiary::Method::GET));
        map
    }
}

fn extract_route(item: &mut syn::TraitItem) -> Option<syn::ExprIf> {
    let method = match item {
        syn::TraitItem::Method(method) => method,
        _ => return None,
    };

    let (attrs, remaining): (Vec<_>, _) = mem::take(&mut method.attrs)
        .into_iter()
        .partition(|attr| PATHS.with(|paths| paths.contains_key(&attr.path)));
    method.attrs = remaining;

    if attrs.len() > 1 {
        for attr in &attrs {
            emit_error!(attr, "Single method can take up to single path attribute");
        }
    }

    let [attr]: [syn::Attribute; 1] = attrs.try_into().ok()?;

    let meta = match attr.parse_meta() {
        Ok(syn::Meta::List(meta)) if meta.nested.len() == 1 => meta,
        _ => {
            emit_error!(attr, "Invalid path attribute");
            return None;
        }
    };
    let http_method = PATHS.with(|paths| paths[&meta.path].clone());

    let path = match meta.nested.first().unwrap() {
        syn::NestedMeta::Lit(syn::Lit::Str(lit)) => lit.value(),
        _ => {
            emit_error!(meta, "Invalid path attribute");
            return None;
        }
    };
    let path = match path.trim().strip_prefix('/') {
        Some(path) => path,
        None => {
            emit_error!(meta, "Path attribute should starts with `/`");
            return None;
        }
    };

    if path.contains(&['?', '#'][..]) {
        emit_error!(
            meta,
            "Path attribute should not contains query or the hash fragments"
        );
        return None;
    }

    let mut path_params = Vec::new();
    let mut segments = Vec::new();
    let mut validator = String::new();

    for (idx, seg) in path.split('/').enumerate() {
        let param = seg.strip_prefix('{').and_then(|s| s.strip_suffix('}'));
        if let Some(name) = param {
            path_params.push((name, idx));
            segments.push(None);
        } else {
            segments.push(Some(seg));
            validator.push('/');
            validator.push_str(seg);
        }
    }

    if let Err(err) = validator.parse::<http::uri::PathAndQuery>() {
        emit_error!(meta, "Path attribute contains invalid URI - {}", err);
        return None;
    }

    let sig = &method.sig;

    if sig.asyncness.is_none() {
        emit_error!(sig, "Handler function should be async");
        return None;
    }
    if sig.variadic.is_some() {
        emit_error!(sig, "Handler function cannot be variadic");
        return None;
    }

    let is_arc_self = sig.inputs.first().map_or(false, |arg| {
        thread_local! {
            static PAT_SELF: syn::Pat = parse_quote!(self);
            static TY_ARC_SELF: syn::Type = parse_quote!(Arc<Self>);
        }
        match arg {
            syn::FnArg::Receiver(_) => false,
            syn::FnArg::Typed(pt) => {
                PAT_SELF.with(|var_self| var_self == &*pt.pat) &&
                    TY_ARC_SELF.with(|ty_arc_self| ty_arc_self == &*pt.ty)
            },
        }
    });
    if !is_arc_self {
        emit_error!(sig, "Handler function should take `self: Arc<Self>`");
        return None;
    }

    for arg in sig.inputs.iter().skip(1) {
        let arg = match arg {
            syn::FnArg::Receiver(_) => unreachable!("Only first arg can be the receiver"),
            syn::FnArg::Typed(arg) => arg,
        };
        let name = match &*arg.pat {
            syn::Pat::Ident(syn::PatIdent { subpat: None, ident, .. }) => ident,
            _ => {
                emit_error!(arg, "Argument should have single name");
                continue
            }
        };
        if let Some((_, idx)) = path_params.iter().find(|(param, idx)| name == param) {
            ;
        } else {
            emit_error!(arg, "Cannot find argument name from the path parameters");
            continue
        }
    }

    None
}

fn extract_default_route(item: &mut syn::TraitItem) -> Option<(syn::Expr, syn::Attribute)> {
    None
}
