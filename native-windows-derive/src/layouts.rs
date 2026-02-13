use quote::ToTokens;
use syn::{Attribute, Error, Expr, Field, Ident, Result};

use crate::{
    controls::NwdControl,
    partials::NwdPartial,
    shared::{Parameters, parse_type_borrow},
};

#[derive(Clone, Copy, Debug)]
pub struct GridLayoutChild {
    pub col: u32,
    pub row: u32,
    pub col_span: u32,
    pub row_span: u32,
}

#[derive(Clone, Debug)]
pub struct FlexboxLayoutChild {
    pub param_names: Vec<Ident>,
    pub param_values: Vec<Expr>,
}

#[derive(Debug)]
pub enum LayoutChild {
    Init {
        field_name: String,
        params: Parameters,
    },
    Grid(GridLayoutChild),
    Flexbox(FlexboxLayoutChild),
}

impl LayoutChild {
    pub fn init(field_name: String, attr: &Attribute) -> Result<Self> {
        Ok(Self::Init {
            field_name,
            params: Parameters::parse_attr(attr)?,
        })
    }

    pub fn prepare(field: &Field) -> Result<Option<LayoutChild>> {
        let field_name = field
            .ident
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or("Unnamed".to_string());

        field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("nwg_layout_item"))
            .map(|attr| LayoutChild::init(field_name, attr))
            .transpose()
    }

    pub fn parse(&mut self, parent_type: &Ident) {
        if parent_type == "GridLayout" {
            *self = Self::parse_grid_layout_params(self);
        } else if parent_type == "FlexboxLayout" {
            *self = Self::parse_flexbox_layout_params(self);
        } else {
            panic!("Unknown parent type: {:?}", parent_type);
        }
    }

    pub fn parent_matches(&self, parent: &Ident) -> bool {
        match self {
            LayoutChild::Init { params: p, .. } => p
                .params
                .iter()
                .filter(|p| p.ident == "layout")
                .any(|p| match &p.e {
                    Expr::Path(exp_path) => exp_path
                        .path
                        .segments
                        .last()
                        .map(|seg| &seg.ident == parent)
                        .unwrap_or(false),
                    _ => false,
                }),
            _ => {
                false
                // panic!( "Tried to match control to layout, but `parent_matches` was called on {:?}. It should be an `LayoutChild::Init` value",self)
            }
        }
    }

    fn parse_grid_layout_params(child: &mut LayoutChild) -> LayoutChild {
        let [mut col, mut row, mut col_span, mut row_span] = [0, 0, 1, 1];

        match child {
            LayoutChild::Init { params: p, .. } => {
                for p in p.params.iter() {
                    let attr_name = p.ident.to_string();
                    match &attr_name as &str {
                        "col" => col = Self::int_value(&p.e),
                        "row" => row = Self::int_value(&p.e),
                        "col_span" => col_span = Self::int_value(&p.e),
                        "row_span" => row_span = Self::int_value(&p.e),
                        _ => {}
                    }
                }
            }
            _ => panic!("Called parse on a non-Init child layout"),
        };

        LayoutChild::Grid(GridLayoutChild {
            col,
            col_span,
            row,
            row_span,
        })
    }

    fn parse_flexbox_layout_params(child: &mut LayoutChild) -> LayoutChild {
        let mut param_names = Vec::with_capacity(4);
        let mut param_values = Vec::with_capacity(4);

        match child {
            LayoutChild::Init { params: p, .. } => {
                for p in p.params.iter() {
                    if &p.ident == "layout" {
                        continue;
                    }

                    let child_name = format!("child_{}", &p.ident);
                    param_names.push(Ident::new(&child_name, p.ident.span()));
                    param_values.push(p.e.clone());
                }
            }
            _ => panic!("Called parse on a non-Init child layout"),
        }

        LayoutChild::Flexbox(FlexboxLayoutChild {
            param_names,
            param_values,
        })
    }

    fn int_value(expr: &Expr) -> u32 {
        match expr {
            Expr::Lit(lit) => match &lit.lit {
                syn::Lit::Int(i) => i.base10_parse().unwrap(),
                _ => panic!("Layout item members must be int literal."),
            },
            _ => panic!("Layout item members must be int literal."),
        }
    }

    fn to_tokens(&self, id: &Ident, nested: bool, layout_item: bool) -> pm2::TokenStream {
        let param_name = if nested || layout_item {
            quote! {child_layout}
        } else {
            quote! {child}
        };

        match self {
                    LayoutChild::Grid(GridLayoutChild {
                        col,
                        row,
                        col_span,
                        row_span,
                    }) => {
                        if nested {
                            let field_col = col + col_span;
                            quote! {
                            child_item(GridLayoutItem::new(&ui.#id.label_handle(), #col, #row, #col_span, #row_span))
                            .child_item(GridLayoutItem::new(&ui.#id, #field_col, #row, #col_span, #row_span))
                            }
                        } else {
                            quote! {
                            child_item(GridLayoutItem::new(&ui.#id, #col, #row, #col_span, #row_span))
                            }
                        }
                    }
                    LayoutChild::Flexbox(FlexboxLayoutChild {
                        param_names,
                        param_values,
                    }) => quote! {
                        #param_name(&ui.#id)
                        #(.#param_names(#param_values))*
                    },
                    LayoutChild::Init { field_name, .. } => Error::new_spanned(
                        field_name,
                        format!(
                            "Unmatched layout item for field \"{}\", Did you forget the `layout` parameter?",
                            field_name
                        ),
                    ).into_compile_error(),
                }
    }
}

pub enum ControlLayout<'b> {
    Control(&'b NwdControl<'b>),
    Layout(&'b NwdLayout<'b>),
    Partial(&'b NwdPartial<'b>),
}
impl<'b> ControlLayout<'b> {
    pub fn weight(&self) -> u16 {
        match self {
            ControlLayout::Control(c) => c.weight[1],
            ControlLayout::Layout(c) => c.weight[1],
            ControlLayout::Partial(c) => c.weight[1],
        }
    }
}

impl<'b> ToTokens for ControlLayout<'b> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let (id, layout, nested, layout_item) = match self {
            ControlLayout::Control(c) => (c.id, &c.layout, c.nested, false),
            ControlLayout::Layout(c) => (c.id, &c.layout, false, true),
            ControlLayout::Partial(c) => (c.id, &c.layout, false, c.as_layout_p),
        };

        let item_tk = layout.as_ref().unwrap().to_tokens(id, nested, layout_item);

        item_tk.to_tokens(tokens);
    }
}

//
// Main layout
//

pub fn layout_parameters(field: &Field) -> Result<(Vec<Ident>, Vec<Expr>)> {
    let nwg_layout = |attr: &&Attribute| attr.path().is_ident("nwg_layout");

    let attr = match field.attrs.iter().find(nwg_layout) {
        Some(attr) => attr,
        None => unreachable!(),
    };

    let params = Parameters::parse_attr(attr)?.params;
    let mut names = Vec::with_capacity(params.len());
    let mut exprs = Vec::with_capacity(params.len());

    for p in params {
        names.push(p.ident);
        exprs.push(p.e);
    }

    Ok((names, exprs))
}

#[derive(Debug)]
pub(crate) struct NwdLayout<'a> {
    pub id: &'a Ident,
    pub ty: &'a Ident,

    pub layout: Option<LayoutChild>,
    pub layout_index: usize,

    pub names: Vec<Ident>,
    pub values: Vec<Expr>,
    pub weight: [u16; 2],
    pub sublayout: bool,
}

impl<'a> NwdLayout<'a> {
    pub fn parse(field: &'a Field, field_pos: u16, sublayout: bool) -> Result<Self> {
        let id = field.ident.as_ref().unwrap();
        let ty = NwdLayout::parse_type(field)?;
        let (names, values) = layout_parameters(field)?;

        Ok(Self {
            id,
            ty,
            layout: LayoutChild::prepare(field)?,
            layout_index: 0,
            names,
            values,
            weight: [0, field_pos as u16],
            sublayout,
        })
    }

    fn find_attr(attr: &&Attribute) -> bool {
        attr.path().is_ident("nwg_layout")
    }

    fn valid_attr(attr: &Attribute) -> bool {
        Self::find_attr(&attr)
    }

    pub fn valid(field: &Field) -> bool {
        field.attrs.iter().any(Self::valid_attr)
    }

    fn parse_type(field: &Field) -> Result<&Ident> {
        // TODO: extract type from nwg_layout first
        parse_type_borrow(&field, "nwg_layout")
    }

    pub fn expand_parent(&mut self) {
        let parent_index = self.names.iter().position(|n| n == "parent");
        if parent_index.is_none() {
            return;
        }

        let i = parent_index.unwrap();
        let parent_expr: Expr = match &self.values[i] {
            Expr::Path(p) => {
                let id = &p.path.segments.last().unwrap().ident;
                syn::parse_str(&format!("&ui.{}", id)).unwrap()
            }
            _ => {
                panic!("Bad expression type for parent of field {}", self.id);
            }
        };

        self.values[i] = parent_expr;
    }
}

pub struct LayoutGen<'b> {
    pub layout: &'b NwdLayout<'b>,
    pub children: Vec<ControlLayout<'b>>,
}

impl<'b> ToTokens for LayoutGen<'b> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let ty = &self.layout.ty;
        let id = &self.layout.id;
        let names = &self.layout.names;
        let values = &self.layout.values;
        let children = &self.children;

        let sublayout = self.layout.sublayout;
        let build = if self.layout.layout.is_some() {
            quote! {build_partial(&ui.#id)}
        } else if !sublayout {
            quote! {build(&ui.#id)}
        } else {
            quote! {build_conditional(&ui.#id, expand_layout_p)}
        };

        let layout_tk = quote! {
            #ty::builder()
            #(.#names(#values))*
            #(.#children)*
            .#build?;
        };
        layout_tk.to_tokens(tokens);
    }
}
