use itertools::Itertools;
use quote::ToTokens;
use syn::{Attribute, Error, Expr, Ident, Result};

use crate::{
    controls::NwdControl,
    events::{ControlEventShortcuts, ControlEvents},
    layouts::{ControlLayout, LayoutGen, NwdLayout},
    partials::NwdPartial,
    resources::NwdResource,
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

pub struct NwgUi<'a> {
    controls: Vec<NwdControl<'a>>,
    resources: Vec<NwdResource<'a>>,
    layouts: Vec<NwdLayout<'a>>,
    partials: Vec<NwdPartial<'a>>,
    events: ControlEvents,
    root_id: Option<&'a Ident>,
    root_type: Option<Ident>,
}

impl<'a> NwgUi<'a> {
    pub fn build(
        data: &'a syn::DataStruct,
        attrs: Vec<&Attribute>,
        partial_p: bool,
    ) -> Result<NwgUi<'a>> {
        let named_fields = match &data.fields {
            syn::Fields::Named(n) => &n.named,
            _ => panic!("Ui structure must have named fields"),
        };

        let mut controls = Vec::with_capacity(named_fields.len());
        let mut resources = Vec::with_capacity(named_fields.len());
        let mut layouts = Vec::with_capacity(named_fields.len());
        let mut partials = Vec::with_capacity(named_fields.len());
        let mut events = ControlEvents::with_capacity(partial_p, named_fields.len());

        events.parse_global(attrs)?;

        let mut root_id: Option<&Ident> = None;
        let mut root_type: Option<Ident> = None;
        let partial_parent_expr: Expr = syn::parse_str("parent_ref.unwrap()").unwrap();
        let parent_ident = Ident::new("parent", pm2::Span::call_site());

        // First pass: parse controls, layouts, and events
        for (field_pos, field) in named_fields.iter().enumerate() {
            if NwdControl::valid(field) {
                let control = NwdControl::parse(field, field_pos as u16)?;
                if root_id.is_none() && NwdControl::is_root(field) {
                    (root_id, root_type) = control.get_root_info();
                }

                events.add_top_level_handle(field);
                events.parse(field)?;

                controls.push(control);
            }

            if NwdResource::valid(field) {
                let resource = NwdResource::parse(field)?;

                resources.push(resource);
            } else if NwdLayout::valid(field) {
                let layout = NwdLayout::parse(field, field_pos as u16, partial_p)?;

                // Reorder layouts
                match layouts.iter().position(|control: &NwdLayout| {
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
            } else if NwdPartial::valid(field) {
                let partial = NwdPartial::parse(field, field_pos as u16, partial_p)?;

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
        fn compute_weight(controls: &[NwdControl], index: usize, weight: &mut [u16; 2]) {
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

    pub fn shortcuts(&self) -> NwgUiShortcuts<'_> {
        NwgUiShortcuts(self)
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

pub struct NwgUiControls<'a>(&'a NwgUi<'a>);

impl<'a> ToTokens for NwgUiControls<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let controls = &self.0.controls;
        quote! {
            #(#controls)*
        }
        .to_tokens(tokens);
    }
}

pub struct NwgUiResources<'a>(&'a NwgUi<'a>);

impl<'a> ToTokens for NwgUiResources<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let resources = &self.0.resources;
        quote! {
            #(#resources)*
        }
        .to_tokens(tokens);
    }
}

pub struct NwgUiEvents<'a>(&'a NwgUi<'a>);

impl<'a> ToTokens for NwgUiEvents<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        self.0.events.to_tokens(tokens);
    }
}

pub struct NwgUiShortcuts<'a>(&'a NwgUi<'a>);

impl<'a> NwgUiShortcuts<'a> {
    pub fn len(&self) -> usize {
        self.0.events.shortcuts_len()
    }
}

impl<'a> ToTokens for NwgUiShortcuts<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        ControlEventShortcuts(&self.0.events).to_tokens(tokens);
    }
}

pub struct NwgUiLayouts<'a>(&'a NwgUi<'a>);

impl<'a> ToTokens for NwgUiLayouts<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
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
        layouts_tk.to_tokens(tokens);
    }
}

pub struct NwgUiPartials<'a>(&'a NwgUi<'a>);

impl<'a> NwgUiPartials<'a> {
    pub fn len(&self) -> usize {
        self.0.partials.len()
    }
}

impl<'a> ToTokens for NwgUiPartials<'a> {
    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let partials = &self.0.partials;

        quote! {
            #(#partials)*
        }
        .to_tokens(tokens);
    }
}
