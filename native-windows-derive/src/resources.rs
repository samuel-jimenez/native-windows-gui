use quote::ToTokens;
use syn::{Attribute, Expr, Field, Ident, Result};

use crate::shared::{Parameters, parameters, parse_type_attr, parse_type_clone};

pub(crate) struct NwdResource<'a> {
    id: &'a Ident,
    ty: Ident,
    names: Vec<Ident>,
    values: Vec<Expr>,
}

impl<'a> NwdResource<'a> {
    pub fn parse(field: &'a Field) -> Result<Self> {
        let id = field.ident.as_ref().unwrap();
        let ty = NwdResource::parse_type(field)?;
        let (names, values) = parameters(field, NwdResource::find_attr)?;

        Ok(Self {
            id,
            ty,
            names,
            values,
        })
    }

    fn find_attr(attr: &&Attribute) -> bool {
        attr.path().is_ident("nwg_resource")
    }

    fn valid_attr(attr: &Attribute) -> bool {
        Self::find_attr(&attr)
    }

    pub fn valid(field: &Field) -> bool {
        field.attrs.iter().any(Self::valid_attr)
    }

    fn parse_type(field: &Field) -> Result<Ident> {
        // Check for `ty` in nwg_resource
        let attr = match field.attrs.iter().find(Self::find_attr) {
            Some(attr) => attr,
            None => unreachable!(),
        };
        parse_type_attr(Parameters::parse_attr(attr)?).map_or_else(
            ||          // Use field type
            parse_type_clone(&field, &attr.path().get_ident().unwrap()),
            Ok,
        )
    }
}

impl<'b> ToTokens for NwdResource<'b> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let ty = &self.ty;
        let member = self.id;
        let names = &self.names;
        let values = &self.values;
        let resource_tk = quote! {
            #ty::builder()
                #(.#names(#values))*
                .build(&mut data.#member)?;
        };

        resource_tk.to_tokens(tokens);
    }
}
