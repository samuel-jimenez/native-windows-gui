use std::{
    cell::{Ref, RefMut},
    cmp::max,
    fmt::Display,
};

use derive_setters::Setters;
use taffy::{Dimension, Size};

use super::ControlHandle;
use crate::{
    ComboBox, ComboBoxFlags, FlexboxLayout, Font, HTextAlign, Label, NwgError, VTextAlign,
};

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
    pub layout: FlexboxLayout,

    label: Label,
    pub field: ComboBox<D>,
}

impl<D: Display + Default> LabeledCombo<D> {
    pub fn builder<'a>() -> LabeledComboBuilder<'a, D> {
        LabeledComboBuilder::default()
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
        let (w_label, h_label) = self.label.size();
        let (w_field, h_field) = self.field.size();
        (w_label + w_field, max(h_label, h_field))
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

    pub fn set_border_color(&self, color: Option<[u8; 3]>) {
        self.label.set_border_color(color);
    }
}
#[derive(Setters)]
pub struct LabeledComboBuilder<'a, D: Display + Default> {
    #[setter(name=label)]
    label_text: &'a str,
    label_h_align: HTextAlign,
    label_v_align: VTextAlign,
    label_width: Dimension,
    size: (i32, i32),
    position: (i32, i32),
    visible: bool,
    enabled: bool,
    focus: bool,
    upcase: bool,
    downcase: bool,
    autoscroll: bool,
    scrollbar: bool,
    #[setter(strip_option)]
    flags: Option<ComboBoxFlags>,
    ex_flags: u32,
    font: Option<&'a Font>,
    #[setter(strip_option)]
    collection: Option<Vec<D>>,
    selected_index: Option<usize>,
    background_color: Option<[u8; 3]>,
    #[setter(into, strip_option)]
    parent: Option<ControlHandle>,
}
impl<'a, D: Display + Default> Default for LabeledComboBuilder<'a, D> {
    fn default() -> Self {
        Self {
            label_text: "",
            label_h_align: HTextAlign::Left,
            label_v_align: VTextAlign::Top,
            label_width: Dimension::percent(0.45),
            size: (100, 25),
            position: (0, 0),
            visible: true,
            enabled: true,
            focus: false,
            upcase: false,
            downcase: false,
            autoscroll: true,
            scrollbar: true,
            flags: None,
            ex_flags: 0,
            font: None,
            collection: None,
            selected_index: None,
            background_color: None,
            parent: None,
        }
    }
}

impl<'a, D: Display + Default> LabeledComboBuilder<'a, D> {
    const FIELD_SIZE: Size<Dimension> = Size {
        width: Dimension::percent(1.0),
        height: Dimension::auto(),
    };

    pub fn v_align(self, _align: VTextAlign) -> LabeledComboBuilder<'a, D> {
        // Disabled for now because of a bug. Keep the method for backward compatibility
        self
    }

    pub fn build(self, out: &mut LabeledCombo<D>) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("LabeledCombo")),
        }?;

        let label_size = Size {
            width: self.label_width,
            height: Dimension::auto(),
        };

        // Drop the old object
        *out = LabeledCombo::default();

        Label::builder()
            .parent(&parent)
            .text(self.label_text)
            .h_align(self.label_h_align)
            .v_align(self.label_v_align)
            .font(self.font)
            .background_color(self.background_color)
            .build(&mut out.label)?;

        let mut field = ComboBox::builder().parent(&parent);
        if self.flags.is_some() {
            field = field.flags(self.flags.unwrap());
        }

        field
            .ex_flags(self.ex_flags)
            .size(self.size)
            .font(self.font)
            .visible(self.visible)
            .enabled(self.enabled)
            .focus(self.focus)
            .upcase(self.upcase)
            .downcase(self.downcase)
            .autoscroll(self.autoscroll)
            .scrollbar(self.scrollbar)
            .build(&mut out.field)?;

        FlexboxLayout::builder()
            .parent(&parent)
            .child(&out.label)
            .child_size(label_size)
            .child(&out.field)
            .child_size(Self::FIELD_SIZE)
            .build_partial(&out.layout)?;

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

impl<D: Display + Default> PartialEq for LabeledCombo<D> {
    fn eq(&self, other: &Self) -> bool {
        self.field == other.field
    }
}
