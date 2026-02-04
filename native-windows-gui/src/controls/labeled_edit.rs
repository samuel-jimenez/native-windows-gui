use std::ops::Range;

use derive_setters::Setters;
use taffy::{Dimension, Size};

use super::ControlHandle;
use crate::{
    FlexboxLayout, Font, HTextAlign, Label, NwgError, TextInput, TextInputFlags, VTextAlign,
};

/**
A labeled input control is an edit control with included label.

Requires the `labeled` feature.

**Builder parameters:**
  * `parent`:           **Required.** The labeled input parent container.
  * `label`:            The labeled input label text.
  * `text`:             The labeled input text.
  * `placeholder_text`: The labeled input placeholder text.
  * `size`:             The labeled input size.
  * `position`:         The labeled input position.
  * `flags`:            A combination of the TextInputFlags values.
  * `ex_flags`:         A combination of win32 window extended flags. Unlike `flags`, ex_flags must be used straight from winapi
  * `font`:             The font used for the labeled input text
  * `limit`:            The maximum number of characters that can be inserted in the control
  * `readonly`:         If the labeled input should allow user input or not
  * `password`:         The password character. If set to None, the input is a regular control.
  * `align`:            The alignment of the text in the labeled input
  * `background_color`: The color of the top and bottom padding. This is not the white background under the text.
  * `focus`:            The control receives focus after being created

**Control events:**
  * `OnTextInput`: When a LabeledEdit value is changed
  * `MousePress(_)`: Generic mouse press events on the button
  * `OnMouseMove`: Generic mouse mouse event
  * `OnMouseWheel`: Generic mouse wheel event

```rust
use native_windows_gui as nwg;
fn build_box(tbox: &mut nwg::LabeledEdit, window: &nwg::Window, font: &nwg::Font) {
    nwg::LabeledEdit::builder()
        .label("Hello")
        .text("World")
        .font(Some(font))
        .parent(window)
        .build(tbox);
}
```
*/

#[derive(Default)]
pub struct LabeledEdit {
    pub layout: FlexboxLayout,

    label: Label,
    pub field: TextInput,
}

impl LabeledEdit {
    pub fn builder<'a>() -> LabeledEditBuilder<'a> {
        LabeledEditBuilder::default()
    }

    /// Return the text displayed in the label
    pub fn label(&self) -> String {
        self.label.text()
    }

