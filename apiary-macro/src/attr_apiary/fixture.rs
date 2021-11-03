use syn::parse_quote;
#[derive(Debug)]
pub struct Fixture {
    lt_param_async_trait: syn::GenericParam,
    pat_self: syn::Pat,
    ty_arc_self: syn::Type,
}

impl Fixture {
    const GET: &'static str = "get";
    const DOC: &'static str = "doc";
    const SERVER: &'static str = "server";

    pub fn new() -> Self {
        Fixture {
            lt_param_async_trait: parse_quote!('async_trait),
            pat_self: parse_quote!(self),
            ty_arc_self: parse_quote!(Arc<Self>),
        }
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

    pub fn is_get(&self, p: &syn::Path) -> bool {
        p.is_ident(Self::GET)
    }

    pub fn is_server(&self, p: &syn::Path) -> bool {
        p.is_ident(Self::SERVER)
    }
}
