#![allow(unused)]
use std::cell::RefCell;

use winapi::{
    shared::windef::HBRUSH,
    um::winuser::{
        BS_GROUPBOX, WM_ERASEBKGND, WM_SIZE, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
        WS_DISABLED, WS_EX_CONTROLPARENT, WS_GROUP, WS_TABSTOP, WS_VISIBLE,
    },
};

use super::{ControlBase, ControlHandle};
use crate::{
    Font, Frame, NwgError, RawEventHandler,
    win32::{base_helper::check_hwnd, window_helper as wh},
};

const NOT_BOUND: &'static str = "GroupBox is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: GroupBox handle is not HWND!";

/**
A group box is a rectangle containing an application-defined text label.
So you can't nest buttons (ok).
And windows thinks GroupBox are buttons (bad thing).
So, begin the hoops to jump through...

GroupBox is not behind any features.

**Builder parameters:**
  * `parent`:   **Required.** The group box parent container.
  * `text`:     The group box text.
  * `size`:     The group box size.
  * `position`: The group box position.
  * `flags`:    A combination of the GroupBoxFlags values.
  * `ex_flags`: A combination of win32 window extended flags. Unlike `flags`, ex_flags must be used straight from winapi
  * `font`:     The font used for the group box text

**Control events:**
  * `MousePress(_)`: Generic mouse press events on the group box
  * `OnMouseMove`: Generic mouse mouse event
  * `OnMouseWheel`: Generic mouse wheel event

```rust
use native_windows_gui as nwg;
fn build_group box(group box: &mut nwg::GroupBox, window: &nwg::Window, font: &nwg::Font) {
    nwg::GroupBox::builder()
        .text("Hello")
        .font(Some(font))
        .parent(window)
        .build(group box);
}
```

*/
#[derive(Default)]
pub struct GroupBox {
    pub handle: ControlHandle,
    groupbox_w32: _GroupBox,
    handler0: RefCell<Option<RawEventHandler>>,
}

impl GroupBox {
    pub fn builder<'a>() -> GroupBoxBuilder<'a> {
        GroupBoxBuilder {
            text: "GroupBox",
            size: (100, 25),
            position: (0, 0),
            visible: true,
            enabled: true,
            ex_flags: 0,
            font: None,
            parent: None,
        }
    }

    /// Returns the font of the control
    pub fn font(&self) -> Option<Font> {
        self.groupbox_w32.font()
    }

