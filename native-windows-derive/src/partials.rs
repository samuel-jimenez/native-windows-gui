use quote::ToTokens;
use syn::{Attribute, Expr, Field, Ident, Result};

use crate::{
    layouts::LayoutChild,
    shared::{Parameters, parse_type_borrow},
};

pub(crate) struct NwdPartial<'a> {
    pub id: &'a Ident,
    ty: &'a Ident,
    pub parent: Option<Ident>,
    pub layout: Option<LayoutChild>,
    pub layout_index: usize,
    pub weight: [u16; 2],
    nested: bool,
    pub as_layout_p: bool,
}

impl<'a> NwdPartial<'a> {
    pub fn parse(field: &'a Field, field_pos: u16, nested: bool) -> Result<Self> {
        let (parent, as_layout_p) = NwdPartial::parse_attrs(field)?;

        Ok(Self {
            id: field.ident.as_ref().unwrap(),
            ty: NwdPartial::parse_type(field)?,
            parent,
            layout: LayoutChild::prepare(field)?,
            layout_index: 0,
            weight: [0, field_pos as u16],
            nested,
            as_layout_p,
        })
    }

    fn find_attr(attr: &&Attribute) -> bool {
        attr.path().is_ident("nwg_partial") || attr.path().is_ident("nwg_partial_control")
    }

    fn valid_attr(attr: &Attribute) -> bool {
        Self::find_attr(&attr)
    }

    pub fn valid(field: &Field) -> bool {
        field.attrs.iter().any(Self::valid_attr)
    }

    fn parse_type(field: &Field) -> Result<&Ident> {
        parse_type_borrow(&field, "nwg_partial")
    }

    fn parse_attrs(field: &Field) -> Result<(Option<Ident>, bool)> {
        let attr = match field.attrs.iter().find(Self::find_attr) {
            Some(attr) => attr,
            None => unreachable!(),
        };
        // #[nwg_partial_control]
        let as_layout_p = attr.path().is_ident("nwg_partial");

        let params = Parameters::parse_attr(attr)?.params;

        // #[nwg_partial(be: control)]
        let as_layout_p = as_layout_p
            && match params.iter().find(|p| p.ident == "be").map(|p| &p.e) {
                Some(v) => match v {
                    Expr::Path(p) => p.path.segments.last().map(|seg| seg.ident.to_string()),
                    _ => None,
                },
                None => None,
            }
            .is_none_or(|x| x == "layout");

        let parent = match params.iter().find(|p| p.ident == "parent").map(|p| &p.e) {
            Some(v) => match v {
                Expr::Path(p) => p.path.segments.last().map(|seg| seg.ident.clone()),
                _ => None,
            },
            None => None,
        };
        Ok((parent, as_layout_p))
    }
}

impl<'b> ToTokens for NwdPartial<'b> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let ty = &self.ty;
        let id = &self.id;
        let parent = &self.parent;
        let nested = &self.nested;
        let expand_layout_p = self.as_layout_p && self.layout.is_some();

        let partial_tk = if parent.is_none() {
            if !nested {
                quote! {
                    #ty::build_partial::<&Window>(&mut data.#id, None, #expand_layout_p)?;
                }
            } else {
                quote! {
                    #ty::build_partial(&mut data.#id, Some(parent_ref.unwrap()), #expand_layout_p)?;
                }
            }
        } else {
            quote! {
                #ty::build_partial(&mut data.#id, Some(&data.#parent), #expand_layout_p)?;
            }
        };

        partial_tk.to_tokens(tokens);
    }
}
