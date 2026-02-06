use syn::{Attribute, Error, Expr, Field, Ident, Result};

use crate::shared::Parameters;

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
