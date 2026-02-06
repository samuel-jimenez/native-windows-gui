use itertools::Itertools;
use quote::ToTokens;
use syn::{Error, Expr, Field, Ident, Result};

use crate::{
    events::ControlEvents,
    layouts::{FlexboxLayoutChild, GridLayoutChild, LayoutChild, layout_parameters},
    shared::Parameters,
};

const TOP_LEVEL: &'static [&'static str] = &["Window", "MessageWindow", "ExternCanvas"];

const AUTO_PARENT: &'static [&'static str] = &[
    "Window",
    "TabsContainer",
    "Tab",
    "MessageWindow",
    "ExternCanvas",
];

const AUTO_TAB_PARENT: &'static [&'static str] = &["TabsContainer"];

const SUB_CONTROL: &'static [&'static str] = &["LabeledEdit", "LabeledCombo"];

fn parse_type_attr(parameters: Parameters) -> Option<Ident> {
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

fn parse_type_borrow<'a>(field: &'a Field, ident: &str) -> Result<&'a Ident> {
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

fn parse_type_clone(field: &Field, ident: &Ident) -> Result<Ident> {
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

struct NwgControl<'a> {
    id: &'a Ident,
    parent_id: Option<String>,

    ty: Ident,

    layout: Option<LayoutChild>,
    layout_index: usize,

    names: Vec<Ident>,
    values: Vec<Expr>,

    // First value if the parent order, second value is the insert order
    weight: [u16; 2],

    // Contains sub controls?
    nested: bool,
}

