use crate::config::field_config::{FieldConfig, GetterType, SetterType};
use crate::config::struct_config::StructConfig;
use crate::syntax::{RichStruct, StructField};

use crate::cmp::StructCmpConfig;
use itertools::{Either, Itertools};
use proc_macro2::{Literal, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Attribute, Generics, Ident, Type, Visibility};
use crate::ops::StructOpsConfig;

#[derive(Clone)]
pub struct RichStructContent {
    pub config: StructConfig,
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub fields: Vec<StructFieldContent>,
}

impl RichStructContent {
    pub fn from_syntax(syntax: RichStruct) -> syn::Result<Self> {
        let (config, attrs) = StructConfig::from_attribute(syntax.attrs, syntax.ident.span())?;
        let fields = syntax
            .fields
            .into_iter()
            .enumerate()
            .map(|(idx, field)| -> Result<_, syn::Error> {
                let content = StructFieldContent::from_syntax(
                    field,
                    config.override_auto_set,
                    config.override_auto_get,
                )?;
                let seq = content.config.init_seq.unwrap_or(idx as isize);
                Ok((content, seq))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .sorted_by_key(|(_, i)| *i)
            .map(|(content, _)| content)
            .collect();

        let val = Self {
            config,
            attrs,
            vis: syntax.vis,
            ident: syntax.ident,
            generics: syntax.generics,
            fields,
        };

        Ok(val)
    }

    fn can_impl_default(&self) -> bool {
        self.fields.iter().all(|f| f.config.default_value.is_some())
    }

    pub fn to_impl(&self) -> syn::Result<TokenStream2> {
        let impl_ = self.generate_impl();
        let default = if self.can_impl_default() && self.config.generate_default {
            self.impl_default()
        } else {
            Default::default()
        };
        let const_default = if self.can_impl_default() && self.config.const_default {
            self.impl_const_default()
        } else {
            Default::default()
        };
        let std_default = if self.can_impl_default() && self.config.impl_std_default {
            self.impl_std_default()
        } else {
            Default::default()
        };
        let debug_impl = if self.config.manual_debug {
            self.impl_debug()
        } else {
            Default::default()
        };
        let cmp_impl = StructCmpConfig::impl_cmp(self)?;
        let ops_impl = StructOpsConfig::impl_ops(self)?;

        Ok(quote! {
            #impl_

            #default

            #const_default

            #std_default

            #debug_impl

            #cmp_impl

            #ops_impl
        })
    }

    fn generate_impl(&self) -> TokenStream2 {
        let fns = self
            .fields
            .iter()
            .flat_map(|field| field.generate_impl_code());
        let ident = &self.ident;
        let (impl_g, type_g, where_clause) = self.generics.split_for_impl();

        let p_default = if self.config.partial_default {
            self.impl_partial_default()
        } else {
            Default::default()
        };

        quote! {
            impl #impl_g #ident #type_g #where_clause {
                #(#fns)*

                #p_default
            }
        }
    }

    fn impl_default_construct(&self) -> TokenStream2 {
        let stmt = self.fields.iter().map(|field| {
            let name = &field.ident;
            let ty = &field.field_type;
            // SAFETY: Caller-guaranteed
            let default_expr = field.config.default_value.as_ref().unwrap();
            quote_spanned! {
                default_expr.span() => let #name: #ty = #default_expr;
            }
        });

        let idents = self.fields.iter().map(|field| &field.ident);
        quote! {
            #(#stmt)*

            Self {
                #(#idents),*
            }
        }
    }

    // complete block
    // all fields must have default value
    fn impl_default(&self) -> TokenStream2 {
        let construct = self.impl_default_construct();
        let ident = &self.ident;
        let (impl_g, type_g, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_g ::datastruct::DataStruct for #ident #type_g #where_clause {
                fn data_default() -> Self {
                    #construct
                }
            }
        }
    }

