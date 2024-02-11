use crate::cmp::FieldCmpConfig;
use crate::ops::FieldOpsConfig;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, Lit, Meta, MetaNameValue, NestedMeta, Type};

#[derive(Clone)]
pub struct FieldConfig {
    pub default_value: Option<Expr>,
    pub init_seq: Option<isize>,
    pub auto_set: SetterType,
    pub auto_get: GetterType,
    pub no_debug: bool,
    /// `do_with_xxx(&mut self, f: impl FnOnce(&mut value))`
    pub do_with: bool,
    /// `map_xxx(mut self, f: impl FnOnce(value) -> value) -> Self`
    pub map: bool,
    pub cmp: FieldCmpConfig,
    pub ops: FieldOpsConfig,
}

impl FieldConfig {
    pub fn from_attribute(
        attrs: Vec<Attribute>,
        default_set: SetterType,
        default_get: GetterType,
    ) -> syn::Result<(Self, Vec<Attribute>)> {
        let mut avec: Vec<Attribute> = Vec::with_capacity(attrs.len());
        let mut config = Self {
            default_value: None,
            init_seq: None,
            auto_set: default_set,
            auto_get: default_get,
            no_debug: false,
            do_with: false,
            map: false,
            cmp: Default::default(),
            ops: Default::default(),
        };

        for attr in attrs {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                if meta_list.path.is_ident("dfield") {
                    for meta in meta_list.nested {
                        if let NestedMeta::Meta(meta) = meta {
                            if meta.path().is_ident("default") {
                                match meta {
                                    Meta::NameValue(
                                        MetaNameValue {
                                            lit: Lit::Str(lit), ..
                                        }) => {
                                        if lit.value().is_empty() {
                                            return Err(syn::Error::new(
                                                lit.span(),
                                                "`default` value should not be empty",
                                            ));
                                        }
                                        config.default_value = Some(syn::parse_str(&lit.value()).map_err(|mut e| {
                                            e.extend(syn::Error::new(
                                                lit.span(),
                                                "`default` value should be a valid expression",
                                            ));
                                            e
                                        })?);
                                        continue;
                                    }
                                    _ => return Err(syn::Error::new(
                                        meta.span(),
                                        "invalid `default` value, see the documentation for more information",
                                    ))
                                }
                            } else if meta.path().is_ident("seq")
                                || meta.path().is_ident("sequence")
                            {
                                match meta {
                                    Meta::NameValue(
                                        MetaNameValue {
                                            lit: Lit::Int(lit), ..
                                        }) => {
                                        let value: isize = lit.base10_parse()?;
                                        config.init_seq = Some(value);
                                        continue;
                                    }
                                    _ => return Err(syn::Error::new(
                                        meta.span(),
                                        "invalid `seq` value, see the documentation for more information",
                                    ))
                                }
                            } else if meta.path().is_ident("get") {
                                match meta {
                                    Meta::Path(_) => config.auto_get = Default::default(),
                                    Meta::NameValue(
                                        MetaNameValue {
                                            lit: Lit::Str(lit), ..
                                        }) => {
                                        config.auto_get = GetterType::from_str(lit.value())
                                            .ok_or_else(|| syn::Error::new(lit.span(), "unknown `get` type"))?;
                                        continue;
                                    }
                                    _ => return Err(syn::Error::new(
                                        meta.span(),
                                        "invalid `get` value, see the documentation for more information",
                                    ))
                                }
                            } else if meta.path().is_ident("set") {
                                match meta {
                                    Meta::Path(_) => config.auto_set = Default::default(),
                                    Meta::NameValue(
                                        MetaNameValue {
                                            lit: Lit::Str(lit), ..
                                        }) => {
                                        config.auto_set = SetterType::from_str(lit.value())
                                            .ok_or_else(|| syn::Error::new(lit.span(), "unknown `set` type"))?;
                                        continue;
                                    }
                                    _ => return Err(syn::Error::new(
                                        meta.span(),
                                        "invalid `set` value, see the documentation for more information",
                                    ))
                                }
                            } else if meta.path().is_ident("do_with") {
                                match meta {
                                    Meta::Path(_) => config.do_with = true,
                                    Meta::NameValue(
                                        MetaNameValue {
                                            lit: Lit::Bool(lit), ..
                                        }) => {
                                        config.do_with = lit.value
                                    }
                                    _ => return Err(syn::Error::new(
                                        meta.span(),
                                        "invalid `do_with` value, see the documentation for more information",
                                    ))
                                }
                            } else if meta.path().is_ident("map") {
                                match meta {
                                    Meta::Path(_) => config.map = true,
                                    Meta::NameValue(
                                        MetaNameValue {
                                            lit: Lit::Bool(lit), ..
                                        }) => {
                                        config.map = lit.value
                                    }
                                    _ => return Err(syn::Error::new(
                                        meta.span(),
                                        "invalid `map` value, see the documentation for more information",
                                    ))
                                }
                            } else if meta.path().is_ident("no_debug") {
                                match meta {
                                    Meta::Path(_) => config.no_debug = true,
                                    Meta::NameValue(
                                        MetaNameValue {
                                            lit: Lit::Bool(lit), ..
                                        }) => {
                                        config.no_debug = lit.value
                                    }
                                    _ => return Err(syn::Error::new(
                                        meta.span(),
                                        "invalid `no_debug` value, see the documentation for more information",
                                    ))
                                }
                            } else if meta.path().is_ident("cmp") {
                                if let Meta::List(ml) = meta {
                                    let cmp_cfg = FieldCmpConfig::from_meta(&ml)?;
                                    config.cmp = cmp_cfg;
                                } else {
                                    return Err(syn::Error::new(meta.span(), "invalid `cmp` value, see the documentation for more information"));
                                }
                            } else if meta.path().is_ident("ops") {
                                if let Meta::List(ml) = meta {
                                    let ops_cfg = FieldOpsConfig::from_meta(&ml)?;
                                    config.ops = ops_cfg;
                                } else {
                                    return Err(syn::Error::new(meta.span(), "invalid `ops` value, see the documentation for more information"));
                                }
                            }
                        }
                    }
                }
            }

            avec.push(attr)
        }

