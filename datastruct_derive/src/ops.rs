use crate::utils::collect_meta::collect_meta_map;
use crate::utils::synerr::{ResultExt, SynErrorExt};

use crate::generate::RichStructContent;
use itertools::Itertools;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{Expr, Ident, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StructOpsConfig {
    add: Option<OpsAssignableType>,
    sub: Option<OpsAssignableType>,
    mul: Option<OpsAssignableType>,
    div: Option<OpsAssignableType>,
}

macro_rules! __help_impl_struct_impl_ops {
    (non-assign $fn_name:ident, $field_name:ident, $impl_fn:ident, $trait_name:path, $trait_fn:ident) => {
        fn $fn_name(syntax: &RichStructContent) -> syn::Result<TokenStream2> {
            let (fields, err_list): (Vec<_>, Vec<_>) = syntax
                .fields
                .iter()
                .map(|field| {
                    field
                        .config
                        .ops
                        .$field_name
                        .clone()
                        .unwrap_or_default()
                        .$impl_fn(&field.ident)
                        .map(|op| {
                            let ident = &field.ident;
                            quote! { #ident: #op }
                        })
                })
                .partition_result();

            if !err_list.is_empty() {
                let mut err: Option<syn::Error> = None;
                for e in err_list {
                    err.update_or_combine(e)
                }

                err.ok_or(()).swap()?;
            }

            let ident = &syntax.ident;
            let (impl_g, type_g, where_clause) = syntax.generics.split_for_impl();

            let fields = fields.iter().filter(|token| !token.is_empty());

            Ok(quote! {
                impl #impl_g $trait_name for #ident #type_g #where_clause {
                    type Output = Self;

                    fn $trait_fn(self, rhs: Self) -> Self {
                        Self {
                            #(#fields),*
                        }
                    }
                }
            })
        }
    };
    (assign $fn_name:ident, $field_name:ident, $impl_fn:ident, $trait_name:path, $trait_fn:ident) => {
        fn $fn_name(syntax: &RichStructContent) -> syn::Result<TokenStream2> {
            let (fields, err_list): (Vec<_>, Vec<_>) = syntax
                .fields
                .iter()
                .map(|field| {
                    field
                        .config
                        .ops
                        .$field_name
                        .clone()
                        .unwrap_or_default()
                        .$impl_fn(&field.ident)
                })
                .partition_result();

            if !err_list.is_empty() {
                let mut err: Option<syn::Error> = None;
                for e in err_list {
                    err.update_or_combine(e)
                }

                err.ok_or(()).swap()?;
            }

            let ident = &syntax.ident;
            let (impl_g, type_g, where_clause) = syntax.generics.split_for_impl();

            Ok(quote! {
                impl #impl_g $trait_name for #ident #type_g #where_clause {
                    fn $trait_fn(&mut self, rhs: Self) {
                        #(#fields;)*
                    }
                }
            })
        }
    };
}

impl StructOpsConfig {
    pub fn mut_and(&mut self, other: Self) {
        macro_rules! __impl_override {
            ($self:ident, $other:ident, $($ident:ident),+ $(,)?) => {
                $(if let Some(v) = $other.$ident {
                    $self.$ident = Some(v)
                })+
            };
        }

        __impl_override!(self, other, add, sub, mul, div);
    }

    pub fn from_meta(meta_list: &MetaList) -> syn::Result<Self> {
        let mut config: StructOpsConfig = Default::default();

        let map: HashMap<OpsType, Option<OpsAssignableType>> =
            collect_meta_map(meta_list, |_, ident, lit| {
                let ops_type = OpsType::from_str(ident.to_string())
                    .ok_or_else(|| syn::Error::new(ident.span(), "invalid ops type"))?;
                let val: Option<OpsAssignableType> = if let Some(lit) = lit {
                    match lit {
                        Lit::Str(s) => {
                            Some(OpsAssignableType::from_str(s.value()).ok_or_else(|| {
                                syn::Error::new(lit.span(), "invalid ops operation type")
                            })?)
                        }
                        Lit::Bool(b) => {
                            if b.value {
                                Some(Default::default())
                            } else {
                                None
                            }
                        }
                        _ => return Err(syn::Error::new(lit.span(), "invalid ops operation type")),
                    }
                } else {
                    Some(Default::default())
                };

                Ok((ops_type, val))
            })?;

        config.add = map.get(&OpsType::Add).copied().flatten();
        config.sub = map.get(&OpsType::Sub).copied().flatten();
        config.mul = map.get(&OpsType::Mul).copied().flatten();
        config.div = map.get(&OpsType::Div).copied().flatten();

        Ok(config)
    }

