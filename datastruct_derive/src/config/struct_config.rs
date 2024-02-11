use crate::cmp::StructCmpConfig;
use crate::config::field_config::{GetterType, SetterType};
use crate::utils::collect_meta::collect_meta_set;
use crate::ops::StructOpsConfig;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Attribute, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructConfig {
    pub generate_default: bool,
    pub const_default: bool,
    pub impl_std_default: bool,
    pub partial_default: bool,
    pub manual_debug: bool,
    pub override_auto_get: GetterType,
    pub override_auto_set: SetterType,
    pub cmp: StructCmpConfig,
    pub ops: StructOpsConfig
}

impl StructConfig {
    pub fn from_attribute(
        attrs: Vec<Attribute>,
        parent_span: Span,
    ) -> Result<(Self, Vec<Attribute>), syn::Error> {
        let mut config = StructConfig {
            generate_default: false,
            const_default: false,
            impl_std_default: false,
            partial_default: false,
            manual_debug: false,
            override_auto_get: GetterType::No,
            override_auto_set: SetterType::No,
            cmp: Default::default(),
            ops: Default::default(),
        };

        let mut avec = Vec::with_capacity(attrs.len());
        for attr in attrs {
            if let Ok(Some(ml)) = meta_list_from_attr(&attr) {
                for meta in ml.nested {
                    if let NestedMeta::Meta(meta) = meta {
                        if meta.path().is_ident("default") {
                            match meta {
                                Meta::Path(_) => config.generate_default = true,
                                Meta::NameValue(
                                    MetaNameValue {
                                        lit: Lit::Bool(lit),
                                        ..
                                    }) => config.generate_default = lit.value,
                                _ => return Err(syn::Error::new(
                                    meta.span(),
                                    "`default` argument should be like `default = true` or simply `default`",
                                ))
                            };
                            continue;
                        } else if meta.path().is_ident("const") {
                            match meta {
                                Meta::Path(_) => config.const_default = true,
                                Meta::NameValue(
                                    MetaNameValue {
                                        lit: Lit::Bool(lit),
                                        ..
                                    }) => config.const_default = lit.value,
                                _ => return Err(syn::Error::new(
                                    meta.span(),
                                    "`const` argument should be like `const = true` or simply `const`",
                                ))
                            }
                            continue;
                        } else if meta.path().is_ident("std_default") {
                            match meta {
                                Meta::Path(_) => config.impl_std_default = true,
                                Meta::NameValue(
                                    MetaNameValue {
                                        lit: Lit::Bool(lit),
                                        ..
                                    }) => config.impl_std_default = lit.value,
                                _ => return Err(syn::Error::new(
                                    meta.span(),
                                    "`std_default` argument should be like `std_default = true` or simply `std_default`",
                                ))
                            }
                            continue;
                        } else if meta.path().is_ident("debug") {
                            match meta {
                                Meta::Path(_) => config.manual_debug = true,
                                Meta::NameValue(
                                    MetaNameValue {
                                        lit: Lit::Bool(lit),
                                        ..
                                    }) => config.manual_debug = lit.value,
                                _ => return Err(syn::Error::new(
                                    meta.span(),
                                    "`debug` argument should be like `debug = true` or simply `debug`",
                                ))
                            }
                            continue;
                        } else if meta.path().is_ident("partial") {
                            match meta {
                                Meta::Path(_) => config.partial_default = true,
                                Meta::NameValue(
                                    MetaNameValue {
                                        lit: Lit::Bool(lit),
                                        ..
                                    }) => config.const_default = lit.value,
                                _ => return Err(syn::Error::new(
                                    meta.span(),
                                    "`partial` argument should be like `partial = true` or simply `partial`",
                                ))
                            }
                            continue;
                        } else if meta.path().is_ident("set") {
                            match meta {
                                Meta::Path(_) => config.override_auto_set = Default::default(),
                                Meta::NameValue(
                                    MetaNameValue {
                                        lit: Lit::Str(lit), ..
                                    }) => {
                                    config.override_auto_set = SetterType::from_str(lit.value())
                                        .ok_or_else(|| {
                                            syn::Error::new(lit.span(), "unknown `set` type")
                                        })?
                                }
                                _ => return Err(syn::Error::new(
                                    meta.span(),
                                    "invalid `set` value, see the documentation for more information",
                                ))
                            }
                            continue;
                        } else if meta.path().is_ident("get") {
                            match meta {
                                Meta::Path(_) => config.override_auto_get = Default::default(),
                                Meta::NameValue(
                                    MetaNameValue {
                                        lit: Lit::Str(lit), ..
                                    }) => {
                                    config.override_auto_get = GetterType::from_str(lit.value())
                                        .ok_or_else(|| {
                                            syn::Error::new(lit.span(), "unknown `get` type")
                                        })?
                                }
                                _ => return Err(syn::Error::new(
                                    meta.span(),
                                    "invalid `get` value, see the documentation for more information",
                                ))
                            }
                            continue;
                        } else if meta.path().is_ident("cmp") {
                            match meta {
                                Meta::List(ml) => {
                                    collect_meta_set(&ml, |item, span| {
                                        match item {
                                            "eq" => config.cmp.eq = true,
                                            "peq" | "partial_eq" => config.cmp.partial_eq = true,
                                            "ord" | "cmp" => config.cmp.ord = true,
                                            "partial_ord" | "pord" | "partial_cmp" | "pcmp" => config.cmp.partial_ord = true,
                                            _ => return Err(syn::Error::new(span, "invalid `cmp` value"))
                                        };
                                        Ok(())
                                    })?;
                                }
                                _ => return Err(syn::Error::new(
                                    meta.span(),
                                    "invalid `cmp` value, see the documentation for more information",
                                ))
                            }
                        } else if meta.path().is_ident("ops") {
                            match meta {
                                Meta::List(ml) => config.ops.mut_and(StructOpsConfig::from_meta(&ml)?),
                                _ => return Err(syn::Error::new(
                                    meta.span(),
                                    "invalid `cmp` value, see the documentation for more information",
                                ))
                            }
                        }
                    }
                }
            }

            avec.push(attr);
        }

        if (config.generate_default || config.const_default) && config.partial_default {
            return Err(syn::Error::new(
                parent_span,
                "partial default does nothing if all fields have default values.",
            ));
        }

        Ok((config, avec))
    }
}

fn meta_list_from_attr(attr: &Attribute) -> syn::Result<Option<MetaList>> {
    if let Meta::List(meta_list) = attr.parse_meta()? {
        if meta_list.path.is_ident("dstruct") {
            return Ok(Some(meta_list));
        }
    }

    Ok(None)
}
