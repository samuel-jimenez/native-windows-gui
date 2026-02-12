extern crate proc_macro as pm;
extern crate proc_macro2 as pm2;

#[macro_use]
extern crate syn;
use pm2::Span;
use syn::{DeriveInput, Error, Ident};

#[macro_use]
extern crate quote;

use proc_macro_crate::{FoundCrate, crate_name};

mod controls;
mod events;
mod layouts;
mod shared;

mod ui;
use ui::NwgUi;

struct BaseNames {
    n_module: Ident,
    n_partial_module: Ident,
    n_struct: Ident,
    n_struct_ui: Ident,
}

fn to_snake_case(s: &str) -> String {
    let mut snake = String::with_capacity(s.len());

    for (i, c) in s.char_indices() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                snake.push('_');
            }
            snake.push_str(c.to_lowercase().to_string().as_ref());
        } else {
            snake.push(c);
        }
    }

    snake
}

fn parse_base_names(d: &DeriveInput) -> BaseNames {
    let base_name = d.ident.to_string();
    let module_name = format!("{}_ui", to_snake_case(&base_name));
    let partial_module = format!("partial_{}_ui", to_snake_case(&base_name));
    let struct_name = format!("{}Ui", &base_name);

    BaseNames {
        n_module: Ident::new(&module_name, Span::call_site()),
        n_partial_module: Ident::new(&partial_module, Span::call_site()),
        n_struct: Ident::new(&base_name, Span::call_site()),
        n_struct_ui: Ident::new(&struct_name, Span::call_site()),
    }
}

fn parse_ui_data(d: &DeriveInput) -> Option<&syn::DataStruct> {
    match &d.data {
        syn::Data::Struct(ds) => Some(ds),
        _ => None,
    }
}
fn find_attr(attr: &&syn::Attribute) -> bool {
    attr.path().is_ident("nwg_shortcuts")
}

fn fetch_attr(d: &DeriveInput) -> Option<&syn::Attribute> {
    d.attrs.iter().find(find_attr)
}