        Ok((config, avec))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetterType {
    /// Both `Set` and `With`
    #[default]
    Full,
    /// `set_xxx(&mut self, value)`
    Set,
    /// `with_xxx(mut self, value) -> Self`
    With,
    No,
}

impl SetterType {
    pub fn from_str(s: impl AsRef<str>) -> Option<Self> {
        match s.as_ref() {
            "full" | "all" => Some(SetterType::Full),
            "set" => Some(SetterType::Set),
            "with" => Some(SetterType::With),
            "no" => Some(SetterType::No),
            _ => None,
        }
    }

    fn set(ident: &str, ty: &Type, span: &Span) -> TokenStream2 {
        let func_name = proc_macro2::Ident::new(&format!("set_{ident}"), *span);
        let ident = proc_macro2::Ident::new(ident, *span);
        quote! {
            pub fn #func_name(&mut self, #ident: #ty) {
                self.#ident = #ident;
            }
        }
    }

    fn with(ident: &str, ty: &Type, span: &Span) -> TokenStream2 {
        let func_name = proc_macro2::Ident::new(&format!("with_{ident}"), *span);
        let ident = proc_macro2::Ident::new(ident, *span);
        quote! {
            pub fn #func_name(mut self, #ident: #ty) -> Self {
                self.#ident = #ident;
                self
            }
        }
    }

    pub fn to_code(self, ident: &str, ty: &Type, span: &Span) -> Vec<TokenStream2> {
        match self {
            Self::Full => vec![Self::set(ident, ty, span), Self::with(ident, ty, span)],
            Self::Set => vec![Self::set(ident, ty, span)],
            Self::With => vec![Self::with(ident, ty, span)],
            Self::No => vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GetterType {
    /// Both `Move` and `Get`.
    Full,
    /// `get_xxx(self) -> value`
    Move,
    /// `xxx(&self) -> &value`
    #[default]
    Get,
    No,
}

impl GetterType {
    pub fn from_str(s: impl AsRef<str>) -> Option<Self> {
        match s.as_ref() {
            "full" | "all" => Some(GetterType::Full),
            "move" => Some(GetterType::Move),
            "get" => Some(GetterType::Get),
            "no" => Some(GetterType::No),
            _ => None,
        }
    }

    fn get(ident: &str, ty: &Type, span: &Span) -> TokenStream2 {
        let func_name = proc_macro2::Ident::new(ident, *span);
        let ident = proc_macro2::Ident::new(ident, *span);
        quote! {
            pub fn #func_name(&self) -> &#ty {
                &self.#ident
            }
        }
    }

    fn r#move(ident: &str, ty: &Type, span: &Span) -> TokenStream2 {
        let func_name = proc_macro2::Ident::new(&format!("get_{ident}"), *span);
        let ident = proc_macro2::Ident::new(ident, *span);
        quote! {
            pub fn #func_name(self) -> #ty {
                self.#ident
            }
        }
    }

    pub fn to_code(self, ident: &str, ty: &Type, span: &Span) -> Vec<TokenStream2> {
        match self {
            Self::Full => vec![Self::get(ident, ty, span), Self::r#move(ident, ty, span)],
            Self::Get => vec![Self::get(ident, ty, span)],
            Self::Move => vec![Self::r#move(ident, ty, span)],
            Self::No => vec![],
        }
    }
}
