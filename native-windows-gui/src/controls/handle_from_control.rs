use std::convert::From;
#[allow(unused)]
use std::fmt::Display;

use super::{
    Button, CheckBox, ControlHandle, GroupBox, ImageFrame, Label, RadioButton, TextInput, Window,
};

macro_rules! handles {
    ($control:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?) => {

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? From<&$control$(< $( $lt ),+ >)?> for ControlHandle {
            fn from(control: &$control$(< $( $lt ),+ >)?) -> Self {
                control.handle
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? PartialEq<ControlHandle> for $control$(< $( $lt ),+ >)? {
            fn eq(&self, other: &ControlHandle) -> bool {
                self.handle == *other
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? PartialEq<$control$(< $( $lt ),+ >)?> for ControlHandle {
            fn eq(&self, other: &$control$(< $( $lt ),+ >)?) -> bool {
                *self == other.handle
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? From<&$control$(< $( $lt ),+ >)?> for crate::FlexboxLayoutChild {
            fn from(control: &$control$(< $( $lt ),+ >)?) -> Self {
                control.handle.into()
            }
        }
    };
}

macro_rules! partial_eq {
    ($control:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? PartialEq<$control$(< $( $lt ),+ >)?> for $control$(< $( $lt ),+ >)? {
            fn eq(&self, other: &Self) -> bool {
                self.handle == other.handle
            }
        }
    };
}

#[macro_export]
macro_rules! subclass_control_base {
  ( $ty:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?,
    $base_type:ident $(< $( $blt:tt ),+ >)?,
    $field:ident) => {

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? ::std::ops::Deref for $ty$(< $( $lt ),+ >)? {
            type Target = $crate::$base_type$(< $( $blt ),+ >)?;
            fn deref(&self) -> &$crate::$base_type$(< $( $blt ),+ >)? {
                &self.$field
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? ::std::ops::DerefMut for $ty$(< $( $lt ),+ >)? {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? From<&$ty$(< $( $lt ),+ >)?> for $crate::ControlHandle {
            fn from(control: &$ty$(< $( $lt ),+ >)?) -> Self {
                control.$field.handle.clone()
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? PartialEq<$ty$(< $( $lt ),+ >)?> for $crate::ControlHandle {
            fn eq(&self, other: &$ty$(< $( $lt ),+ >)?) -> bool {
                *self == other.$field.handle
            }
        }
    };
}

/**
Automatically implements the functionalities required to process an external struct as a NWG control

```rust
extern crate native_windows_gui as nwg;
use nwg::subclass_control;

pub struct TestControl {
    edit: nwg::TextInput,
    custom_data: String,
}

subclass_control!(TestControl, TextInput, edit);
```
*/
#[macro_export]
macro_rules! subclass_control {
  ( $ty:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?,
    $base_type:ident $(< $( $blt:tt ),+ >)?,
    $field:ident) => {
        $crate::subclass_control_base!($ty$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?, $base_type$(< $( $blt ),+ >)?, $field);

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? From<&$ty$(< $( $lt ),+ >)?> for $crate::FlexboxLayoutChild {
            fn from(control: &$ty$(< $( $lt ),+ >)?) -> Self {
                $crate::ControlHandle::from(control).into()
            }
        }

    };
}

/**
Automatically implements the functionalities required to process an external struct as a NWG sub-layout.

```rust
extern crate native_windows_gui as nwg;
use nwg::subclass_layout;

pub struct TestLayout {
    layout: nwg::FlexboxLayout,
    edit: nwg::TextInput,
    custom_data: String,
}

subclass_layout!(TestLayout, FlexboxLayout, layout);
```
*/

#[macro_export]
macro_rules! subclass_layout {
  ( $ty:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?,
    $layout_type:ident,
    $layout_field:ident $(.$subfield:tt)*) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            From<&$ty$(< $( $lt ),+ >)?>
        for $crate::$layout_type {
           fn from(control: &$ty$(< $( $lt ),+ >)?) -> Self {
                control.$layout_field$(.$subfield)*.clone()
            }
        }

        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? From<&$ty$(< $( $lt ),+ >)?> for $crate::FlexboxLayoutChild {
            fn from(control: &$ty$(< $( $lt ),+ >)?) -> Self {
                $crate::$layout_type::from(control).into()
            }
        }
    };
}

/**
Automatically implements the functionalities required to process an external struct as both a NWG sub-layout and a NWG control.

```rust
extern crate native_windows_gui as nwg;
use nwg::subclass_control_layout;

pub struct TestLayout {
    layout: nwg::FlexboxLayout,
    edit: nwg::TextInput,
    custom_data: String,
}

subclass_control_layout!(TestLayout, TextInput, edit, FlexboxLayout, layout);
```
*/

#[macro_export]
macro_rules! subclass_control_layout {
  ( $ty:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?,
    $base_type:ident $(< $( $blt:tt ),+ >)?,
    $field:ident,
    $layout_type:ident,
    $layout_field:ident $(.$subfield:tt)*) => {
        $crate::subclass_control_base!($ty$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?, $base_type$(< $( $blt ),+ >)?, $field);
        $crate::subclass_layout!($ty$(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?, $layout_type, $layout_field$(.$subfield)*);
    };
}

handles!(Window);
handles!(Button);
handles!(GroupBox);
handles!(ImageFrame);
handles!(Label);
handles!(CheckBox);
handles!(RadioButton);
handles!(TextInput);

#[cfg(feature = "textbox")]
use super::TextBox;

#[cfg(feature = "textbox")]
handles!(TextBox);

#[cfg(feature = "status-bar")]
use super::StatusBar;

#[cfg(feature = "status-bar")]
handles!(StatusBar);

#[cfg(feature = "tooltip")]
use super::Tooltip;

#[cfg(feature = "tooltip")]
handles!(Tooltip);

#[cfg(feature = "trackbar")]
use super::TrackBar;

#[cfg(feature = "trackbar")]
handles!(TrackBar);

#[cfg(feature = "menu")]
use super::{Menu, MenuItem, MenuSeparator};

#[cfg(feature = "menu")]
handles!(Menu);
#[cfg(feature = "menu")]
handles!(MenuItem);
#[cfg(feature = "menu")]
handles!(MenuSeparator);

#[cfg(feature = "combobox")]
use super::ComboBox;

#[cfg(feature = "combobox")]
handles!(ComboBox<D: Display + Default>);

#[cfg(feature = "combobox")]
partial_eq!(ComboBox<D: Display + Default>);

#[cfg(all(feature = "combobox", feature = "labeled"))]
use super::LabeledCombo;

#[cfg(all(feature = "combobox", feature = "labeled"))]
subclass_control_layout!(LabeledCombo<D: Display + Default>, ComboBox<D>, field, FlexboxLayout, layout);

#[cfg(feature = "labeled")]
use super::LabeledEdit;

#[cfg(feature = "labeled")]
subclass_control_layout!(LabeledEdit, TextInput, field, FlexboxLayout, layout);

#[cfg(feature = "listbox")]
use super::ListBox;

#[cfg(feature = "listbox")]
handles!(ListBox<D: Display + Default>);

#[cfg(feature = "tabs")]
use super::{Tab, TabsContainer};

#[cfg(feature = "tabs")]
handles!(TabsContainer);

#[cfg(feature = "tabs")]
handles!(Tab);

#[cfg(feature = "datetime-picker")]
use super::DatePicker;

#[cfg(feature = "datetime-picker")]
handles!(DatePicker);

#[cfg(feature = "progress-bar")]
use super::ProgressBar;

#[cfg(feature = "progress-bar")]
handles!(ProgressBar);

#[cfg(feature = "tree-view")]
use super::TreeView;

#[cfg(feature = "tree-view")]
handles!(TreeView);

#[cfg(feature = "tray-notification")]
use super::TrayNotification;

#[cfg(feature = "tray-notification")]
handles!(TrayNotification);

#[cfg(feature = "message-window")]
use super::MessageWindow;

#[cfg(feature = "message-window")]
handles!(MessageWindow);

#[cfg(feature = "animation-timer")]
use super::AnimationTimer;

#[cfg(feature = "animation-timer")]
handles!(AnimationTimer);

#[cfg(feature = "notice")]
use super::Notice;

#[cfg(feature = "notice")]
handles!(Notice);

#[cfg(feature = "list-view")]
use super::ListView;

#[cfg(feature = "list-view")]
handles!(ListView);

#[cfg(feature = "extern-canvas")]
use super::ExternCanvas;

#[cfg(feature = "extern-canvas")]
handles!(ExternCanvas);

#[cfg(feature = "frame")]
use super::Frame;

#[cfg(feature = "frame")]
handles!(Frame);

#[cfg(feature = "rich-textbox")]
use super::RichTextBox;

#[cfg(feature = "rich-textbox")]
handles!(RichTextBox);

#[cfg(feature = "rich-textbox")]
use super::RichLabel;

#[cfg(feature = "rich-textbox")]
handles!(RichLabel);

#[cfg(feature = "scroll-bar")]
use super::ScrollBar;

#[cfg(feature = "scroll-bar")]
handles!(ScrollBar);

#[cfg(feature = "number-select")]
use super::NumberSelect;

#[cfg(feature = "number-select")]
handles!(NumberSelect);

#[cfg(feature = "plotting")]
use super::Plotters;

#[cfg(feature = "plotting")]
handles!(Plotters);
