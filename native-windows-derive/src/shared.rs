use syn::{
    Attribute, Error, Expr, Field, Ident, Result,
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

pub fn parameters(
    field: &Field,
    find_attr: fn(attr: &&Attribute) -> bool,
) -> Result<(Vec<Ident>, Vec<Expr>)> {
    let attr = match field.attrs.iter().find(find_attr) {
        Some(attr) => attr,
        None => unreachable!(),
    };

    let params = Parameters::parse_attr(attr)?.params;
    let mut names = Vec::with_capacity(params.len());
    let mut exprs = Vec::with_capacity(params.len());

    for p in params {
        if p.ident == "ty" || p.ident == "nested" {
            continue;
        }

        names.push(p.ident);
        exprs.push(p.e);
    }

    Ok((names, exprs))
}

pub(crate) fn parse_type_attr(parameters: Parameters) -> Option<Ident> {
    // Check for `ty` in attribute
    parameters
        .params
        .iter()
        .find(|p| p.ident == "ty")
        .map(|p| &p.e)
        .map(|e| match e {
            Expr::Path(p) => Some(p),
            _ => None,
        })
        .flatten()
        .map(|p| p.path.get_ident().clone())
        .flatten()
        .map(|ident| ident.clone())
}

pub(crate) fn parse_type_borrow<'a>(field: &'a Field, ident: &str) -> Result<&'a Ident> {
    // Use field type
    match &field.ty {
        syn::Type::Path(p) => Some(p),
        _ => None,
    }
    .map(|p| p.path.segments.last())
    .flatten()
    .ok_or(Error::new_spanned(
        field.ident.as_ref().unwrap(),
        format!(
            "Impossible to parse type for field {:?}. Try specifying it in the {} attribute.",
            field.ident, ident
        ),
    ))
    .map(|seg| &seg.ident)
}

pub(crate) fn parse_type_clone(field: &Field, ident: &Ident) -> Result<Ident> {
    // Use field type
    match &field.ty {
        syn::Type::Path(p) => Some(p),
        _ => None,
    }
    .map(|p| p.path.segments.last())
    .flatten()
    .ok_or(Error::new_spanned(
        field.ident.as_ref().unwrap(),
        format!(
            "Impossible to parse type for field {:?}. Try specifying it in the {} attribute.",
            field.ident, ident
        ),
    ))
    .map(|seg| seg.ident.clone())
}
