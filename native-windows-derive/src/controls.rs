use quote::ToTokens;
use syn::{Attribute, Error, Expr, Field, Ident, Result};

use crate::{
    layouts::LayoutChild,
    shared::{Parameters, parameters, parse_type_attr, parse_type_clone},
};

const SUB_CONTROL: &'static [&'static str] = &["LabeledEdit", "LabeledCombo"];

pub fn expand_flags(member_name: &Ident, ty: &Ident, flags: Expr) -> Result<Expr> {
    let flags_type = format!("{}Flags", ty);

    let flags_value = match &flags {
        Expr::Lit(expr_lit) => Ok(&expr_lit.lit),
        _ => Err(()),
    }
    .map(|lit| match lit {
        syn::Lit::Str(value) => Ok(value),
        _ => Err(()),
    })
    .flatten()
    .map_err(|_| {
        Error::new_spanned(
            &flags,
            format!(
                "Compressed flags must be type `&str` for control `{}` ",
                member_name,
            ),
        )
    })?;

    let flags = flags_value.value();
    let splitted: Vec<&str> = flags.split('|').collect();

    let flags_count = splitted.len() - 1;
    let mut final_flags: String = String::with_capacity(100);
    for (i, value) in splitted.into_iter().enumerate() {
        final_flags.push_str(&flags_type);
        final_flags.push_str("::");
        final_flags.push_str(value);

        if i != flags_count {
            final_flags.push('|');
        }
    }

    syn::parse_str(&final_flags).map_err(|e| {
        Error::new_spanned(
            member_name,
            format!(
                "Failed to parse flags value for control {}: {}",
                member_name, e
            ),
        )
    })
}

pub(crate) struct NwdControl<'a> {
    pub(crate) id: &'a Ident,
    pub(crate) parent_id: Option<String>,

    pub(crate) ty: Ident,

    pub(crate) layout: Option<LayoutChild>,
    pub(crate) layout_index: usize,

    pub(crate) names: Vec<Ident>,
    pub(crate) values: Vec<Expr>,

    // First value if the parent order, second value is the insert order
    pub(crate) weight: [u16; 2],

    // Contains sub controls?
    pub(crate) nested: bool,
}

impl<'a> NwdControl<'a> {
    pub fn parse(field: &'a Field, field_pos: u16) -> Result<Self> {
        let id = field.ident.as_ref().unwrap();
        let (ty, nested) = NwdControl::parse_attrs(field)?;
        let (names, values) = parameters(field, NwdControl::find_attr)?;
        let nested = nested || SUB_CONTROL.iter().any(|nest| ty == nest);
        Ok(Self {
            id,
            parent_id: None,
            ty,
            layout: LayoutChild::prepare(field)?,
            layout_index: 0,
            names,
            values,
            weight: [0, field_pos as u16],
            nested,
        })
    }

    pub(crate) fn get_root_info(&self) -> (Option<&'a Ident>, Option<Ident>) {
        (Some(self.id), Some(self.ty.clone()))
    }

    fn find_attr(attr: &&Attribute) -> bool {
        attr.path().is_ident("nwg_control") || attr.path().is_ident("nwg_control_layout")
    }

    fn valid_attr(attr: &Attribute) -> bool {
        Self::find_attr(&attr)
    }

    pub(crate) fn valid(field: &Field) -> bool {
        field.attrs.iter().any(Self::valid_attr)
    }

    fn parse_attrs(field: &Field) -> Result<(Ident, bool)> {
        let attr = match field.attrs.iter().find(Self::find_attr) {
            Some(attr) => attr,
            None => unreachable!(),
        };

        let parameters = Parameters::parse_attr(attr)?;

        let nested = attr.path().is_ident("nwg_control_layout")
            || parameters.params.iter().any(|p| p.ident == "nested");

        // Check for `ty` in attribute
        parse_type_attr(parameters)
            .map_or_else(
                ||          // Use field type
            parse_type_clone(&field, &attr.path().get_ident().unwrap()),
                Ok,
            )
            .map(|ident| (ident, nested))
    }

    pub(crate) fn is_root(field: &Field) -> bool {
        field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("nwg_root"))
    }

    pub(crate) fn expand_flags(&mut self) -> Result<()> {
        let flags_index = self.names.iter().position(|n| n == "flags");
        if let Some(i) = flags_index {
            let old_flags = self.values[i].clone();
            self.values[i] = crate::controls::expand_flags(&self.id, &self.ty, old_flags)?;
        }
        Ok(())
    }

    pub(crate) fn expand_parent(&mut self) {
        let parent_index = self.names.iter().position(|n| n == "parent");
        if parent_index.is_none() {
            return;
        }

        let i = parent_index.unwrap();
        let parent_expr: Expr = match &self.values[i] {
            Expr::Path(p) => {
                let id = &p.path.segments.last().unwrap().ident;
                self.parent_id = Some(id.to_string());
                syn::parse_str(&format!("&data.{}", id)).unwrap()
            }
            _ => {
                panic!("Bad expression type for parent of field {}", self.id);
            }
        };

        self.values[i] = parent_expr;
    }
}

impl<'b> ToTokens for NwdControl<'b> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let ty = &self.ty;
        let member = self.id;
        let names = &self.names;
        let values = &self.values;
        let control_tk = quote! {
            #ty::builder()
                #(.#names(#values))*
                .build(&mut data.#member)?;
        };

        control_tk.to_tokens(tokens);
    }
}
