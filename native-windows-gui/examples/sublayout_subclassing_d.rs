/*!
    Example on how to use custom types directly with native windows derive

    `cargo run --example sublayout_subclassing_d --features "flexbox"`
*/

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use nwd::NwgUi;
use nwg::{
    ControlHandle, FlexboxLayout, Font, HTextAlign, Label, NativeUi, NwgError, TextInput,
    TextInputFlags, VTextAlign, subclass_control,
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

//
// Implement a builder API compatible with native window derive
//
impl NumberUnitsEdit {
    pub fn builder<'a>() -> NumberUnitsEditBuilder<'a> {
        NumberUnitsEditBuilder {
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
            flags: None,
            ex_flags: 0,
            limit: 0,
            password: None,
            align: HTextAlign::Left,
            readonly: false,
            focus: false,
            font: None,
            parent: None,
            background_color: None,
        }
    }
}

pub struct NumberUnitsEditBuilder<'a> {
    label_text: &'a str,
    label_h_align: HTextAlign,
    label_v_align: VTextAlign,
    label_width: Dimension,
    units_text: &'a str,
    units_h_align: HTextAlign,
    units_v_align: VTextAlign,
    units_width: Dimension,
    text: &'a str,
    placeholder_text: Option<&'a str>,
    field_width: Dimension,
    size: (i32, i32),
    position: (i32, i32),
    flags: Option<TextInputFlags>,
    ex_flags: u32,
    limit: usize,
    password: Option<char>,
    align: HTextAlign,
    readonly: bool,
    font: Option<&'a Font>,
    parent: Option<ControlHandle>,
    background_color: Option<[u8; 3]>,
    focus: bool,
}

impl<'a> NumberUnitsEditBuilder<'a> {
    pub fn flags(mut self, flags: TextInputFlags) -> NumberUnitsEditBuilder<'a> {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, flags: u32) -> NumberUnitsEditBuilder<'a> {
        self.ex_flags = flags;
        self
    }

    pub fn text(mut self, text: &'a str) -> NumberUnitsEditBuilder<'a> {
        self.text = text;
        self
    }

    pub fn label(mut self, label_text: &'a str) -> NumberUnitsEditBuilder<'a> {
        self.label_text = label_text;
        self
    }

    pub fn label_h_align(mut self, align: HTextAlign) -> NumberUnitsEditBuilder<'a> {
        self.label_h_align = align;
        self
    }

    pub fn label_v_align(mut self, align: VTextAlign) -> NumberUnitsEditBuilder<'a> {
        self.label_v_align = align;
        self
    }

    pub fn label_width(mut self, label_width: Dimension) -> NumberUnitsEditBuilder<'a> {
        self.label_width = label_width;
        self
    }

    pub fn units(mut self, units_text: &'a str) -> NumberUnitsEditBuilder<'a> {
        self.units_text = units_text;
        self
    }

    pub fn units_h_align(mut self, align: HTextAlign) -> NumberUnitsEditBuilder<'a> {
        self.units_h_align = align;
        self
    }

    pub fn units_v_align(mut self, align: VTextAlign) -> NumberUnitsEditBuilder<'a> {
        self.units_v_align = align;
        self
    }

    pub fn units_width(mut self, units_width: Dimension) -> NumberUnitsEditBuilder<'a> {
        self.units_width = units_width;
        self
    }

    pub fn field_width(mut self, field_width: Dimension) -> NumberUnitsEditBuilder<'a> {
        self.field_width = field_width;
        self
    }

    pub fn placeholder_text(
        mut self,
        placeholder_text: Option<&'a str>,
    ) -> NumberUnitsEditBuilder<'a> {
        self.placeholder_text = placeholder_text;
        self
    }

    pub fn size(mut self, size: (i32, i32)) -> NumberUnitsEditBuilder<'a> {
        self.size = size;
        self
    }

    pub fn position(mut self, pos: (i32, i32)) -> NumberUnitsEditBuilder<'a> {
        self.position = pos;
        self
    }

    pub fn limit(mut self, limit: usize) -> NumberUnitsEditBuilder<'a> {
        self.limit = limit;
        self
    }

    pub fn password(mut self, psw: Option<char>) -> NumberUnitsEditBuilder<'a> {
        self.password = psw;
        self
    }

    pub fn align(mut self, align: HTextAlign) -> NumberUnitsEditBuilder<'a> {
        self.align = align;
        self
    }

    pub fn readonly(mut self, read: bool) -> NumberUnitsEditBuilder<'a> {
        self.readonly = read;
        self
    }

    pub fn font(mut self, font: Option<&'a Font>) -> NumberUnitsEditBuilder<'a> {
        self.font = font;
        self
    }

    pub fn background_color(mut self, color: Option<[u8; 3]>) -> NumberUnitsEditBuilder<'a> {
        self.background_color = color;
        self
    }

    pub fn focus(mut self, focus: bool) -> NumberUnitsEditBuilder<'a> {
        self.focus = focus;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> NumberUnitsEditBuilder<'a> {
        self.parent = Some(p.into());
        self
    }

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

        let mut field = TextInput::builder().parent(&parent);
        if self.flags.is_some() {
            field = field.flags(self.flags.unwrap());
        }

        field
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

impl From<&NumberUnitsEdit> for FlexboxLayout {
    fn from(control: &NumberUnitsEdit) -> Self {
        control.layout.clone()
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
