use super::ControlHandle;
use crate::{
    ComboBox, ComboBoxFlags, FlexboxLayout, Font, HTextAlign, Label, NwgError, VTextAlign,
};
use std::cell::{Ref, RefMut};
use std::fmt::Display;

/**
A labeled combobox is a combobox control with included label.

Requires the `labeled` and `combobox` features.

**Builder parameters:**
  * `parent`:         **Required.** The combobox parent container.
  * `size`:           The combobox size.
  * `position`:       The combobox position.
  * `enabled`:        If the combobox can be used by the user. It also has a grayed out look if disabled.
  * `flags`:          A combination of the ComboBoxFlags values.
  * `ex_flags`:       A combination of win32 window extended flags. Unlike `flags`, ex_flags must be used straight from winapi
  * `font`:           The font used for the combobox text
  * `collection`:     The default collection of the combobox
  * `selected_index`: The default selected index. None means no values are selected.
  * `focus`:          The control receive focus after being created

**Control events:**
  * `OnComboBoxClosed`: When the combobox dropdown is closed
  * `OnComboBoxDropdown`: When the combobox dropdown is opened
  * `OnComboxBoxSelection`: When a new value in a combobox is choosen
  * `MousePress(_)`: Generic mouse press events on the checkbox
  * `OnMouseMove`: Generic mouse mouse event
  * `OnMouseWheel`: Generic mouse wheel event


```rust
use native_windows_gui as nwg;
fn build_combobox(combo: &mut nwg::LabeledCombo<&'static str>, window: &nwg::Window) {
    let data = vec!["one", "two"];

    nwg::LabeledCombo::builder()
        .size((200, 300))
        .label("Data")
        .collection(data)
        .selected_index(Some(0))
        .parent(window)
        .build(combo);
}
```
*/
#[derive(Default)]
pub struct LabeledCombo<D: Display + Default> {
    layout: FlexboxLayout,

    label: Label,
    field: ComboBox<D>,
}

impl<D: Display + Default> LabeledCombo<D> {
    pub fn builder<'a>() -> LabeledComboBuilder<'a, D> {
        LabeledComboBuilder {
            label_text: "",
            label_h_align: HTextAlign::Left,
            label_v_align: VTextAlign::Top,
            size: (100, 25),
            position: (0, 0),
            enabled: true,
            focus: false,
            flags: None,
            ex_flags: 0,
            font: None,
            collection: None,
            selected_index: None,
            parent: None,
        }
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

    /// Remove the item at the selected index and returns it.
    /// Panic of the index is out of bounds
    pub fn remove(&self, index: usize) -> D {
        self.field.remove(index)
    }

    /// Sort the inner collection by the display value of it's items and update the view
    /// Internally this uses `Vec.sort_unstable_by`.
    pub fn sort(&self) {
        self.field.sort()
    }

    /// Show or hide the dropdown of the combox
    pub fn dropdown(&self, v: bool) {
        self.field.dropdown(v)
    }

    /// Return the index of the currently selected item. Return `None` if no item is selected.
    pub fn selection(&self) -> Option<usize> {
        self.field.selection()
    }

    /// Return the display value of the currently selected item
    /// Return `None` if no item is selected. This reads the visual value.
    pub fn selection_string(&self) -> Option<String> {
        self.field.selection_string()
    }

    pub fn selection_string_or_text(&self) -> String {
        self.field.selection_string_or_text()
    }

    /// Set the currently selected item in the combobox.
    /// Does nothing if the index is out of bound
    /// If the value is None, remove the selected value
    pub fn set_selection(&self, index: Option<usize>) {
        self.field.set_selection(index)
    }

    /// Search an item that begins by the value and select the first one found.
    /// The search is not case sensitive, so this string can contain any combination of uppercase and lowercase letters.
    /// Return the index of the selected string or None if the search was not successful
    pub fn set_selection_string(&self, value: &str) -> Option<usize> {
        self.field.set_selection_string(value)
    }

    /// Add a new item to the combobox. Sort the collection if the combobox is sorted.
    pub fn push(&self, item: D) {
        self.field.push(item)
    }

    /// Insert an item in the collection and the control.
    ///
    /// SPECIAL behaviour! If index is `std::usize::MAX`, the item is added at the end of the collection.
    /// The method will still panic if `index > len` with every other values.
    pub fn insert(&self, index: usize, item: D) {
        self.field.insert(index, item)
    }

    /// Update the visual of the control with the inner collection.
    /// This rebuild every item in the combobox and can take some time on big collections.
    pub fn sync(&self) {
        self.field.sync()
    }

    /// Set the item collection of the combobox. Return the old collection
    pub fn set_collection(&self, col: Vec<D>) -> Vec<D> {
        self.field.set_collection(col)
    }

    /// Return the number of items in the control. NOT the inner rust collection
    pub fn len(&self) -> usize {
        self.field.len()
    }

    //
    // Common control functions
    //

    /// Return the font of the control
    pub fn font(&self) -> Option<Font> {
        self.field.font()
    }

    /// Set the font of the control
    pub fn set_font(&self, font: Option<&Font>) {
        self.label.set_font(font);
        self.field.set_font(font);
    }

    /// Return true if the control currently has the keyboard focus
    pub fn focus(&self) -> bool {
        self.field.focus()
    }

    /// Set the keyboard focus on the button.
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
        self.field.set_enabled(v)
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

    /// Get read-only access to the inner collection of the combobox
    /// This call refcell.borrow under the hood. Be sure to drop the value before
    /// calling other combobox methods
    pub fn collection(&self) -> Ref<'_, Vec<D>> {
        self.field.collection()
    }

