/*!
    Example on how to use custom types directly with native windows derive

    `cargo run --example sublayout_subclassing_d --features "flexbox"`
*/

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use derive_setters::Setters;
use nwd::NwgUi;
use nwg::{
    ControlHandle, FlexboxLayout, Font, HTextAlign, Label, NativeUi, NwgError, TextInput,
    VTextAlign, subclass_control, subclass_layout,
    taffy::{Dimension, Size, style::FlexDirection, style_helpers::auto},
};

#[derive(Default)]
pub struct NumberUnitsEdit {
    layout: FlexboxLayout,

    label: Label,
    field: TextInput,
    units: Label,
}

// Implements default trait so that the control can be used by native windows derive
// The parameters are: subclass_control!(user type, base type, base field name)
subclass_control!(NumberUnitsEdit, TextInput, field);

// Implements default trait so that the layout can be used by native windows derive
// The parameters are: subclass_layout!(user type, base type, base field name)
subclass_layout!(NumberUnitsEdit, FlexboxLayout, layout);

//
// Implement a builder API compatible with native window derive
//
impl NumberUnitsEdit {
    pub fn builder<'a>() -> NumberUnitsEditBuilder<'a> {
        NumberUnitsEditBuilder::default()
    }
}

#[derive(Setters)]
pub struct NumberUnitsEditBuilder<'a> {
    #[setter(name=label)]
    label_text: &'a str,
    label_h_align: HTextAlign,
    label_v_align: VTextAlign,
    label_width: Dimension,
    #[setter(name=units)]
    units_text: &'a str,
    units_h_align: HTextAlign,
    units_v_align: VTextAlign,
    units_width: Dimension,
    text: &'a str,
    placeholder_text: Option<&'a str>,
    field_width: Dimension,
    size: (i32, i32),
    position: (i32, i32),
    ex_flags: u32,
    limit: usize,
    password: Option<char>,
    align: HTextAlign,
    readonly: bool,
    visible: bool,
    enabled: bool,
    focus: bool,
    number: bool,
    autoscroll: bool,
    tab_stop: bool,
    font: Option<&'a Font>,
    #[setter(into, strip_option)]
    parent: Option<ControlHandle>,
}
impl<'a> Default for NumberUnitsEditBuilder<'a> {
    fn default() -> Self {
        Self {
            label_text: "",
            label_h_align: HTextAlign::Left,
            label_v_align: VTextAlign::Center,
            label_width: Dimension::percent(0.50),
            units_text: "",
            units_h_align: HTextAlign::Left,
            units_v_align: VTextAlign::Center,
            units_width: Dimension::percent(0.25),
            text: "",
            field_width: Dimension::percent(0.25),
            placeholder_text: None,
            size: (100, 25),
            position: (0, 0),
            ex_flags: 0,
            limit: 0,
            password: None,
            align: HTextAlign::Left,
            readonly: false,
            visible: true,
            enabled: true,
            focus: false,
            number: false,
            autoscroll: true,
            tab_stop: true,
            font: None,
            parent: None,
        }
    }
}

impl<'a> NumberUnitsEditBuilder<'a> {
    pub fn build(self, out: &mut NumberUnitsEdit) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("NumberUnitsEdit")),
        }?;

        let label_size = Size {
            width: self.label_width,
            height: auto(),
        };
        let units_size = Size {
            width: self.units_width,
            height: auto(),
        };
        let field_size = Size {
            width: self.field_width,
            height: auto(),
        };

        // Drop the old object
        *out = Default::default();

        Label::builder()
            .parent(&parent)
            .text(self.label_text)
            .h_align(self.label_h_align)
            .v_align(self.label_v_align)
            .font(self.font)
            .build(&mut out.label)?;

        TextInput::builder()
            .parent(&parent)
            .align(self.align)
            .size(self.size)
            .text(self.text)
            .placeholder_text(self.placeholder_text)
            .font(self.font)
            .password(self.password)
            .readonly(self.readonly)
            .focus(self.focus)
            .build(&mut out.field)?;

        Label::builder()
            .parent(&parent)
            .text(self.units_text)
            .h_align(self.units_h_align)
            .v_align(self.units_v_align)
            .font(self.font)
            .build(&mut out.units)?;

        FlexboxLayout::builder()
            .parent(&parent)
            .child(&out.label)
            .child_size(label_size)
            .child(&out.field)
            .child_size(field_size)
            .child(&out.units)
            .child_size(units_size)
            .build_partial(&out.layout)?;

        Ok(())
    }
}

//
// Actual interface code
//

#[derive(Default, NwgUi)]
pub struct SubclassApp {
    #[nwg_control(size: (300, 300), position: (700, 300), title: "Subclass example")]
    #[nwg_events( OnWindowClose: [SubclassApp::exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, flex_direction: FlexDirection::Column)]
    layout: nwg::FlexboxLayout,

    #[nwg_control(text: "Simple button", focus: true)]
    #[nwg_layout_item(layout: layout)]
    button1: nwg::Button,

    // `nested: true` expands this item as a sub-layout.
    #[nwg_control(nested: true, text: "5", label:"Subclassed:", units:"g/mL")]
    #[nwg_layout_item(layout: layout)]
    button3: NumberUnitsEdit,
}

impl SubclassApp {
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _app = SubclassApp::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