/**

The `NwgUi` macro implements the native-windows-gui `NativeUi` trait on the selected struct

For a detailed documentation of this macro see the documentation "native-windows-docs/nwd_basics.html"


# Usage

```rust
use native_windows_gui as nwg;

#[derive(NwgUi, Default)]
pub struct BasicApp {
    #[nwg_control(title: "Window")]
    #[nwg_events( OnWindowClose: [nwg::stop_thread_dispatch()] )]
    window: nwg::Window,

    #[nwg_resource(family: "Arial")]
    font: nwg::Font,

    #[nwg_layout(parent: window)]
    my_layout: nwg::GridLayout,

    #[nwg_control(text: "Button")]
    #[nwg_layout_item(layout: my_layout, col: 0, row: 0)]
    button: nwg::Button,
}

// ...

let my_ui = BasicAppUi::build_ui(Default::default()).unwrap();
```

The macro creates a new struct named `[StructName]Ui` in a submodule named `[struct_name]_ui`.

The trait `NativeUi` is implemented on this struct and the boilerplate code is generated for every field tagged by attributes.
Fields without attributes, even `nwg` types, are left untouched.

Finally, the derive macro also creates a default event handler that will live through the ui struct lifetime.


# Attributes usage

Actual UI creation works by tagging the struct fields with the some attributes

## Controls

Use the `nwg_control` attribute to instance a control from a struct field:

```
nwg_control(builder_field: builder_value,*)
```

This syntax is basically a compressed version of the nwg control builders. The control attribute
also has built-in helpers: auto parent detection and compressed flags syntax (see the docs for more info on these features).

```
#[nwg_control(text: "Heisenberg", size: (280, 25), position: (10, 10))]
name_edit: nwg::TextInput,

// is the same as

nwg::TextInput::builder()
    .text("Heisenberg")
    .size((280, 25))
    .position((10, 10))
    .build(&mut data.text_edit);
```

## Resources

Use the `nwg_resource` to generate a resource from a struct field. It works the exact same way as `nwg_controls`.
Resources are always instanced before the controls.

## Events

Use the `nwg_events` attribute to add events to the default event handler. Events can only be applied to a field that
was tagged with `nwg_control`.

```
nwg_events( EVENT_TYPE: [CALLBACK(ARGS),*] )
```

where:
 - **EVENT_TYPE** is any value of the Event enum.
 - **CALLBACK** is the function that will be called when the event is triggered.
 - **ARGS** specifies the parameters of the callback (optional).

## Events arguments

By default, native windows derive assumes the callback is a method of the Ui structure. So for example,
`TestApp::callback1` assumes the method has the following signature `callback1(&self)`.

That's very limiting. For example, if the same callback is used by two different controls, there's no way to differenciate them. In order to fix this, NWD lets you define the callbacks parameters using those identifiers:

 - **SELF**: Sends the ui struct `&UiStruct`. If there are no parameters, this is the default.
 - **RC_SELF**: Sends the rc ui struct `&Rc<UiStruct>`. Useful for binding dynamic events
 - **CTRL**: Sends the control that triggered the event. Ex: `&Button`
 - **HANDLE**: Sends the handle of the control. `&ControlHandle`
 - **EVT**: Sends the event that was triggered. `&Event`
 - **EVT_DATA**: Sends the data of the event that was triggered. `&EventData`

It's also possible to not use any parameters, ex: `TestApp::callback1()`.

Different event types:

```
struct TestApp {
    #[nwg_control]
    #[nwg_events(
        OnButtonClick: [TestApp::callback1, TestApp::callback2],
        OnMouseMove: [TestApp::callback3(SELF, CTRL)],
        OnButtonDoubleClick: [callback, another_callback()]
    )]
    button: nwg::Button
}

fn callback(me: &TestApp) {}
fn another_callback() {}

impl TestApp {
    fn callback1(&self) { }
    fn callback2(&self) { }
    fn callback3(&self, ctrl: &nwg::Button) { }
}
```

## Layouts

Use the `nwg_layout` attribute to instance a layout from a struct field and `nwg_layout_item` to associate a control to a layout.

Under the hood, both these attribute work the same way as `nwg_control`. `nwg_layout` uses the builder attribute for a the layout struct and
`nwg_layout_item` uses the parameters of the item type of the parent (ex: `GridLayoutItem` for `GridLayout`).

```
#[derive(Default, NwgUi)]
pub struct LayoutApp {
    #[nwg_control(size: (600, 400), flags: "MAIN_WINDOW|VISIBLE")]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 1, margin: [40, 5, 30, 5])]
    grid_layout: nwg::GridLayout,

    #[nwg_control(text: "Field Label")]
    #[nwg_layout_item(layout: grid_layout, row: 0, col: 0)]
    label: nwg::Label,

    #[nwg_control]
    #[nwg_layout_item(layout: grid_layout, row: 0, col: 1)]
    right_edit: nwg::TextInput,

    #[nwg_control]
    #[nwg_layout_item(layout: grid_layout, row: 1, col: 0, col_span: 2)]
    hello_button: nwg::Button,

    #[nwg_control]
    #[nwg_layout_item(layout: grid_layout, row: 2, col: 0, row_span: 2, col_span: 2)]
    list_view: nwg::ListView,
}
```

NWD cannot guess the parent of layout items.

Flexbox layouts can be nested.

```
#[derive(Default, NwgUi)]
pub struct NestedApp {

#[nwg_control(size: (600, 400), position: (300, 300), title: "Nested example", flags: "MAIN_WINDOW|VISIBLE")]
window: nwg::Window,

#[nwg_layout(parent: window, flex_direction: FlexDirection::Column)]
window_layout: nwg::FlexboxLayout,

#[nwg_layout(parent: window, flex_direction: FlexDirection::Row)]
#[nwg_layout_item(layout: window_layout)]
row_layout: nwg::FlexboxLayout,

#[nwg_control]
#[nwg_layout_item(layout: window_layout)]
hello_button: nwg::Button,

#[nwg_control(text: "Left")]
#[nwg_layout_item(layout: row_layout)]
left_edit: nwg::TextInput,

#[nwg_control(text: "Right",)]
#[nwg_layout_item(layout: row_layout)]
right_edit: nwg::TextInput,

#[nwg_control]
#[nwg_layout_item(layout: window_layout)]
list_view: nwg::ListView,
}
```

## Partials

Use the `nwg_partial` attribute to instance a partial from a struct field:

If parts of your UI is another struct that implements the `PartialUi` trait, it can be easily included in your base UI using `nwg_partial`.
The attribute accepts an optional parameter "parent" to pass a parent control to the partial initializer. Unlike the parent in `nwg_controls`,
it must be explicitly defined.

nwg_partial works by calling `PartialUi::build_partial` after initializing the controls of the base UI, calling `PartialUi::process_event` in the default event handler,
and binds the default handler to the handles returned by `PartialUi::handles`

Also see `NwgPartial` for the macro to generate a nwg partial.

```
struct Ui {
    window: nwg::Window,

    #[nwg_partial(parent: window)]
    partial: MyPartial
}
```

*/
#[proc_macro_derive(
    NwgUi,
    attributes(
        nwg_root,
        nwg_control,
        nwg_resource,
        nwg_events,
        nwg_layout,
        nwg_layout_item,
        nwg_partial,
        nwg_control_layout,
        nwg_partial_control,
        nwg_shortcuts,
    )
)]
pub fn derive_ui(input: pm::TokenStream) -> pm::TokenStream {
    match derive_base(&parse_macro_input!(input as DeriveInput)) {
        Ok(val) => val,
        Err(err) => err.into_compile_error(),
    }
    .into()
}