    pub fn impl_ops(syntax: &RichStructContent) -> syn::Result<TokenStream2> {
        let mut ts = TokenStream2::new();
        let mut err: Option<syn::Error> = None;

        macro_rules! __help_impl_ops_item {
            ($err:ident, $ts:ident, $syntax:ident, $plain:ident, $assign:ident, $ident:ident) => {
                if let Some(v) = $syntax.config.ops.$ident {
                    match v {
                        OpsAssignableType::Both => {
                            match Self::$plain(syntax) {
                                Ok(v) => $ts.extend(v),
                                Err(e) => $err.update_or_combine(e),
                            }
                            match Self::$assign(syntax) {
                                Ok(v) => $ts.extend(v),
                                Err(e) => $err.update_or_combine(e),
                            }
                        }
                        OpsAssignableType::Plain => match Self::$plain(syntax) {
                            Ok(v) => $ts.extend(v),
                            Err(e) => $err.update_or_combine(e),
                        },
                        OpsAssignableType::Assign => match Self::$assign(syntax) {
                            Ok(v) => $ts.extend(v),
                            Err(e) => $err.update_or_combine(e),
                        },
                    }
                }
            };
        }

        __help_impl_ops_item! { err, ts, syntax, impl_add, impl_add_assign, add }
        __help_impl_ops_item! { err, ts, syntax, impl_sub, impl_sub_assign, sub }
        __help_impl_ops_item! { err, ts, syntax, impl_mul, impl_mul_assign, mul }
        __help_impl_ops_item! { err, ts, syntax, impl_div, impl_div_assign, div }

        err.ok_or(()).swap()?;

        Ok(ts)
    }

    __help_impl_struct_impl_ops!(non-assign impl_add, add, impl_add, ::std::ops::Add, add);
    __help_impl_struct_impl_ops!(non-assign impl_sub, sub, impl_sub, ::std::ops::Sub, sub);
    __help_impl_struct_impl_ops!(non-assign impl_mul, mul, impl_mul, ::std::ops::Mul, mul);
    __help_impl_struct_impl_ops!(non-assign impl_div, div, impl_div, ::std::ops::Div, div);

