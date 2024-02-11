use itertools::Itertools;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{Lit, MetaList};

use crate::generate::RichStructContent;
use crate::utils::collect_meta::collect_meta_map;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StructCmpConfig {
    pub partial_eq: bool,
    pub eq: bool,
    pub partial_ord: bool,
    pub ord: bool,
}

impl StructCmpConfig {
    pub fn impl_cmp(syntax: &RichStructContent) -> syn::Result<TokenStream2> {
        let mut ts: TokenStream2 = TokenStream2::new();

        if syntax.config.cmp.partial_eq {
            ts.extend(Self::impl_partial_eq(syntax)?)
        }

        if syntax.config.cmp.eq {
            ts.extend(Self::impl_eq(syntax))
        }

        ts.extend(Self::impl_rich_ord(syntax)?);

        Ok(ts)
    }

    fn impl_partial_eq(syntax: &RichStructContent) -> syn::Result<TokenStream2> {
        let ident = &syntax.ident;
        let (impl_g, type_g, where_clause) = syntax.generics.split_for_impl();

        let equations = syntax
            .fields
            .iter()
            .filter(|s| s.config.cmp.eq)
            .map(|field| {
                let ident = &field.ident;
                quote_spanned! {
                    ident.span() => (self.#ident == rhs.#ident)
                }
            });

        Ok(quote! {
            impl #impl_g ::std::cmp::PartialEq for #ident #type_g #where_clause {
                fn eq(&self, rhs: &Self) -> bool {
                    #(#equations)&&*
                }
            }
        })
    }

    fn impl_eq(syntax: &RichStructContent) -> TokenStream2 {
        let ident = &syntax.ident;
        let (impl_g, type_g, where_clause) = syntax.generics.split_for_impl();
        quote! { impl #impl_g ::std::cmp::Eq for #ident #type_g #where_clause {} }
    }

    // if `ord` is implemented and `partial_ord` is required to implement
    // but there's no configuration about `partial_ord`,
    // then `partial_ord` will be simply `Some(ord)`.
    fn impl_rich_ord(syntax: &RichStructContent) -> syn::Result<TokenStream2> {
        let mut ts: TokenStream2 = TokenStream2::new();

        if syntax.config.cmp.ord {
            ts.extend(Self::impl_ord(syntax)?)
        }

        if syntax.config.cmp.ord
            && syntax.config.cmp.partial_ord
            && syntax
                .fields
                .iter()
                .all(|x| x.config.cmp.partial_ord.is_none())
        {
            let ident = &syntax.ident;
            let (impl_g, type_g, where_clause) = syntax.generics.split_for_impl();

            ts.extend(quote! {
                impl #impl_g ::std::cmp::PartialOrd for #ident #type_g #where_clause {
                    fn partial_cmp(&self, rhs: &Self) -> ::core::option::Option<::std::cmp::Ordering> {
                        ::core::option::Option::Some(self.cmp(rhs))
                    }
                }
            });
        } else if syntax.config.cmp.partial_ord {
            ts.extend(Self::impl_partial_ord(syntax)?)
        }

        Ok(ts)
    }

