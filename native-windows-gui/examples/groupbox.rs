/*!
    A very simple application that show how to use a groupbox.

    Requires the following features: `cargo run --example groupbox --features "flexbox"`
*/

extern crate native_windows_gui as nwg;
use nwg::NativeUi;

#[derive(Default)]
pub struct FlexBoxApp {
    window: nwg::Window,
    layout: nwg::FlexboxLayout,
    sub_layout: nwg::FlexboxLayout,
    button0: nwg::Button,
    groupbox: nwg::GroupBox,
    button1: nwg::Button,
    button2: nwg::Button,
    button3: nwg::Button,
    button4: nwg::Button,
    button5: nwg::Button,
}

impl FlexBoxApp {
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

//
// ALL of this stuff is handled by native-windows-derive
//
mod flexbox_app_ui {
    use std::{cell::RefCell, ops::Deref, rc::Rc};

    use native_windows_gui as nwg;
    use nwg::{Button, FlexboxLayout, GroupBox, Window};

    use super::*;

    pub struct FlexBoxAppUi {
        inner: Rc<FlexBoxApp>,
        default_handler: RefCell<Option<nwg::EventHandler>>,
    }

    impl nwg::NativeUi<FlexBoxAppUi> for FlexBoxApp {
        fn build_ui(mut data: FlexBoxApp) -> Result<FlexBoxAppUi, nwg::NwgError> {
            use nwg::Event as E;

            // Controls
            Window::builder()
                .size((500, 300))
                .position((300, 300))
                .title("GroupBox example")
                .build(&mut data.window)?;
            Button::builder()
                .text("Btn 0")
                .parent(&data.window)
                .build(&mut data.button0)?;
            GroupBox::builder()
                .text("Group!Box")
                .parent(&data.window)
                .build(&mut data.groupbox)?;
            Button::builder()
                .parent(&data.groupbox)
                .text("Btn 1")
                .build(&mut data.button1)?;
            Button::builder()
                .parent(&data.groupbox)
                .text("Btn 2")
                .build(&mut data.button2)?;
            Button::builder()
                .parent(&data.groupbox)
                .text("Btn 3")
                .build(&mut data.button3)?;
            Button::builder()
                .parent(&data.groupbox)
                .text("Btn 4")
                .build(&mut data.button4)?;
            Button::builder()
                .parent(&data.groupbox)
                .text("Btn 5")
                .build(&mut data.button5)?;

            // Wrap-up
            let ui = FlexBoxAppUi {
                inner: Rc::new(data),
                default_handler: Default::default(),
            };

            // Events
            let evt_ui = Rc::downgrade(&ui.inner);
            let handle_events = move |evt, _evt_data, handle| {
                if let Some(evt_ui) = evt_ui.upgrade() {
                    match evt {
                        E::OnWindowClose => {
                            if &handle == &evt_ui.window {
                                FlexBoxApp::exit(&evt_ui);
                            }
                        }
                        _ => {}
                    }
                }
            };

            *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
                &ui.window.handle,
                handle_events,
            ));

            // Layout
            use nwg::taffy::{
                Dimension, LengthPercentage, LengthPercentageAuto,
                geometry::{Rect, Size},
                style::FlexDirection,
                style_helpers::{auto, length, percent},
            };
            const FIFTY_PC: Dimension = Dimension::percent(0.5);
            const PT_10: LengthPercentage = LengthPercentage::length(10.0);
            const PT_5: LengthPercentageAuto = LengthPercentageAuto::length(5.0);
            // this obviously will be font dependent.
            // TODO add a builder flag to auto-size client area.
            const PT_35: LengthPercentage = LengthPercentage::length(35.0);

            const GROUP_PADDING: Rect<LengthPercentage> = Rect {
                left: PT_10,
                right: PT_10,
                top: PT_35,
                bottom: PT_10,
            };
            const PADDING: Rect<LengthPercentage> = Rect {
                left: PT_10,
                right: PT_10,
                top: PT_10,
                bottom: PT_10,
            };
            const MARGIN: Rect<LengthPercentageAuto> = Rect {
                left: PT_5,
                right: PT_5,
                top: PT_5,
                bottom: PT_5,
            };
            FlexboxLayout::builder()
                .parent(&ui.groupbox)
                .flex_direction(FlexDirection::Column)
                .padding(GROUP_PADDING)
                .child(&ui.button1)
                .child_margin(MARGIN)
                .child_size(Size {
                    width: auto(),
                    height: percent(0.25),
                })
                .child(&ui.button2)
                .child_margin(MARGIN)
                .child_size(Size {
                    width: auto(),
                    height: percent(0.25),
                })
                .child(&ui.button3)
                .child_margin(MARGIN)
                .child_flex_grow(2.0)
                .child_size(Size {
                    width: auto(),
                    height: percent(0.25),
                })
                .child(&ui.button4)
                .child_margin(MARGIN)
                .child_size(Size {
                    width: auto(),
                    height: percent(0.25),
                })
                .child(&ui.button5)
                .child_margin(MARGIN)
                .child_size(Size {
                    width: auto(),
                    height: percent(0.25),
                })
                .build(&ui.sub_layout)?;
            FlexboxLayout::builder()
                .parent(&ui.window)
                .flex_direction(FlexDirection::Row)
                .padding(PADDING)
                .child(&ui.button0)
                .child_margin(MARGIN)
                .child_max_size(Size {
                    width: length(200.0),
                    height: auto(),
                })
                .child_size(Size {
                    width: FIFTY_PC,
                    height: auto(),
                })
                .child(&ui.groupbox)
                .child_margin(MARGIN)
                .child_flex_grow(2.0)
                .child_size(Size {
                    width: auto(),
                    height: auto(),
                })
                .build(&ui.layout)?;

            return Ok(ui);
        }
    }

    impl Drop for FlexBoxAppUi {
        /// To make sure that everything is freed without issues, the default handler must be unbound.
        fn drop(&mut self) {
            let handler = self.default_handler.borrow();
            if handler.is_some() {
                nwg::unbind_event_handler(handler.as_ref().unwrap());
            }
        }
    }

    impl Deref for FlexBoxAppUi {
        type Target = FlexBoxApp;

        fn deref(&self) -> &FlexBoxApp {
            &self.inner
        }
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _ui = FlexBoxApp::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
