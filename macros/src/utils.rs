use proc_macro2::TokenStream;
use quote::quote;
use regex::Regex;
use std::collections::HashSet;
use syn::Ident;
use syn::*;

pub fn parse_parent() -> File {
    let path = proc_macro::Span::call_site().source_file().path();
    let source = std::fs::read_to_string(path).unwrap_or_default();
    parse_file(source.as_str()).unwrap()
}

pub fn get_generic_requirements<I, J>(inputs: I, params: J) -> Vec<Ident>
where
    I: Iterator<Item = Type>,
    J: Iterator<Item = GenericParam>,
{
    let params = params.collect::<Vec<_>>();
    let maybe_generic_inputs = inputs
        .filter_map(|input| match input {
            Type::Path(path) => Some(path),
            Type::Reference(reference) => match *reference.elem {
                Type::Path(path) => Some(path),
                _ => None,
            },
            _ => None,
        })
        .flat_map(|path| {
            let mut paths = vec![];
            fn add_arguments(path: &TypePath, paths: &mut Vec<PathSegment>) {
                let last = path.path.segments.last().unwrap();
                paths.push(last.clone());
                if let PathArguments::AngleBracketed(ref args) = last.arguments {
                    for arg in args.args.iter() {
                        if let GenericArgument::Type(ty) = arg {
                            let maybe_path = match ty {
                                Type::Path(path) => Some(path),
                                Type::Reference(reference) => match *reference.elem {
                                    Type::Path(ref path) => Some(path),
                                    _ => None,
                                },
                                _ => None,
                            };
                            maybe_path.map(|path| add_arguments(path, paths));
                        }
                    }
                }
            }
            add_arguments(&path, &mut paths);

            paths
        });
    let mut requirements = vec![];
    for input in maybe_generic_inputs {
        params
            .iter()
            .filter_map(|param| match param {
                GenericParam::Type(param) => Some(param),
                _ => None,
            })
            .find(|param| param.ident == input.ident)
            .map(|param| {
                requirements.push(param.ident.clone());
            });
    }
    let req_set: HashSet<_> = requirements.into_iter().collect();
    req_set.into_iter().collect()
}

pub fn relevant_impls(names: Vec<&Ident>, source: &File) -> Vec<ItemImpl> {
    source
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(item) => Some(item),
            _ => None,
        })
        .filter(|item| item.trait_.is_none())
        .filter(|item| {
            let path = match &*item.self_ty {
                Type::Path(path) => path,
                _ => return false,
            };

            if path.qself.is_some() {
                return false;
            }
            if path.path.segments.len() != 1 {
                return false;
            }
            if !names.contains(&&path.path.segments[0].ident) {
                return false;
            }

            true
        })
        .cloned()
        .collect()
}

pub fn relevant_methods(
    name: &Ident,
    attr: &str,
    source: &File,
) -> Vec<(ImplItemMethod, ItemImpl)> {
    let get_methods = |item: ItemImpl| -> Vec<_> {
        item.items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Method(method) => Some(method),
                _ => None,
            })
            .filter(|method| {
                method
                    .attrs
                    .iter()
                    .find(|a| a.path.is_ident(&attr))
                    .is_some()
            })
            .filter(|method| matches!(method.vis, Visibility::Public(_)))
            .filter(|method| method.sig.unsafety.is_none())
            .filter(|method| method.sig.asyncness.is_none())
            .filter(|method| method.sig.abi.is_none())
            .map(|method| (method.clone(), item.clone()))
            .collect()
    };

    relevant_impls(vec![name, &strip_version(name)], source)
        .into_iter()
        .flat_map(get_methods)
        .collect()
}

pub fn gen_param_input(generics: &Generics, bracketed: bool) -> TokenStream {
    let gen_params = generics.params.iter().map(|p| match p {
        GenericParam::Type(p) => {
            let ident = &p.ident;
            quote!(#ident)
        }
        GenericParam::Lifetime(p) => {
            let ident = &p.lifetime.ident;
            quote!(#ident)
        }
        GenericParam::Const(p) => {
            let ident = &p.ident;
            quote!(#ident)
        }
    });

    if gen_params.len() == 0 {
        quote!()
    } else if bracketed {
        quote!(<#(#gen_params),*>)
    } else {
        quote!(#(#gen_params),*)
    }
}

pub fn strip_version(ident: &Ident) -> Ident {
    let name = ident.to_string();
    let re = Regex::new(r"V([0-9]+)").unwrap();
    let stripped_name = re.replace_all(&name, "").to_string();
    Ident::new(stripped_name.as_str(), ident.span())
}

macro_rules! named_fields {
    ($target:ident) => {
        $target
            .data
            .as_ref()
            .take_struct()
            .unwrap()
            .fields
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                (f.ident.as_ref().map(|v| quote!(#v)).unwrap_or_else(|| {
                    let i = syn::Index::from(i);
                    quote!(#i)
                }), f)
            })
    };
}

pub(crate) use named_fields;

pub struct Types {
    pub state_trait: TokenStream,
    pub store_ty: TokenStream,
    pub attacher_ty: TokenStream,
    pub flusher_ty: TokenStream,
    pub loader_ty: TokenStream,
    pub result_ty: TokenStream,
    pub terminated_trait: TokenStream,
    pub encode_trait: TokenStream,
    pub decode_trait: TokenStream,
    pub encoder_ty: TokenStream,
    pub decoder_ty: TokenStream,
}

impl Default for Types {
    fn default() -> Self {
        Self {
            state_trait: quote! { ::orga::state::State },
            store_ty: quote! { ::orga::store::Store },
            attacher_ty: quote! { ::orga::state::Attacher },
            flusher_ty: quote! { ::orga::state::Flusher },
            loader_ty: quote! { ::orga::state::Loader },
            result_ty: quote! { ::orga::Result },
            terminated_trait: quote! { ::orga::encoding::Terminated },
            encode_trait: quote! { ::orga::encoding::Encode },
            decode_trait: quote! { ::orga::encoding::Decode },
            encoder_ty: quote! { ::orga::encoding::encoder::Encoder },
            decoder_ty: quote! { ::orga::encoding::decoder::Decoder },
        }
    }
}