    fn impl_ord(syntax: &RichStructContent) -> syn::Result<TokenStream2> {
        let mut cmp_seq = syntax
            .fields
            .iter()
            .filter_map(|x| x.config.cmp.ord.map(|d| (x, d)))
            .sorted_by_key(|(_, x)| *x)
            .map(|(field, _)| {
                let ident = &field.ident;
                quote! {
                    self.#ident.cmp(&other.#ident)
                }
            })
            .peekable();

        let base = cmp_seq.next().ok_or_else(|| {
            syn::Error::new(
                syntax.ident.span(),
                "at least one field can be `ord`ed if you want to derive `cmp.ord`",
            )
        })?;

        let cmp = if cmp_seq.peek().is_none() {
            base
        } else {
            quote! {
                #base #(.then_with(|| #cmp_seq))*
            }
        };

        let (impl_g, type_g, where_clause) = syntax.generics.split_for_impl();
        let ident = &syntax.ident;

        Ok(quote! {
            impl #impl_g ::std::cmp::Ord for #ident #type_g #where_clause {
                fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                    #cmp
                }
            }
        })
    }

    fn impl_partial_ord(syntax: &RichStructContent) -> syn::Result<TokenStream2> {
        let mut cmp_seq = syntax
            .fields
            .iter()
            .filter_map(|x| x.config.cmp.partial_ord.map(|d| (x, d)))
            .sorted_by_key(|(_, x)| *x)
            .map(|(field, _)| {
                let ident = &field.ident;
                quote! {
                    self.#ident.partial_cmp(&other.#ident)
                }
            })
            .peekable();

        let base = cmp_seq.next().ok_or_else(|| {
            syn::Error::new(
                syntax.ident.span(),
                "at least one field can be `partial_ord`ed if you want to derive `cmp.partial_ord`",
            )
        })?;

        let par_cmp = if cmp_seq.peek().is_none() {
            base
        } else {
            quote! {
                #base
                    #(.and_then(|__gen_dparord|
                        #cmp_seq.map(|__gen_dparord_self| __gen_dparord.then(__gen_dparord_self)))
                    )*
            }
        };

        let (impl_g, type_g, where_clause) = syntax.generics.split_for_impl();
        let ident = &syntax.ident;

        Ok(quote! {
            impl #impl_g ::std::cmp::PartialOrd for #ident #type_g #where_clause {
                fn partial_cmp(&self, other: &Self) -> ::core::option::Option<::std::cmp::Ordering> {
                    #par_cmp
                }
            }
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FieldCmpConfig {
    pub eq: bool,
    pub ord: Option<isize>,
    pub partial_ord: Option<isize>,
}

impl Default for FieldCmpConfig {
    fn default() -> Self {
        Self {
            eq: true,
            ord: None,
            partial_ord: None,
        }
    }
}

impl FieldCmpConfig {
    pub fn from_meta(meta_list: &MetaList) -> syn::Result<Self> {
        let mut config: Self = Default::default();

        collect_meta_map(meta_list, |idx, k, v| {
            match k.to_string().as_str() {
                "eq" | "peq" => match v {
                    Some(Lit::Bool(lit)) => config.eq = lit.value,
                    Some(Lit::Str(lit)) => match lit.value().parse::<bool>() {
                        Ok(val) => config.eq = val,
                        Err(e) => {
                            return Err(syn::Error::new(
                                lit.span(),
                                format!("cannot parse `eq` value: {:?}", e),
                            ));
                        }
                    },
                    None => config.eq = true,
                    _ => {
                        return Err(syn::Error::new(
                            k.span(),
                            "invalid `eq` value, see the documentation for more information",
                        ));
                    }
                },
                "cmp" | "ord" => match v {
                    Some(Lit::Bool(lit)) => {
                        if lit.value {
                            config.ord = Some(idx as isize)
                        } else {
                            config.ord = None
                        }
                    }
                    Some(Lit::Str(lit)) => match lit.value().parse::<isize>() {
                        Ok(val) => config.ord = Some(val),
                        Err(e) => {
                            return Err(syn::Error::new(
                                lit.span(),
                                format!("cannot parse `cmp` value: {:?}", e),
                            ));
                        }
                    },
                    Some(Lit::Int(lit)) => config.ord = Some(lit.base10_parse()?),
                    None => config.ord = Some(idx as isize),
                    _ => {
                        return Err(syn::Error::new(
                            k.span(),
                            "invalid `cmp` value, see the documentation for more information",
                        ));
                    }
                },
                "partial_cmp" | "pcmp" | "partial_ord" | "pord" => match v {
                    Some(Lit::Bool(lit)) => {
                        if lit.value {
                            config.partial_ord = Some(idx as isize)
                        } else {
                            config.partial_ord = None
                        }
                    }
                    Some(Lit::Str(lit)) => match lit.value().parse::<isize>() {
                        Ok(val) => config.partial_ord = Some(val),
                        Err(e) => {
                            return Err(syn::Error::new(
                                lit.span(),
                                format!("cannot parse `partial_cmp` value: {:?}", e),
                            ));
                        }
                    },
                    Some(Lit::Int(lit)) => config.partial_ord = Some(lit.base10_parse()?),
                    None => config.partial_ord = Some(idx as isize),
                    _ => return Err(syn::Error::new(
                        k.span(),
                        "invalid `partial_cmp` value, see the documentation for more information",
                    )),
                },
                _ => {}
            };

            Ok(((), ()))
        })?;

        Ok(config)
    }
}