    /// Get mutable access to the inner collection of the combobox. Does not update the visual
    /// control. Call `sync` to update the view. This call refcell.borrow_mut under the hood.
    /// Be sure to drop the value before calling other combobox methods
    pub fn collection_mut(&self) -> RefMut<'_, Vec<D>> {
        self.field.collection_mut()
    }
}

pub struct LabeledComboBuilder<'a, D: Display + Default> {
    label_text: &'a str,
    label_h_align: HTextAlign,
    label_v_align: VTextAlign,
    size: (i32, i32),
    position: (i32, i32),
    enabled: bool,
    focus: bool,
    flags: Option<ComboBoxFlags>,
    ex_flags: u32,
    font: Option<&'a Font>,
    collection: Option<Vec<D>>,
    selected_index: Option<usize>,
    parent: Option<ControlHandle>,
}

impl<'a, D: Display + Default> LabeledComboBuilder<'a, D> {
    pub fn flags(mut self, flags: ComboBoxFlags) -> LabeledComboBuilder<'a, D> {
        self.flags = Some(flags);
        self
    }

    pub fn ex_flags(mut self, flags: u32) -> LabeledComboBuilder<'a, D> {
        self.ex_flags = flags;
        self
    }

    pub fn label(mut self, label_text: &'a str) -> LabeledComboBuilder<'a, D> {
        self.label_text = label_text;
        self
    }
    pub fn label_h_align(mut self, align: HTextAlign) -> LabeledComboBuilder<'a, D> {
        self.label_h_align = align;
        self
    }

    pub fn label_v_align(mut self, align: VTextAlign) -> LabeledComboBuilder<'a, D> {
        self.label_v_align = align;
        self
    }

    pub fn size(mut self, size: (i32, i32)) -> LabeledComboBuilder<'a, D> {
        self.size = size;
        self
    }

    pub fn position(mut self, pos: (i32, i32)) -> LabeledComboBuilder<'a, D> {
        self.position = pos;
        self
    }

    pub fn font(mut self, font: Option<&'a Font>) -> LabeledComboBuilder<'a, D> {
        self.font = font;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> LabeledComboBuilder<'a, D> {
        self.parent = Some(p.into());
        self
    }

    pub fn collection(mut self, collection: Vec<D>) -> LabeledComboBuilder<'a, D> {
        self.collection = Some(collection);
        self
    }

    pub fn selected_index(mut self, index: Option<usize>) -> LabeledComboBuilder<'a, D> {
        self.selected_index = index;
        self
    }

    pub fn enabled(mut self, e: bool) -> LabeledComboBuilder<'a, D> {
        self.enabled = e;
        self
    }

    pub fn focus(mut self, focus: bool) -> LabeledComboBuilder<'a, D> {
        self.focus = focus;
        self
    }

    pub fn v_align(self, _align: VTextAlign) -> LabeledComboBuilder<'a, D> {
        // Disabled for now because of a bug. Keep the method for backward compatibility
        self
    }

    pub fn build(self, out: &mut LabeledCombo<D>) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("LabeledCombo")),
        }?;

        // Drop the old object
        *out = LabeledCombo::default();

        Label::builder()
            .parent(&parent)
            .text(self.label_text)
            .h_align(self.label_h_align)
            .v_align(self.label_v_align)
            .font(self.font)
            .build(&mut out.label)?;

        let mut field = ComboBox::builder().parent(&parent);
        if self.flags.is_some() {
            field = field.flags(self.flags.unwrap());
        }

        field
            .ex_flags(self.ex_flags)
            .size(self.size)
            .font(self.font)
            .enabled(self.enabled)
            .focus(self.focus)
            .build(&mut out.field)?;

        FlexboxLayout::builder()
            .parent(&parent)
            .child(&out.label)
            .child(&out.field)
            .build(&out.layout)?;

        if self.collection.is_some() {
            out.field.set_collection(self.collection.unwrap());
        }

        if self.selected_index.is_some() {
            out.field.set_selection(self.selected_index);
        }

        out.set_enabled(self.enabled);

        Ok(())
    }
}
impl<D: Display + Default> ::std::ops::Deref for LabeledCombo<D> {
    type Target = crate::ComboBox<D>;
    fn deref(&self) -> &crate::ComboBox<D> {
        &self.field
    }
}
impl<D: Display + Default> ::std::ops::DerefMut for LabeledCombo<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.field
    }
}
impl<D: Display + Default> Into<crate::ControlHandle> for &LabeledCombo<D> {
    fn into(self) -> crate::ControlHandle {
        self.field.handle.clone()
    }
}
impl<D: Display + Default> Into<crate::ControlHandle> for &mut LabeledCombo<D> {
    fn into(self) -> crate::ControlHandle {
        self.field.handle.clone()
    }
}
impl<D: Display + Default> PartialEq<LabeledCombo<D>> for crate::ControlHandle {
    fn eq(&self, other: &LabeledCombo<D>) -> bool {
        *self == other.field.handle
    }
}

impl<D: Display + Default> PartialEq for LabeledCombo<D> {
    fn eq(&self, other: &Self) -> bool {
        self.field == other.field
    }
}

impl<D: Display + Default> From<&LabeledCombo<D>> for FlexboxLayout {
    fn from(control: &LabeledCombo<D>) -> Self {
        control.layout.clone()
    }
}