    /// Sets the font of the control
    pub fn set_font(&self, font: Option<&Font>) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::set_window_font(handle, font.map(|f| f.handle), true);
        }
        self.groupbox_w32.set_font(font)
    }

    /// Returns true if the control user can interact with the control, return false otherwise
    pub fn enabled(&self) -> bool {
        self.groupbox_w32.enabled()
    }

    /// Enable or disable the control
    pub fn set_enabled(&self, v: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_enabled(handle, v) }
        self.groupbox_w32.set_enabled(v)
    }

    /// Returns true if the control is visible to the user. Will return true even if the
    /// control is outside of the parent client view (ex: at the position (10000, 10000))
    pub fn visible(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_visibility(handle) }
    }

    /// Show or hide the control to the user
    pub fn set_visible(&self, v: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_visibility(handle, v) }
    }

    /// Returns the size of the group box in the parent window
    pub fn size(&self) -> (u32, u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_size(handle) }
    }

    /// Sets the size of the group box in the parent window
    pub fn set_size(&self, x: u32, y: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_size(handle, x, y, false) }
    }

    /// Returns the position of the group box in the parent window
    pub fn position(&self) -> (i32, i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_position(handle) }
    }

    /// Sets the position of the group box in the parent window
    pub fn set_position(&self, x: i32, y: i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_position(handle, x, y) }
    }

    /// Returns the group box label
    pub fn text(&self) -> String {
        self.groupbox_w32.text()
    }

    /// Sets the group box label
    pub fn set_text<'a>(&self, v: &'a str) {
        self.groupbox_w32.set_text(v)
    }

    /// Winapi class name used during control creation
    pub fn class_name(&self) -> &'static str {
        // "GroupBox" /?TODO
        "NWG_FRAME"
    }

    /// Winapi base flags used during window creation
    pub fn flags(&self) -> u32 {
        WS_VISIBLE
    }

    /// Winapi flags required by the control
    pub fn forced_flags(&self) -> u32 {
        WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS
    }

    /// Keep GroupBox same size as surroundng frame.
    fn hook_non_client_size(&mut self) {
        use std::mem;

        use winapi::{
            shared::windef::{POINT, RECT},
            um::winuser::{
                COLOR_WINDOW, FillRect, GetDC, GetWindowRect, ReleaseDC, ScreenToClient,
            },
        };

        use crate::bind_raw_event_handler_inner;

        if self.handle.blank() {
            panic!("{}", NOT_BOUND);
        }
        self.handle.hwnd().expect(BAD_HANDLE);

        let groupbox = self.groupbox_w32.handle.hwnd().expect(BAD_HANDLE);

        unsafe {
            let handler =
                bind_raw_event_handler_inner(&self.handle, 0, move |hwnd, msg, _w, _l| {
                    match msg {
                        WM_SIZE => {
                            let mut window: RECT = mem::zeroed();
                            GetWindowRect(hwnd, &mut window);

                            let mut bottom_right_pt = POINT {
                                x: window.right,
                                y: window.bottom,
                            };
                            ScreenToClient(hwnd, &mut bottom_right_pt);

                            wh::set_window_size(
                                groupbox,
                                bottom_right_pt.x.try_into().unwrap(),
                                bottom_right_pt.y.try_into().unwrap(),
                                false,
                            )
                        }
                        _ => {}
                    }

                    None
                });

            *self.handler0.borrow_mut() = Some(handler.unwrap());
        }
    }
}

impl Drop for GroupBox {
    fn drop(&mut self) {
        self.handle.destroy();
    }
}

pub struct GroupBoxBuilder<'a> {
    text: &'a str,
    size: (i32, i32),
    position: (i32, i32),
    visible: bool,
    enabled: bool,
    ex_flags: u32,
    font: Option<&'a Font>,
    parent: Option<ControlHandle>,
}

impl<'a> GroupBoxBuilder<'a> {
    pub fn ex_flags(mut self, flags: u32) -> GroupBoxBuilder<'a> {
        self.ex_flags = flags;
        self
    }

    pub fn text(mut self, text: &'a str) -> GroupBoxBuilder<'a> {
        self.text = text;
        self
    }

    pub fn size(mut self, size: (i32, i32)) -> GroupBoxBuilder<'a> {
        self.size = size;
        self
    }

    pub fn position(mut self, pos: (i32, i32)) -> GroupBoxBuilder<'a> {
        self.position = pos;
        self
    }

    pub fn visible(mut self, e: bool) -> GroupBoxBuilder<'a> {
        self.visible = e;
        self
    }

    pub fn enabled(mut self, e: bool) -> GroupBoxBuilder<'a> {
        self.enabled = e;
        self
    }

    pub fn font(mut self, font: Option<&'a Font>) -> GroupBoxBuilder<'a> {
        self.font = font;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> GroupBoxBuilder<'a> {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut GroupBox) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("GroupBox")),
        }?;

        // out.set_enabled TODO
        let mut flags = out.flags();
        if !self.visible {
            flags &= !WS_VISIBLE
        }
        if !self.enabled {
            flags |= WS_DISABLED
        }

        // Drop the old object
        *out = GroupBox::default();

        out.handle = ControlBase::build_hwnd()
            .class_name(out.class_name())
            .forced_flags(out.forced_flags())
            .flags(flags)
            .ex_flags(WS_EX_CONTROLPARENT | self.ex_flags)
            .size(self.size)
            .position(self.position)
            .text(self.text)
            .parent(Some(parent))
            .build()?;

        _GroupBoxBuilder {
            text: self.text,
            size: self.size,
            position: (0, 0),
            // enabled: false,
            // flags: None,
            flags: flags,

            ex_flags: 0,
            font: self.font,
            parent: Some(out.handle),
        }
        .build(&mut out.groupbox_w32)?;
        out.hook_non_client_size();

        if self.font.is_some() {
            out.set_font(self.font);
        } else {
            out.set_font(Font::global_default().as_ref());
        }

        Ok(())
    }
}

