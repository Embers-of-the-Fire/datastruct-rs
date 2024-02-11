use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, token, Attribute, Expr, Generics, Ident, Token, Type, Visibility};

#[derive(Clone)]
pub struct RichStruct {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    _struct_token: Token![struct],
    pub ident: Ident,
    pub generics: Generics,
    _brace_token: token::Brace,
    pub fields: Punctuated<StructField, Token![,]>,
}

impl RichStruct {
    pub fn parse_struct(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(RichStruct {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            _struct_token: input.parse()?,
            ident: input.parse()?,
            generics: input.parse()?,
            _brace_token: braced!(content in input),
            fields: content.parse_terminated(StructField::parse_field)?,
        })
    }
}

#[derive(Clone)]
pub struct StructField {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    _colon: Token![:],
    pub field_type: Type,
    pub default_value: Option<FieldDefaultValue>,
}

impl StructField {
    fn parse_field(input: ParseStream) -> syn::Result<Self> {
        Ok(StructField {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            ident: input.parse()?,
            _colon: input.parse()?,
            field_type: input.parse()?,
            default_value: if input.peek(Token![=]) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}

#[derive(Clone)]
pub struct FieldDefaultValue {
    _eq: Token![=],
    pub value: Expr,
}

impl Parse for FieldDefaultValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::parse_value(input)
    }
}

impl FieldDefaultValue {
    fn parse_value(input: ParseStream) -> syn::Result<Self> {
        Ok(FieldDefaultValue {
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}