    fn impl_std_default(&self) -> TokenStream2 {
        let construct = self.impl_default_construct();
        let ident = &self.ident;
        let (impl_g, type_g, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_g ::std::default::Default for #ident #type_g #where_clause {
                fn default() -> Self {
                    #construct
                }
            }
        }
    }

    fn impl_const_default(&self) -> TokenStream2 {
        let construct = self.impl_default_construct();
        let ident = &self.ident;
        let (impl_g, type_g, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_g ::datastruct::ConstDataStruct for #ident #type_g #where_clause {
                const DEFAULT: Self = {
                    #construct
                };
            }
        }
    }

    fn impl_partial_default(&self) -> TokenStream2 {
        let (default, non_default): (Vec<_>, Vec<_>) =
            self.fields
                .iter()
                .partition_map(|f| match &f.config.default_value {
                    None => Either::Right(f),
                    Some(d) => Either::Left((f, d)),
                });

        let non_default_impl = non_default.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.field_type;
            quote! {
                #ident: #ty
            }
        });

        let default_impl = default.iter().map(|(field, default_expr)| {
            let ident = &field.ident;
            let ty = &field.field_type;
            quote_spanned! {
                default_expr.span() => let #ident: #ty = #default_expr;
            }
        });

        let idents = self.fields.iter().map(|f| &f.ident);

        quote! {
            pub fn partial_default(#(#non_default_impl),*) -> Self {
                #(#default_impl)*

                Self {
                    #(#idents),*
                }
            }
        }
    }

    fn impl_debug(&self) -> TokenStream2 {
        let struct_name: Literal = Literal::string(&self.ident.to_string());
        let struct_ident = &self.ident;
        let fields = self
            .fields
            .iter()
            .filter(|field| !field.config.no_debug)
            .map(|field| {
                let field_ident = &field.ident;
                let field_string: Literal = Literal::string(&field.ident.to_string());
                quote! {
                    .field(#field_string, &self.#field_ident)
                }
            });

        let (impl_g, type_g, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_g ::std::fmt::Debug for #struct_ident #type_g #where_clause {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    f.debug_struct(#struct_name)
                        #(#fields)*
                        .finish()
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct StructFieldContent {
    pub config: FieldConfig,
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub field_type: Type,
}

impl StructFieldContent {
    pub fn from_syntax(syntax: StructField, set: SetterType, get: GetterType) -> syn::Result<Self> {
        let (mut config, attrs) = FieldConfig::from_attribute(syntax.attrs, set, get)?;

        if let (None, Some(t)) = (&config.default_value, syntax.default_value) {
            config.default_value = Some(t.value)
        }

        Ok(Self {
            config,
            attrs,
            vis: syntax.vis,
            ident: syntax.ident,
            field_type: syntax.field_type,
        })
    }

    fn generate_impl_code(&self) -> Vec<TokenStream2> {
        let mut code = Vec::with_capacity(4);
        code.extend(self.config.auto_get.to_code(
            &self.ident.to_string(),
            &self.field_type,
            &self.ident.span(),
        ));
        code.extend(self.config.auto_set.to_code(
            &self.ident.to_string(),
            &self.field_type,
            &self.ident.span(),
        ));

        if self.config.do_with {
            let func_ident = Ident::new(&format!("do_with_{}", self.ident), self.ident.span());
            let ident = &self.ident;
            let ty = &self.field_type;
            code.push(quote! {
                pub fn #func_ident(&mut self, func: impl FnOnce(&mut #ty)) {
                    func(&mut self.#ident);
                }
            });
        }

        if self.config.map {
            let func_ident = Ident::new(&format!("map_{}", self.ident), self.ident.span());
            let ident = &self.ident;
            let ty = &self.field_type;
            code.push(quote! {
                pub fn #func_ident(mut self, func: impl FnOnce(#ty) -> #ty) -> Self {
                    self.#ident = func(self.#ident);
                    self
                }
            });
        }

        code
    }
}