fn derive_base(base: &DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let names = parse_base_names(&base);
    let ui_data = parse_ui_data(&base).expect("NWG derive can only be implemented on structs");
    let attrs = fetch_attr(&base);

    let module_name = &names.n_module;
    let struct_name = &names.n_struct;
    let ui_struct_name = &names.n_struct_ui;

    let (generics, generic_names, where_clause) = &base.generics.split_for_impl();

    let ui = NwgUi::build(&ui_data, attrs, false)?;

    let controls = ui.controls();
    let resources = ui.resources();
    let partials = ui.partials();
    let layouts = ui.layouts();
    let events = ui.events();
    let shortcuts = ui.shortcuts();

    let nwg = get_crate_name();
    let shortcuts_impl = match shortcuts.len() {
        0 => quote! {},
        _ => quote! {impl #generics #struct_name #generic_names #where_clause {
                                #shortcuts
        }},
    };

    Ok(quote! {
            mod #module_name {
                extern crate #nwg as nwg;
                use nwg::*;
                use super::*;
                use std::ops::Deref;
                use std::cell::RefCell;
                use std::rc::Rc;
                use std::fmt;

                pub struct #ui_struct_name #generics #where_clause {
                    inner: Rc<#struct_name #generic_names>,
                    default_handlers: RefCell<Vec<EventHandler>>
                }

                impl #generics NativeUi<#ui_struct_name #generic_names> for #struct_name #generic_names #where_clause {
                    fn build_ui(mut data: Self) -> Result<#ui_struct_name #generic_names, NwgError> {
                        #resources
                        #controls
                        #partials

                        let inner = Rc::new(data);
                        let ui = #ui_struct_name { inner: inner.clone(), default_handlers: Default::default() };

                        #events
                        #layouts

                        Ok(ui)
                    }
                }

                #shortcuts_impl

                impl #generics Drop for #ui_struct_name #generic_names #where_clause {
                    /// To make sure that everything is freed without issues, the default handler must be unbound.
                    fn drop(&mut self) {
                        let mut handlers = self.default_handlers.borrow_mut();
                        for handler in handlers.drain(0..) {
                            nwg::unbind_event_handler(&handler);
                        }
                    }
                }

                impl #generics Deref for #ui_struct_name #generic_names #where_clause {
                    type Target = #struct_name #generic_names;

                    fn deref(&self) -> &Self::Target {
                        &self.inner
                    }
                }

                impl #generics fmt::Debug for #ui_struct_name #generic_names #where_clause {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "[#ui_struct_name Ui]")
                    }
                }
            }

    })
}

