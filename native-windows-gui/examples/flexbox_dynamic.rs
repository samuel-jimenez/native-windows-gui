/*!
    Shows how to add controls dynamically into a flexbox layout

    `cargo run --example flexbox_dynamic --features "flexbox"`
*/

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use std::cell::RefCell;

use nwg::{NativeUi, taffy};
use taffy::{
    geometry::Size,
    style::*,
    style_helpers::{auto, length},
};

#[derive(Default)]
pub struct FlexboxDynamic {
    window: nwg::Window,
    layout: nwg::FlexboxLayout,
    buttons: RefCell<Vec<nwg::Button>>,
}

mod flexbox_dynamic_ui {
    extern crate native_windows_gui as nwg;
    use std::{cell::RefCell, fmt, ops::Deref, rc::Rc};

    use nwg::*;

    use super::*;
    pub struct FlexboxDynamicUi {
        inner: Rc<FlexboxDynamic>,
        default_handlers: RefCell<Vec<EventHandler>>,
    }
    impl NativeUi<FlexboxDynamicUi> for FlexboxDynamic {
        fn build_ui(mut data: Self) -> Result<FlexboxDynamicUi, NwgError> {
            Window::builder()
                .size((300, 500))
                .position((400, 200))
                .title("Flexbox example")
                .build(&mut data.window)?;
            let inner = Rc::new(data);
            let ui = FlexboxDynamicUi {
                inner: inner.clone(),
                default_handlers: Default::default(),
            };
            let window_handles: &[&ControlHandle] = &[&ui.window.handle];
            for handle in window_handles.iter() {
                let evt_ui = Rc::downgrade(&inner);
                let handle_events = move |_evt, _evt_data, _handle| {
                    if let Some(evt_ui) = evt_ui.upgrade() {
                        match _evt {
                            Event::OnWindowClose => {
                                if &_handle == &evt_ui.window {
                                    nwg::stop_thread_dispatch()
                                }
                            }
                            Event::OnInit => {
                                if &_handle == &evt_ui.window {
                                    FlexboxDynamic::setup(&evt_ui)
                                }
                            }
                            _ => {}
                        }
                    }
                };
                ui.default_handlers
                    .borrow_mut()
                    .push(full_bind_event_handler(handle, handle_events));
            }
            FlexboxLayout::builder()
                .parent(&ui.window)
                .flex_direction(FlexDirection::Column)
                .build(&ui.layout)?;
            Ok(ui)
        }
    }
    impl Drop for FlexboxDynamicUi {
        /// To make sure that everything is freed without issues, the default handler must be unbound.
        fn drop(&mut self) {
            let mut handlers = self.default_handlers.borrow_mut();
            for handler in handlers.drain(0..) {
                nwg::unbind_event_handler(&handler);
            }
        }
    }
    impl Deref for FlexboxDynamicUi {
        type Target = FlexboxDynamic;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl fmt::Debug for FlexboxDynamicUi {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(format_args!("[#ui_struct_name Ui]"))
        }
    }
}
impl FlexboxDynamic {
    fn setup(&self) {
        let mut buttons = self.buttons.borrow_mut();
        for i in 0..20 {
            buttons.push(nwg::Button::default());
            let button_index = buttons.len() - 1;
            nwg::Button::builder()
                .text(&format!("Button {}", i + 1))
                .parent(&self.window)
                .build(&mut buttons[button_index])
                .expect("Failed to create button");
            let style = Style {
                size: Size {
                    width: auto(),
                    height: length(100.0),
                },
                justify_content: Some(JustifyContent::Center),
                ..Default::default()
            };
            self.layout
                .add_child(&buttons[button_index], style)
                .expect("Failed to add button to layout");
        }
    }
}
fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let _app = FlexboxDynamic::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
