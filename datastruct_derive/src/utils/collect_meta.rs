use crate::utils::synerr::{ResultExt, SynErrorExt};
use proc_macro2::Span;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use syn::spanned::Spanned;
use syn::{Ident, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

pub fn collect_meta_set<T: Hash + Eq>(
    meta_list: &MetaList,
    mut func: impl FnMut(&str, Span) -> syn::Result<T>,
) -> syn::Result<HashSet<T>> {
    let mut set = HashSet::new();
    let mut err: Option<syn::Error> = None;

    for meta in &meta_list.nested {
        match meta {
            NestedMeta::Meta(Meta::Path(pth)) => {
                if let Some(path) = pth.get_ident() {
                    match func(&path.to_string(), meta.span()) {
                        Ok(val) => {
                            set.insert(val);
                        }
                        Err(e) => err.update_or_combine(e),
                    }
                } else {
                    err.update_or_combine(syn::Error::new(
                        meta.span(),
                        "invalid meta argument, expect ident or string literal"
                    ))
                }
            }
            NestedMeta::Lit(Lit::Str(lit)) => match func(&lit.value(), meta.span()) {
                Ok(val) => {
                    set.insert(val);
                }
                Err(e) => err.update_or_combine(e),
            },
            _ => err.update_or_combine(syn::Error::new(
                meta.span(),
                "invalid meta argument, expect ident or string literal",
            )),
        }
    }

    err.ok_or(()).swap()?;

    Ok(set)
}

pub fn collect_meta_map<K: Hash + Eq, V>(
    meta_list: &MetaList,
    mut func: impl FnMut(usize, &Ident, Option<&Lit>) -> syn::Result<(K, V)>,
) -> syn::Result<HashMap<K, V>> {
    let mut map = HashMap::new();
    let mut err: Option<syn::Error> = None;

    for (idx, meta) in meta_list.nested.iter().enumerate() {
        if let NestedMeta::Meta(meta) = meta {
            match meta {
                Meta::Path(pth) => {
                    if let Some(ident) = pth.get_ident() {
                        let res = func(idx, ident, None);
                        match res {
                            Err(e) => err.update_or_combine(e),
                            Ok(res) => {
                                map.insert(res.0, res.1);
                            }
                        }
                    }
                }
                Meta::NameValue(MetaNameValue { path: pth, lit, .. }) => {
                    if let Some(ident) = pth.get_ident() {
                        let res = func(idx, ident, Some(lit));
                        match res {
                            Err(e) => err.update_or_combine(e),
                            Ok(res) => {
                                map.insert(res.0, res.1);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    err.ok_or(()).swap()?;

    Ok(map)
}
