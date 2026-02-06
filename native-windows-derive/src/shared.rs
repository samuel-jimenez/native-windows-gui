use syn::{
    Attribute, Result,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

#[derive(Debug)]
pub struct Param {
    pub ident: syn::Ident,
    #[allow(dead_code)]
    pub sep: Token![:],
    pub e: syn::Expr,
}

impl Parse for Param {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Param {
            ident: input.parse()?,
            sep: input.parse()?,
            e: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct Parameters {
    pub params: Punctuated<Param, Token![,]>,
}

impl Parse for Parameters {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            params: Punctuated::<Param, Token![,]>::parse_terminated(&input)?,
        })
    }
}

impl Parameters {
    pub fn parse_attr(attr: &Attribute) -> Result<Self> {
        Ok(match attr.meta.require_list() {
            Ok(attr) => attr.parse_args()?,
            Err(_) => Self {
                params: Punctuated::new(),
            },
        })
    }
}
