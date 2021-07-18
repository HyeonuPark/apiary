use std::collections::HashMap;

use http::Uri;
use proc_macro_error::emit_error;

use super::extract::Extracted;
use super::fixture::Fixture;

#[derive(Debug)]
pub struct Parsed {
    pub handlers: Vec<Handler>,
}

#[derive(Debug)]
pub struct Handler {
    pub path_attr: syn::Attribute,
    pub name: syn::Ident,
    pub http_method: Method,
    pub path: Vec<Option<String>>,
    pub params: Vec<Param>,
}
#[derive(Debug)]
pub enum Method {
    Get,
}

#[derive(Debug)]
pub struct Param {
    pub name: syn::Ident,
    pub ty: syn::Type,
    pub src: ParamSrc,
}

#[derive(Debug)]
pub enum ParamSrc {
    Path { idx: usize, is_result: bool },
}

pub fn parse(extracted: &Extracted, fixture: &Fixture) -> Option<Parsed> {
    Some(Parsed {
        handlers: extracted
            .methods
            .iter()
            .filter_map(|method| {
                let mut http_method = None;
                let mut path_attr = None;

                for a in &method.attrs {
                    if fixture.is_get(&a.path) {
                        http_method = Some(Method::Get);
                        path_attr = Some(a.clone());
                    } else {
                        emit_error!(a, "Unexpected attribute");
                        return None;
                    }
                }

                let http_method = http_method?;
                let path_attr = path_attr?;

                let path = match path_attr.parse_meta() {
                    Ok(syn::Meta::List(list)) if list.nested.len() == 1 => {
                        list.nested.first().unwrap().clone()
                    }
                    _ => {
                        emit_error!(path_attr, "Failed to parse attribute");
                        return None;
                    }
                };
                let mut path = match path {
                    syn::NestedMeta::Lit(syn::Lit::Str(lit)) => lit.value(),
                    _ => {
                        emit_error!(path, "Failed to parse attribute");
                        return None;
                    }
                };
                if let Some(idx) = path.find('?') {
                    emit_error!(path_attr, "URI with query string is not supported");
                    path.truncate(idx);
                }
                if let Some(idx) = path.find('#') {
                    emit_error!(path_attr, "URI with hash fragment is not supported");
                    path.truncate(idx);
                }
                let path = path.strip_prefix('/').unwrap_or_else(|| {
                    emit_error!(path_attr, "URI should be started with `/`");
                    &path
                });

                let mut path_params = HashMap::new();
                let path: Vec<_> = path
                    .split('/')
                    .enumerate()
                    .map(|(idx, seg)| {
                        if let Some(name) =
                            seg.strip_prefix('{').and_then(|seg| seg.strip_suffix('}'))
                        {
                            path_params.insert(name.to_owned(), idx);
                            None
                        } else {
                            Some(seg.to_owned())
                        }
                    })
                    .collect();

                let path_check: String = path
                    .iter()
                    .filter_map(|seg| seg.as_deref())
                    .map(|seg| format!("/{}", seg))
                    .collect();
                if path_check.parse::<Uri>().is_err() {
                    emit_error!(path_attr, "Invalid URI");
                }

                let params: Vec<_> = method
                    .args
                    .iter()
                    .filter_map(|arg| {
                        if let Some(idx) = path_params.remove(&arg.name.to_string()) {
                            let is_result = fixture.is_result_type(&arg.ty);

                            Some(Param {
                                name: arg.name.clone(),
                                ty: arg.ty.clone(),
                                src: ParamSrc::Path { idx, is_result },
                            })
                        } else {
                            emit_error!(
                                arg.name,
                                "Fn parameter {} is not found from the URI parameters",
                                arg.name
                            );
                            None
                        }
                    })
                    .collect();

                for param in path_params.keys() {
                    emit_error!(
                        path_attr,
                        "Parameter {} not found from the function parameters",
                        param
                    );
                }

                Some(Handler {
                    path_attr,
                    name: method.name.clone(),
                    http_method,
                    path,
                    params,
                })
            })
            .collect(),
    })
}

impl Method {
    pub fn ident(&self) -> syn::Ident {
        match self {
            Self::Get => quote::format_ident!("GET"),
        }
    }
}