    __help_impl_struct_impl_ops!(assign impl_add_assign, add_assign, impl_add_assign, ::std::ops::AddAssign, add_assign);
    __help_impl_struct_impl_ops!(assign impl_sub_assign, sub_assign, impl_sub_assign, ::std::ops::SubAssign, sub_assign);
    __help_impl_struct_impl_ops!(assign impl_mul_assign, mul_assign, impl_mul_assign, ::std::ops::MulAssign, mul_assign);
    __help_impl_struct_impl_ops!(assign impl_div_assign, div_assign, impl_div_assign, ::std::ops::DivAssign, div_assign);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum OpsType {
    Add,
    Sub,
    Mul,
    Div,
}

impl OpsType {
    fn from_str(s: impl AsRef<str>) -> Option<Self> {
        match s.as_ref() {
            "add" => Some(OpsType::Add),
            "sub" => Some(OpsType::Sub),
            "mul" => Some(OpsType::Mul),
            "div" => Some(OpsType::Div),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum OpsAssignableType {
    Both,
    Assign,
    #[default]
    Plain,
}

impl OpsAssignableType {
    fn from_str(s: impl AsRef<str>) -> Option<Self> {
        match s.as_ref() {
            "both" | "all" => Some(Self::Both),
            "assign" => Some(Self::Assign),
            "plain" | "default" => Some(Self::Plain),
            _ => None,
        }
    }
}

#[derive(Clone, Default)]
pub struct FieldOpsConfig {
    add: Option<OpsOperationType>,
    sub: Option<OpsOperationType>,
    mul: Option<OpsOperationType>,
    div: Option<OpsOperationType>,
    add_assign: Option<OpsOperationType>,
    sub_assign: Option<OpsOperationType>,
    mul_assign: Option<OpsOperationType>,
    div_assign: Option<OpsOperationType>,
}

macro_rules! __help_impl_field_config_match {
    (
        $meta: ident, $config: ident, $err:ident:
        $ident:ident;
        $($ident2:ident;)+
    ) => {
        if $meta.path().is_ident(stringify!($ident)) {
            match $meta {
                Meta::Path(_) => $config.$ident = Some(Default::default()),
                Meta::NameValue(MetaNameValue { lit, .. }) => {
                    match OpsOperationType::from_lit(lit) {
                        Ok(op) => $config.$ident = Some(op),
                        Err(e) => $err.update_or_combine(e),
                    }
                }
                _ => $err.update_or_combine(syn::Error::new(
                    $meta.span(),
                    concat!("invalid ops `", stringify!($ident), "` type")
                ))
            }
        }
        $(else if $meta.path().is_ident(stringify!($ident2)) {
            match $meta {
                Meta::Path(_) => $config.$ident2 = Some(Default::default()),
                Meta::NameValue(MetaNameValue { lit, .. }) => {
                    match OpsOperationType::from_lit(lit) {
                        Ok(op) => $config.$ident2 = Some(op),
                        Err(e) => $err.update_or_combine(e),
                    }
                }
                _ => $err.update_or_combine(syn::Error::new(
                    $meta.span(),
                    concat!("invalid ops `", stringify!($ident2), "` type")
                ))
            }
        })+
        else {
            $err.update_or_combine(syn::Error::new($meta.span(), "invalid ops type"));
        }
    };
}

impl FieldOpsConfig {
    pub fn from_meta(meta_list: &MetaList) -> syn::Result<FieldOpsConfig> {
        let mut config: FieldOpsConfig = Default::default();
        let mut err: Option<syn::Error> = None;

        for meta in meta_list.nested.iter().filter_map(|s| match s {
            NestedMeta::Meta(mt) => Some(mt),
            _ => None,
        }) {
            __help_impl_field_config_match! {
                meta, config, err:
                add; sub; mul; div;
                add_assign; sub_assign; mul_assign; div_assign;
            }
        }

        err.ok_or(()).swap()?;

        Ok(config)
    }
}

#[derive(Debug, Clone, Default)]
enum OpsOperationType {
    Manual(String),
    #[default]
    Inherit,
    Ignore,
}

macro_rules! __help_impl_ops_operation {
    (non-assign $name:ident, $ops:tt) => {
        fn $name(&self, ident: &Ident) -> syn::Result<TokenStream2> {
            self._impl_ops(ident, quote! { $ops })
        }
    };

    (assign $name:ident, $ops:tt) => {
        fn $name(&self, ident: &Ident) -> syn::Result<TokenStream2> {
            self._impl_ops_assign(ident, quote! { $ops })
        }
    };
}

impl OpsOperationType {
    fn from_lit(lit: &Lit) -> syn::Result<Self> {
        match lit {
            Lit::Str(lit_str) => match lit_str.value().as_str() {
                "inherit" | "default" => Ok(Self::Inherit),
                "ignore" | "no" => Ok(Self::Ignore),
                n => Ok(Self::Manual(n.to_string())),
            },
            Lit::Bool(lit_bool) => {
                if lit_bool.value {
                    Ok(Self::Inherit)
                } else {
                    Ok(Self::Ignore)
                }
            }
            _ => Err(syn::Error::new(lit.span(), "invalid ops operation type")),
        }
    }

    __help_impl_ops_operation!(non-assign impl_add, +);
    __help_impl_ops_operation!(non-assign impl_sub, -);
    __help_impl_ops_operation!(non-assign impl_mul, *);
    __help_impl_ops_operation!(non-assign impl_div, /);
    __help_impl_ops_operation!(assign impl_add_assign, +=);
    __help_impl_ops_operation!(assign impl_sub_assign, -=);
    __help_impl_ops_operation!(assign impl_mul_assign, *=);
    __help_impl_ops_operation!(assign impl_div_assign, /=);

    fn _impl_ops(&self, ident: &Ident, op_ident: impl ToTokens) -> syn::Result<TokenStream2> {
        match self {
            Self::Ignore => Ok(quote! { self.#ident }),
            Self::Inherit => Ok(quote! { self.#ident #op_ident rhs.#ident }),
            Self::Manual(s) => syn::parse_str(&s.replace("$self", "self").replace("$rhs", "rhs")),
        }
    }

    fn _impl_ops_assign(
        &self,
        ident: &Ident,
        op_ident: impl ToTokens,
    ) -> syn::Result<TokenStream2> {
        match self {
            Self::Ignore => Ok(quote! {}),
            Self::Inherit => Ok(quote! { self.#ident #op_ident rhs.#ident }),
            Self::Manual(s) => {
                let token: Expr =
                    syn::parse_str(&s.replace("$self", "self").replace("$rhs", "rhs"))?;
                Ok(quote! {
                    self.#ident = #token
                })
            }
        }
    }
}