impl PartialEq for GroupBox {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

/**
This is the raw w32 group box button abomination.
You make me sad.

_GroupBox is not behind any features.

**Builder parameters:**
  * `parent`:   **Required.** The group box parent container.
  * `text`:     The group box text.
  * `size`:     The group box size.
  * `position`: The group box position.
  * `flags`:    A combination of the _GroupBoxFlags values.
  * `ex_flags`: A combination of win32 window extended flags. Unlike `flags`, ex_flags must be used straight from winapi
  * `font`:     The font used for the group box text



```rust
use native_windows_gui as nwg;
fn build_group box(group box: &mut nwg::_GroupBox, window: &nwg::Window, font: &nwg::Font) {
    nwg::_GroupBox::builder()
        .text("Hello")
        .flags(1)
        .font(Some(font))
        .parent(window)
        .build(group box);
}
```

*/
#[derive(Default)]
pub struct _GroupBox {
    pub handle: ControlHandle,
    handler0: RefCell<Option<RawEventHandler>>,
}

impl _GroupBox {
    pub fn builder<'a>() -> _GroupBoxBuilder<'a> {
        _GroupBoxBuilder {
            text: "_GroupBox",
            size: (100, 25),
            position: (0, 0),
            flags: 0,
            ex_flags: 0,
            font: None,
            parent: None,
        }
    }

    /// Returns the font of the control
    pub fn font(&self) -> Option<Font> {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);

        let font_handle = wh::get_window_font(handle);
        if font_handle.is_null() {
            None
        } else {
            Some(Font {
                handle: font_handle,
            })
        }
    }

    /// Sets the font of the control
    pub fn set_font(&self, font: Option<&Font>) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe {
            wh::set_window_font(handle, font.map(|f| f.handle), true);
        }
    }

    /// Returns true if the control user can interact with the control, return false otherwise
    pub fn enabled(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_enabled(handle) }
    }

    /// Enable or disable the control
    pub fn set_enabled(&self, v: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_enabled(handle, v) }
    }

    /// Returns true if the control is visible to the user. Will return true even if the
    /// control is outside of the parent client view (ex: at the position (10000, 10000))
    pub fn visible(&self) -> bool {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_visibility(handle) }
    }

    /// Show or hide the control to the user
    pub fn set_visible(&self, v: bool) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_visibility(handle, v) }
    }

    /// Returns the size of the group box in the parent window
    pub fn size(&self) -> (u32, u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_size(handle) }
    }

    /// Sets the size of the group box in the parent window
    pub fn set_size(&self, x: u32, y: u32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_size(handle, x, y, false) }
    }

    /// Returns the position of the group box in the parent window
    pub fn position(&self) -> (i32, i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_position(handle) }
    }

    /// Sets the position of the group box in the parent window
    pub fn set_position(&self, x: i32, y: i32) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_position(handle, x, y) }
    }

    /// Returns the group box label
    pub fn text(&self) -> String {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::get_window_text(handle) }
    }

    /// Sets the group box label
    pub fn set_text<'a>(&self, v: &'a str) {
        let handle = check_hwnd(&self.handle, NOT_BOUND, BAD_HANDLE);
        unsafe { wh::set_window_text(handle, v) }
    }

    /// Winapi class name used during control creation
    pub fn class_name(&self) -> &'static str {
        "BUTTON"
    }

    /// Winapi flags required by the control
    pub fn forced_flags(&self) -> u32 {
        // WS_CHILD | BS_GROUPBOX | WS_CLIPSIBLINGS
        WS_CHILD | BS_GROUPBOX
    }

    /// Erase the background. Because apparently that is not the default?
    fn hook_non_client_size(&mut self) {
        use std::mem;

        use winapi::{
            shared::windef::{POINT, RECT},
            um::winuser::{
                COLOR_WINDOW, FillRect, GetDC, GetWindowRect, ReleaseDC, ScreenToClient,
            },
        };

        use crate::bind_raw_event_handler_inner;

        if self.handle.blank() {
            panic!("{}", NOT_BOUND);
        }
        self.handle.hwnd().expect(BAD_HANDLE);

        let brush = COLOR_WINDOW as HBRUSH;

        unsafe {
            let handler =
                bind_raw_event_handler_inner(&self.handle, 0, move |hwnd, msg, _w, _l| {
                    match msg {
                        WM_ERASEBKGND => {
                            let mut window: RECT = mem::zeroed();
                            GetWindowRect(hwnd, &mut window);

                            let mut bottom_right_pt = POINT {
                                x: window.right,
                                y: window.bottom,
                            };
                            ScreenToClient(hwnd, &mut bottom_right_pt);

                            let erase_rect = RECT {
                                left: 0,
                                top: 0,
                                right: bottom_right_pt.x,
                                bottom: bottom_right_pt.y,
                            };

                            let dc = GetDC(hwnd);
                            FillRect(dc, &erase_rect, brush);
                            ReleaseDC(hwnd, dc);
                        }
                        _ => {}
                    }

                    None
                });

            *self.handler0.borrow_mut() = Some(handler.unwrap());
        }
    }
}

