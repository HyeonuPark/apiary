use std::mem;

use proc_macro_error::emit_error;

use super::fixture::Fixture;

#[derive(Debug)]
pub struct Extracted {
    pub methods: Vec<Method>,
}

#[derive(Debug)]
pub struct Method {
    pub attrs: Vec<syn::Attribute>,
    pub name: syn::Ident,
    pub args: Vec<Arg>,
    pub return_ty: syn::Type,
}

#[derive(Debug)]
pub struct Arg {
    pub attrs: Vec<syn::Attribute>,
    pub name: syn::Ident,
    pub ty: syn::Type,
}

pub fn extract(input_trait: &mut syn::ItemTrait, fixture: &Fixture) -> Option<Extracted> {
    let methods = input_trait
        .items
        .iter_mut()
        .filter_map(|item| match item {
            syn::TraitItem::Method(method) => extract_method(method, fixture),
            _ => None,
        })
        .collect();

    Some(Extracted { methods })
}

fn extract_method(method: &mut syn::TraitItemMethod, fixture: &Fixture) -> Option<Method> {
    let attrs = mem::take(&mut method.attrs);
    let (attrs, remaining) = attrs
        .into_iter()
        .partition(|attr| fixture.is_method_attr(attr));
    method.attrs = remaining;

    if attrs.is_empty() {
        // method not related with the HTTP route
        return None;
    }

    let sig = method.sig.clone();
    let mut args = method.sig.inputs.iter_mut();
    let is_arc_self = args.next().map_or(false, |arg| fixture.is_arc_self(arg));
    if !is_arc_self {
        emit_error!(
            sig,
            "#[api] handler methods should take `self: Arc<Self>` argument"
        );
    }

    let args = args
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(arg) => Some(arg),
            syn::FnArg::Receiver(arg) => {
                emit_error!(arg, "Receiver can only be located as a first argument");
                None
            }
        })
        .filter_map(|arg| {
            let attrs = mem::take(&mut arg.attrs);
            let (attrs, remain) = attrs
                .into_iter()
                .partition(|attr| fixture.is_arg_attr(attr));
            arg.attrs = remain;

            let name = match &*arg.pat {
                syn::Pat::Ident(pat) => pat.ident.clone(),
                other => {
                    emit_error!(other, "#[api] handler function arguments should have single name like `name: Type`");
                    return None;
                }
            };

            Some(Arg {
                attrs,
                name,
                ty: (*arg.ty).clone(),
            })
        })
        .collect();

    Some(Method {
        attrs,
        name: sig.ident,
        args,
        return_ty: match sig.output {
            syn::ReturnType::Default => syn::Type::Tuple(syn::TypeTuple {
                paren_token: Default::default(),
                elems: Default::default(),
            }),
            syn::ReturnType::Type(_, ty) => *ty,
        },
    })
}