impl<'a> NwgControl<'a> {
    fn parse(field: &'a Field, field_pos: u16) -> Result<Self> {
        let id = field.ident.as_ref().unwrap();
        let (ty, nested) = NwgControl::parse_attrs(field)?;
        let (names, values) = crate::controls::parameters(field, NwgControl::find_attr)?;
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

    fn get_root_info(&self) -> (Option<&'a Ident>, Option<Ident>) {
        (Some(self.id), Some(self.ty.clone()))
    }

    fn find_attr(attr: &&syn::Attribute) -> bool {
        attr.path().is_ident("nwg_control") || attr.path().is_ident("nwg_control_layout")
    }

    fn valid_attr(attr: &syn::Attribute) -> bool {
        Self::find_attr(&attr)
    }

    fn valid(field: &Field) -> bool {
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

    fn is_root(field: &Field) -> bool {
        field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("nwg_root"))
    }

    fn expand_flags(&mut self) -> Result<()> {
        let flags_index = self.names.iter().position(|n| n == "flags");
        if let Some(i) = flags_index {
            let old_flags = self.values[i].clone();
            self.values[i] = crate::controls::expand_flags(&self.id, &self.ty, old_flags)?;
        }
        Ok(())
    }

    fn expand_parent(&mut self) {
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

struct NwgResource<'a> {
    id: &'a Ident,
    ty: Ident,
    names: Vec<Ident>,
    values: Vec<Expr>,
}

impl<'a> NwgResource<'a> {
    fn parse(field: &'a Field) -> Result<Self> {
        let id = field.ident.as_ref().unwrap();
        let ty = NwgResource::parse_type(field)?;
        let (names, values) = crate::controls::parameters(field, NwgResource::find_attr)?;

        Ok(Self {
            id,
            ty,
            names,
            values,
        })
    }

    fn find_attr(attr: &&syn::Attribute) -> bool {
        attr.path().is_ident("nwg_resource")
    }

    fn valid_attr(attr: &syn::Attribute) -> bool {
        Self::find_attr(&attr)
    }

    fn valid(field: &Field) -> bool {
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

#[derive(Debug)]
struct NwgLayout<'a> {
    id: &'a Ident,
    ty: &'a Ident,

    layout: Option<LayoutChild>,
    layout_index: usize,

    names: Vec<Ident>,
    values: Vec<Expr>,
    weight: [u16; 2],
    sublayout: bool,
}

impl<'a> NwgLayout<'a> {
    fn parse(field: &'a Field, field_pos: u16, sublayout: bool) -> Result<Self> {
        let id = field.ident.as_ref().unwrap();
        let ty = NwgLayout::parse_type(field)?;
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

    fn find_attr(attr: &&syn::Attribute) -> bool {
        attr.path().is_ident("nwg_layout")
    }

    fn valid_attr(attr: &syn::Attribute) -> bool {
        Self::find_attr(&attr)
    }

    fn valid(field: &Field) -> bool {
        field.attrs.iter().any(Self::valid_attr)
    }

    fn parse_type(field: &Field) -> Result<&Ident> {
        // TODO: extract type from nwg_layout first
        parse_type_borrow(&field, "nwg_layout")
    }

    fn expand_parent(&mut self) {
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

struct NwgPartial<'a> {
    id: &'a Ident,
    ty: &'a Ident,
    parent: Option<Ident>,
    layout: Option<LayoutChild>,
    layout_index: usize,
    weight: [u16; 2],
    nested: bool,
    as_layout_p: bool,
}

impl<'a> NwgPartial<'a> {
    fn parse(field: &'a Field, field_pos: u16, nested: bool) -> Result<Self> {
        let (parent, as_layout_p) = NwgPartial::parse_attrs(field)?;

        Ok(Self {
            id: field.ident.as_ref().unwrap(),
            ty: NwgPartial::parse_type(field)?,
            parent,
            layout: LayoutChild::prepare(field)?,
            layout_index: 0,
            weight: [0, field_pos as u16],
            nested,
            as_layout_p,
        })
    }

    fn find_attr(attr: &&syn::Attribute) -> bool {
        attr.path().is_ident("nwg_partial") || attr.path().is_ident("nwg_partial_control")
    }

    fn valid_attr(attr: &syn::Attribute) -> bool {
        Self::find_attr(&attr)
    }

    fn valid(field: &Field) -> bool {
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

pub struct NwgUiControls<'a>(&'a NwgUi<'a>);

impl<'a> ToTokens for NwgUiControls<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        struct ControlGen<'b> {
            item: &'b NwgControl<'b>,
        }

        impl<'b> ToTokens for ControlGen<'b> {
            fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
                let item = &self.item;
                let ty = &item.ty;
                let member = item.id;
                let names = &item.names;
                let values = &item.values;
                let control_tk = quote! {
                    #ty::builder()
                        #(.#names(#values))*
                        .build(&mut data.#member)?;
                };

                control_tk.to_tokens(tokens);
            }
        }

        let ui = &self.0;
        let controls: Vec<ControlGen> =
            ui.controls.iter().map(|item| ControlGen { item }).collect();

        let controls_tk = quote! {
            #(#controls)*
        };

        controls_tk.to_tokens(tokens);
    }
}

pub struct NwgUiResources<'a>(&'a NwgUi<'a>);

impl<'a> ToTokens for NwgUiResources<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        struct ResourceGen<'b> {
            item: &'b NwgResource<'b>,
        }

        impl<'b> ToTokens for ResourceGen<'b> {
            fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
                let item = &self.item;
                let ty = &item.ty;
                let member = item.id;
                let names = &item.names;
                let values = &item.values;
                let resource_tk = quote! {
                    #ty::builder()
                        #(.#names(#values))*
                        .build(&mut data.#member)?;
                };

                resource_tk.to_tokens(tokens);
            }
        }

        let ui = &self.0;
        let resources: Vec<ResourceGen> = ui
            .resources
            .iter()
            .map(|item| ResourceGen { item })
            .collect();

        let resources_tk = quote! {
            #(#resources)*
        };

        resources_tk.to_tokens(tokens);
    }
}

pub struct NwgUiEvents<'a>(&'a NwgUi<'a>);

impl<'a> ToTokens for NwgUiEvents<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        self.0.events.to_tokens(tokens);
    }
}

pub struct NwgUiLayouts<'a>(&'a NwgUi<'a>);

impl<'a> ToTokens for NwgUiLayouts<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        enum ControlLayout<'b> {
            Control(&'b NwgControl<'b>),
            Layout(&'b NwgLayout<'b>),
            Partial(&'b NwgPartial<'b>),
        }
        impl<'b> ControlLayout<'b> {
            fn weight(&self) -> u16 {
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
                let param_name = if nested || layout_item {
                    quote! {child_layout}
                } else {
                    quote! {child}
                };

                let item_tk = match layout {
                    Some(LayoutChild::Grid(GridLayoutChild {
                        col,
                        row,
                        col_span,
                        row_span,
                    })) => {
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
                    Some(LayoutChild::Flexbox(FlexboxLayoutChild {
                        param_names,
                        param_values,
                    })) => quote! {
                        #param_name(&ui.#id)
                        #(.#param_names(#param_values))*
                    },
                    Some(LayoutChild::Init { field_name, .. }) => Error::new_spanned(
                        field_name,
                        format!(
                            "Unmatched layout item for field \"{}\", Did you forget the `layout` parameter?",
                            field_name
                        ),
                    ).into_compile_error(),
                    None => panic!("Unfiltered layout item"),
                };

                item_tk.to_tokens(tokens);
            }
        }

        struct LayoutGen<'b> {
            layout: &'b NwgLayout<'b>,
            children: Vec<ControlLayout<'b>>,
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