fn get_crate_name() -> Ident {
    let nwg_name =
        crate_name("native-windows-gui").expect("native-windows-gui is present in `Cargo.toml`");

    match nwg_name {
        FoundCrate::Itself => Ident::new("native_windows_gui", Span::call_site()),
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
    }
}

/**
The `NwgPartial` macro implements the native-windows-gui `PartialUi` trait on the selected struct

`NwgPartial` accepts the same attributes as `NwgUi`. See the docs of the `NwgUi` trait for detailed usage. There are some particularities though:

 - Partials cannot be used by independently. They must be included in a UI that implements `NwgUi`.
 - Partials do not require a top level window. If no window is defined, the partial will require a parent value passed from the `nwg_partial` attribute
 - It's possible to derive both `NwgUi` and `NwgPartial` from the same struct as long as the partial do not need a parent.
 - Partials can contains other partials

```
#[derive(Default, NwgPartial)]
pub struct MyPartial {
  partial_data: u32,

  #[nwg_control]
  button: nwg::Button
}

#[derive(Default, NwgUi)]
pub struct MyApp {
   app_data: u32,

   #[nwg_control]
   #[nwg_events( OnInit: [hello], OnWindowClose: [nwg::stop_thread_dispatch()] )]
   window: nwg::Window,

   #[nwg_partial(parent: window)]
   partial: MyPartial
}
```

*/
#[proc_macro_derive(
    NwgPartial,
    attributes(
        nwg_root,
        nwg_control,
        nwg_resource,
        nwg_events,
        nwg_layout,
        nwg_layout_item,
        nwg_partial,
        nwg_control_layout,
        nwg_partial_control,
        nwg_shortcuts,
    )
)]
pub fn derive_partial(input: pm::TokenStream) -> pm::TokenStream {
    match derive_partial_base(&parse_macro_input!(input as DeriveInput)) {
        Ok(val) => val,
        Err(err) => err.into_compile_error(),
    }
    .into()
}

fn derive_partial_base(base: &DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let names = parse_base_names(&base);
    let attrs = fetch_attr(&base);

    let partial_name = &names.n_partial_module;
    let struct_name = &names.n_struct;

    let (generics, generic_names, where_clause) = &base.generics.split_for_impl();

    let ui_data = parse_ui_data(&base).expect("NWG derive can only be implemented on structs");
    let ui = NwgUi::build(&ui_data, attrs, true)?;

    let controls = ui.controls();
    let resources = ui.resources();
    let partials = ui.partials();
    let layouts = ui.layouts();
    let events = ui.events();
    let shortcuts = ui.shortcuts();
    let (_root_id, _root_type) = ui.root_element();

    let subclass = if _root_id.is_some() {
        let root_id = _root_id.unwrap();
        let root_type = _root_type.unwrap();
        quote! {
        nwg::subclass_control!(#struct_name, #root_type, #root_id);
        }
    } else {
        quote! {}
    };

    let nwg = get_crate_name();
    let shortcuts_impl = match shortcuts.len() {
        0 => quote! {},
        _ => quote! {impl #generics #struct_name #generic_names #where_clause {
                                #shortcuts
        }},
    };

    Ok(quote! {
        mod #partial_name {
            extern crate #nwg as nwg;
            use nwg::*;
            use super::*;

            #subclass


                #shortcuts_impl

            impl #generics PartialUi for #struct_name #generic_names #where_clause {

                #[allow(unused)]
                fn build_partial<W: Into<ControlHandle>>(data: &mut Self, _parent: Option<W>, expand_layout_p: bool) -> Result<(), NwgError> {
                    let parent = _parent.map(|p| p.into());
                    let parent_ref = parent.as_ref();

                    #resources
                    #controls
                    #partials

                    let ui = data;
                    #layouts
                    Ok(())
                }

                fn process_event<'a>(&self, _evt: Event, _evt_data: &EventData, _handle: ControlHandle) {
                    #events
                }

                fn handles(&self) -> Vec<&ControlHandle> {
                    Vec::new()
                }
            }
        }
    })
}
