use syn::parse_quote;
#[derive(Debug)]
pub struct Fixture {
    lt_param_async_trait: syn::GenericParam,
    pat_self: syn::Pat,
    ty_arc_self: syn::Type,
}

impl Fixture {
    const ROUTER: &'static str = "router";
    const GET: &'static str = "get";
    const DOC: &'static str = "doc";
    const RESULT: &'static str = "Result";

    pub fn new() -> Self {
        Fixture {
            lt_param_async_trait: parse_quote!('async_trait),
            pat_self: parse_quote!(self),
            ty_arc_self: parse_quote!(Arc<Self>),
        }
    }

    pub fn is_fn_router(&self, sig: &syn::Signature) -> bool {
        sig.ident == Self::ROUTER
    }

    pub fn is_async_trait_param(&self, param: &syn::GenericParam) -> bool {
        param == &self.lt_param_async_trait
    }

    pub fn is_arc_self(&self, arg: &syn::FnArg) -> bool {
        match arg {
            syn::FnArg::Receiver(_) => false,
            syn::FnArg::Typed(arg) => *arg.pat == self.pat_self && *arg.ty == self.ty_arc_self,
        }
    }

    pub fn is_method_attr(&self, attr: &syn::Attribute) -> bool {
        attr.path.is_ident(Self::GET)
    }

    pub fn is_arg_attr(&self, attr: &syn::Attribute) -> bool {
        attr.path.is_ident(Self::DOC)
    }

    pub fn is_result_type(&self, ty: &syn::Type) -> bool {
        match ty {
            syn::Type::Path(ty) => {
                ty.qself.is_none() && ty.path.segments.last().unwrap().ident == Self::RESULT
            }
            syn::Type::Paren(syn::TypeParen { elem, .. })
            | syn::Type::Group(syn::TypeGroup { elem, .. }) => self.is_result_type(elem),
            _ => false,
        }
    }

    pub fn is_get(&self, p: &syn::Path) -> bool {
        p.is_ident(Self::GET)
    }
}