        let ui = &self.0;
        let layouts: Vec<LayoutGen> = ui
            .layouts
            .iter()
            .enumerate()
            .map(|(i, layout)| LayoutGen {
                layout,
                children: ui
                    .controls
                    .iter()
                    .filter(|c| c.layout.is_some() && c.layout_index == i)
                    .map(|c| ControlLayout::Control(c))
                    .chain(
                        ui.layouts
                            .iter()
                            .filter(|c| c.layout.is_some() && c.layout_index == i)
                            .map(|c| ControlLayout::Layout(c)),
                    )
                    .chain(
                        ui.partials
                            .iter()
                            .filter(|c| c.layout.is_some() && c.layout_index == i)
                            .map(|c| ControlLayout::Partial(c)),
                    )
                    .sorted_by(|a, b| a.weight().cmp(&b.weight()))
                    .collect(),
            })
            .collect();

        let layouts_tk = quote! {
            #(#layouts)*
        };
        // Control(&'b NwgControl<'b>),
        //
        layouts_tk.to_tokens(tokens);
    }
}

pub struct NwgUiPartials<'a>(&'a NwgUi<'a>);

impl<'a> ToTokens for NwgUiPartials<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        struct PartialGen<'b> {
            item: &'b NwgPartial<'b>,
        }

        impl<'b> ToTokens for PartialGen<'b> {
            fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
                let i = &self.item;
                let ty = &i.ty;
                let id = &i.id;
                let parent = &i.parent;
                let nested = &i.nested;
                let expand_layout_p = i.as_layout_p && i.layout.is_some();

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

        let ui = &self.0;
        let partials: Vec<PartialGen> =
            ui.partials.iter().map(|item| PartialGen { item }).collect();

        let partials_tk = quote! {
            #(#partials)*
        };

        partials_tk.to_tokens(tokens);
    }
}

pub struct NwgUi<'a> {
    controls: Vec<NwgControl<'a>>,
    resources: Vec<NwgResource<'a>>,
    layouts: Vec<NwgLayout<'a>>,
    partials: Vec<NwgPartial<'a>>,
    events: ControlEvents,
    root_id: Option<&'a Ident>,
    root_type: Option<Ident>,
}