impl Drop for _GroupBox {
    fn drop(&mut self) {
        self.handle.destroy();
    }
}

pub struct _GroupBoxBuilder<'a> {
    text: &'a str,
    size: (i32, i32),
    position: (i32, i32),
    flags: u32,
    ex_flags: u32,
    font: Option<&'a Font>,
    parent: Option<ControlHandle>,
}

impl<'a> _GroupBoxBuilder<'a> {
    pub fn flags(mut self, flags: u32) -> _GroupBoxBuilder<'a> {
        self.flags = flags;
        self
    }

    pub fn ex_flags(mut self, flags: u32) -> _GroupBoxBuilder<'a> {
        self.ex_flags = flags;
        self
    }

    pub fn text(mut self, text: &'a str) -> _GroupBoxBuilder<'a> {
        self.text = text;
        self
    }

    pub fn size(mut self, size: (i32, i32)) -> _GroupBoxBuilder<'a> {
        self.size = size;
        self
    }

    pub fn position(mut self, pos: (i32, i32)) -> _GroupBoxBuilder<'a> {
        self.position = pos;
        self
    }

    pub fn font(mut self, font: Option<&'a Font>) -> _GroupBoxBuilder<'a> {
        self.font = font;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: C) -> _GroupBoxBuilder<'a> {
        self.parent = Some(p.into());
        self
    }

    pub fn build(self, out: &mut _GroupBox) -> Result<(), NwgError> {
        let parent = match self.parent {
            Some(p) => Ok(p),
            None => Err(NwgError::no_parent("_GroupBox")),
        }?;

        // Drop the old object
        *out = _GroupBox::default();

        out.handle = ControlBase::build_hwnd()
            .class_name(out.class_name())
            .forced_flags(out.forced_flags())
            .flags(self.flags)
            .ex_flags(self.ex_flags)
            .size(self.size)
            .position(self.position)
            .text(self.text)
            .parent(Some(parent))
            .build()?;

        out.hook_non_client_size();

        if self.font.is_some() {
            out.set_font(self.font);
        } else {
            out.set_font(Font::global_default().as_ref());
        }

        Ok(())
    }
}

impl PartialEq for _GroupBox {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