    /// Set the text displayed in the label
    pub fn set_label<'a>(&self, v: &'a str) {
        self.label.set_text(v)
    }

    /// Return the handle of the Label control
    pub fn label_handle(&self) -> ControlHandle {
        self.label.handle
    }

    /// Return the font of the control
    pub fn font(&self) -> Option<Font> {
        self.field.font()
    }

    /// Set the font of the control
    pub fn set_font(&self, font: Option<&Font>) {
        self.label.set_font(font);
        self.field.set_font(font);
    }

    /// Return the password character displayed by the text input. If the input is not a password, return None.
    pub fn password_char(&self) -> Option<char> {
        self.field.password_char()
    }

    /// Set or Remove the password character displayed by the text input.
    /// If the input is not a password all character are re-rendered with the new character
    pub fn set_password_char(&self, c: Option<char>) {
        self.field.set_password_char(c)
    }

    /// Return the number of maximum character allowed in this text input
    pub fn limit(&self) -> u32 {
        self.field.limit()
    }

    /// Set the number of maximum character allowed in this text input
    /// If `limit` is 0, the text length is set to 0x7FFFFFFE characters
    pub fn set_limit(&self, limit: usize) {
        self.field.set_limit(limit)
    }

    /// Check if the content of the text input was modified after it's creation
    pub fn modified(&self) -> bool {
        self.field.modified()
    }

    /// Manually set modified flag of the text input
    pub fn set_modified(&self, e: bool) {
        self.field.set_modified(e)
    }

    /// Undo the last action by the user in the control
    pub fn undo(&self) {
        self.field.undo()
    }

    /// Return the selected range of characters by the user in the text input
    pub fn selection(&self) -> Range<u32> {
        self.field.selection()
    }

    /// Return the selected range of characters by the user in the text input
    pub fn set_selection(&self, r: Range<u32>) {
        self.field.set_selection(r)
    }

    /// Return the length of the user input in the control. This is better than `input.text().len()` as it
    /// does not allocate a string in memory
    pub fn len(&self) -> u32 {
        self.field.len()
    }

    /// Return true if the TextInput value cannot be edited. Retrurn false otherwise.
    /// A user can still copy text from a readonly TextEdit (unlike disabled)
    pub fn readonly(&self) -> bool {
        self.field.readonly()
    }

    /// Set the readonly flag of the text input
    /// A user can still copy text from a readonly TextEdit (unlike disabled)
    pub fn set_readonly(&self, r: bool) {
        self.field.set_readonly(r);
    }

    /// Return true if the control currently has the keyboard focus
    pub fn focus(&self) -> bool {
        self.field.focus()
    }

    /// Set the keyboard focus on the button
    pub fn set_focus(&self) {
        self.field.set_focus()
    }

    /// Return true if the control user can interact with the control, return false otherwise
    pub fn enabled(&self) -> bool {
        self.field.enabled()
    }

    /// Enable or disable the control
    pub fn set_enabled(&self, v: bool) {
        self.label.set_enabled(v);
        self.field.set_enabled(v);
    }

    /// Return true if the control is visible to the user. Will return true even if the
    /// control is outside of the parent client view (ex: at the position (10000, 10000))
    pub fn visible(&self) -> bool {
        self.field.visible()
    }

    /// Show or hide the control to the user
    pub fn set_visible(&self, v: bool) {
        self.label.set_visible(v);
        self.field.set_visible(v);
    }

    /// Return the size of the button in the parent window
    pub fn size(&self) -> (u32, u32) {
        self.field.size()
    }

    /// Set the size of the button in the parent window
    pub fn set_size(&self, x: u32, y: u32) {
        self.field.set_size(x, y)
    }

    /// Return the position of the button in the parent window
    pub fn position(&self) -> (i32, i32) {
        self.field.position()
    }

    /// Set the position of the button in the parent window
    pub fn set_position(&self, x: i32, y: i32) {
        self.field.set_position(x, y)
    }

    /// Return the text displayed in the TextInput
    pub fn text(&self) -> String {
        self.field.text()
    }

    /// Set the text displayed in the TextInput
    pub fn set_text<'a>(&self, v: &'a str) {
        self.field.set_text(v)
    }

    /// Return the placeholder text displayed in the TextInput
    /// when it is empty and does not have focus. The string returned will be
    /// as long as the user specified, however it might be longer or shorter than
    /// the actual placeholder text.
    pub fn placeholder_text<'a>(&self, text_length: usize) -> String {
        self.field.placeholder_text(text_length)
    }

    /// Set the placeholder text displayed in the TextInput
    /// when it is empty and does not have focus
    pub fn set_placeholder_text<'a>(&self, v: Option<&'a str>) {
        self.field.set_placeholder_text(v)
    }
}

#[derive(Setters)]
pub struct LabeledEditBuilder<'a> {
    #[setter(name=label)]
    label_text: &'a str,
    label_h_align: HTextAlign,
    label_v_align: VTextAlign,
    label_width: Dimension,
    text: &'a str,
    placeholder_text: Option<&'a str>,
    size: (i32, i32),
    position: (i32, i32),
    #[setter(strip_option)]
    flags: Option<TextInputFlags>,
    ex_flags: u32,
    limit: usize,
    password: Option<char>,
    align: HTextAlign,
    readonly: bool,
    font: Option<&'a Font>,
    #[setter(into, strip_option)]
    parent: Option<ControlHandle>,
    background_color: Option<[u8; 3]>,
    focus: bool,
}
impl<'a> Default for LabeledEditBuilder<'a> {
    fn default() -> Self {
        Self {
            label_text: "",
            label_h_align: HTextAlign::Left,
            label_v_align: VTextAlign::Center,
            label_width: Dimension::percent(0.45),
            text: "",
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

impl<'a> LabeledEditBuilder<'a> {
    const FIELD_SIZE: Size<Dimension> = Size {
        width: Dimension::percent(1.0),
        height: Dimension::auto(),
    };

    pub fn build(self, out: &mut LabeledEdit) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("LabeledEdit")),
        }?;

        let label_size = Size {
            width: self.label_width,
            height: Dimension::auto(),
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

        FlexboxLayout::builder()
            .parent(&parent)
            .child(&out.label)
            .child_size(label_size)
            .child(&out.field)
            .child_size(Self::FIELD_SIZE)
            .build_partial(&out.layout)?;

        Ok(())
    }
}

impl PartialEq for LabeledEdit {
    fn eq(&self, other: &Self) -> bool {
        self.field == other.field
    }
}