impl<'a> NwgUi<'a> {
    pub fn build(data: &'a syn::DataStruct, partial_p: bool) -> Result<NwgUi<'a>> {
        let named_fields = match &data.fields {
            syn::Fields::Named(n) => &n.named,
            _ => panic!("Ui structure must have named fields"),
        };

        let mut controls = Vec::with_capacity(named_fields.len());
        let mut resources = Vec::with_capacity(named_fields.len());
        let mut layouts = Vec::with_capacity(named_fields.len());
        let mut partials = Vec::with_capacity(named_fields.len());
        let mut events = ControlEvents::with_capacity(partial_p, named_fields.len());

        let mut root_id: Option<&Ident> = None;
        let mut root_type: Option<Ident> = None;
        let partial_parent_expr: Expr = syn::parse_str("parent_ref.unwrap()").unwrap();
        let parent_ident = Ident::new("parent", pm2::Span::call_site());

        // First pass: parse controls, layouts, and events
        for (field_pos, field) in named_fields.iter().enumerate() {
            if NwgControl::valid(field) {
                let control = NwgControl::parse(field, field_pos as u16)?;
                if root_id.is_none() && NwgControl::is_root(field) {
                    (root_id, root_type) = control.get_root_info();
                }

                events.add_top_level_handle(field);
                events.parse(field)?;

                controls.push(control);
            }

            if NwgResource::valid(field) {
                let resource = NwgResource::parse(field)?;

                resources.push(resource);
            } else if NwgLayout::valid(field) {
                let layout = NwgLayout::parse(field, field_pos as u16, partial_p)?;

                // Reorder layouts
                match layouts.iter().position(|control: &NwgLayout| {
                    control
                        .layout
                        .as_ref()
                        .is_some_and(|child_layout| child_layout.parent_matches(layout.id))
                }) {
                    Some(index) => {
                        layouts.insert(index, layout);
                    }
                    None => {
                        layouts.push(layout);
                    }
                }
            } else if NwgPartial::valid(field) {
                let partial = NwgPartial::parse(field, field_pos as u16, partial_p)?;

                events.add_partial(&partial.id);
                events.parse(field)?;

                partials.push(partial);
            }
        }

        layouts.reverse();

        // Parent stuff
        for i in 0..(layouts.len()) {
            // Add the parent value of the layout object if it was not already defined
            let has_attr_parent = layouts[i].names.iter().any(|n| n == "parent");
            if has_attr_parent {
                layouts[i].expand_parent();
            } else {
                if root_id.is_some() {
                    let parent_expr: Expr =
                        syn::parse_str(&format!("&ui.{}", root_id.unwrap())).unwrap();
                    layouts[i].names.push(parent_ident.clone());
                    layouts[i].values.push(parent_expr);
                } else if partial_p {
                    layouts[i].names.push(parent_ident.clone());
                    layouts[i].values.push(partial_parent_expr.clone());
                } else {
                    return Err(Error::new_spanned(
                        named_fields.iter().nth(layouts[i].weight[1].into()),
                        "auto detection of layout parent outside of partial is not yet implemented",
                    )); //TODO detect
                }
            }

            // Match the layout item to the layout object
            let layout_id = layouts[i].id;
            let layout_type = layouts[i].ty;
            macro_rules! expand_layout_parent {
                ( $vec:ident ) => {
                    for control in $vec.iter_mut() {
                        if let Some(child_layout) = control.layout.as_mut() {
                            if child_layout.parent_matches(layout_id) {
                                child_layout.parse(layout_type);
                                control.layout_index = i;
                            }
                        }
                    }
                };
            }
            expand_layout_parent!(controls);
            expand_layout_parent!(layouts);
            expand_layout_parent!(partials);
        }

        for i in 0..(controls.len()) {
            let top_level = TOP_LEVEL.iter().any(|top| &controls[i].ty == top);
            if top_level {
                continue;
            }

            let has_attr_parent = controls[i].names.iter().any(|n| n == "parent");
            if has_attr_parent {
                controls[i].expand_parent();
            } else {
                // Tab requires a TabsContainer parent.
                let auto_parent = if controls[i].ty == "Tab" {
                    AUTO_TAB_PARENT
                } else {
                    AUTO_PARENT
                };

                // Rewind the controls set the parent to the nearest control that supports children
                let parent = controls[0..i]
                    .iter()
                    .rev()
                    .find(|i| auto_parent.iter().any(|top| i.ty == top));

                if let Some(parent) = parent {
                    let parent_id = Some(parent.id.to_string());
                    let parent_expr: Expr =
                        syn::parse_str(&format!("&data.{}", parent.id)).unwrap();
                    controls[i].names.push(parent_ident.clone());
                    controls[i].values.push(parent_expr);
                    controls[i].parent_id = parent_id;
                } else if root_id.is_some_and(|x| x != controls[i].id) {
                    let parent_id = Some(root_id.unwrap().to_string());
                    let parent_expr: Expr =
                        syn::parse_str(&format!("&data.{}", parent_id.as_ref().unwrap())).unwrap();
                    controls[i].names.push(parent_ident.clone());
                    controls[i].values.push(parent_expr);
                    controls[i].parent_id = parent_id;
                } else if partial_p {
                    // If no parent is found, but we are in a partial, use the partial parent.
                    controls[i].names.push(parent_ident.clone());
                    controls[i].values.push(partial_parent_expr.clone());
                    controls[i].parent_id = Some(parent_ident.to_string());
                }
            }
        }

        if root_id.is_some() {
            for control in partials.iter_mut() {
                if control.parent.is_none() {
                    control.parent = root_id.cloned()
                }
            }
        }

        // Parent Weight
        fn compute_weight(controls: &[NwgControl], index: usize, weight: &mut [u16; 2]) {
            match &controls[index].parent_id {
                Some(p) => {
                    if let Some(parent_index) = controls.iter().position(|c| &c.id == &p) {
                        compute_weight(controls, parent_index, weight);
                        weight[0] += 1;
                    }
                }
                None => {}
            }
        }

        for i in 0..(controls.len()) {
            let mut weight = controls[i].weight;
            compute_weight(&controls, i, &mut weight);
            controls[i].weight = weight;
        }

        // Helpers
        for control in controls.iter_mut() {
            control.expand_flags()?;
        }

        // Sort by weight
        controls.sort_unstable_by(|a, b| {
            let a = ((a.weight[0] as u32) << 16) + (a.weight[1] as u32);
            let b = ((b.weight[0] as u32) << 16) + (b.weight[1] as u32);
            a.cmp(&b)
        });

        Ok(NwgUi {
            controls,
            resources,
            layouts,
            partials,
            events,
            root_id,
            root_type,
        })
    }

    pub fn controls(&self) -> NwgUiControls<'_> {
        NwgUiControls(self)
    }

    pub fn resources(&self) -> NwgUiResources<'_> {
        NwgUiResources(self)
    }

    pub fn events(&self) -> NwgUiEvents<'_> {
        NwgUiEvents(self)
    }

    pub fn layouts(&self) -> NwgUiLayouts<'_> {
        NwgUiLayouts(self)
    }

    pub fn partials(&self) -> NwgUiPartials<'_> {
        NwgUiPartials(self)
    }

    pub fn root_element(&self) -> (Option<Ident>, Option<Ident>) {
        (self.root_id.cloned(), self.root_type.clone())
    }
}
