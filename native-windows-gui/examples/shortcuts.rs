/*!
    An application that saves messages into buttons.
    Demonstrate the dynamic functions of NWG.
 `cargo run --example shortcuts`
*/
extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
extern crate std;

use nwg::{NativeUi, ShortcutUi};

#[derive(Default)]
pub struct ShortcutGUI {
    window: nwg::Window,
    layout: nwg::GridLayout,
    add_message_btn: nwg::Button,
    message_title: nwg::TextInput,
    message_content: nwg::TextInput,
}

mod shortcut_g_u_i_ui {
    extern crate native_windows_gui as nwg;
    use std::{cell::RefCell, fmt, ops::Deref, rc::Rc};

    use nwg::*;

    use super::*;
    pub struct ShortcutGUIUi {
        inner: Rc<ShortcutGUI>,
        default_handlers: RefCell<Vec<EventHandler>>,
    }
    impl NativeUi<ShortcutGUIUi> for ShortcutGUI {
        fn build_ui(mut data: Self) -> Result<ShortcutGUIUi, NwgError> {
            Window::builder()
                .size((400, 300))
                .position((800, 300))
                .title("Shortcuts Demo")
                .build(&mut data.window)?;
            Button::builder()
                .text("&Save")
                .focus(true)
                .parent(&data.window)
                .build(&mut data.add_message_btn)?;
            TextInput::builder()
                .text("Title")
                .parent(&data.window)
                .build(&mut data.message_title)?;
            TextInput::builder()
                .text("Hello World!")
                .parent(&data.window)
                .build(&mut data.message_content)?;
            let inner = Rc::new(data);
            let ui = ShortcutGUIUi {
                inner: inner.clone(),
                default_handlers: Default::default(),
            };
            let window_handles: &[&ControlHandle] = &[&ui.window.handle];
            for handle in window_handles.iter() {
                let evt_ui = Rc::downgrade(&inner);
                let handle_events = move |_evt, _evt_data, _handle| {
                    if let Some(evt_ui) = evt_ui.upgrade() {
                        match _evt {
                            Event::OnKeyPress => {
                                if &_handle == &evt_ui.window {
                                    ShortcutGUI::func_0(&evt_ui);
                                    ShortcutGUI::do_shortcut(&evt_ui);
                                }
                            }
                            Event::OnButtonClick => {
                                if &_handle == &evt_ui.add_message_btn {
                                    ShortcutGUI::click(&evt_ui);
                                }
                            }
                            Event::OnWindowClose => {
                                if &_handle == &evt_ui.window {
                                    ShortcutGUI::exit(&evt_ui);
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
            GridLayout::builder()
                .parent(&ui.window)
                .max_row(Some(6))
                .spacing(3)
                .child_item(GridLayoutItem::new(
                    &ui.add_message_btn,
                    0u32,
                    0u32,
                    1u32,
                    1u32,
                ))
                .child_item(GridLayoutItem::new(
                    &ui.message_title,
                    1u32,
                    0u32,
                    2u32,
                    1u32,
                ))
                .child_item(GridLayoutItem::new(
                    &ui.message_content,
                    3u32,
                    0u32,
                    3u32,
                    1u32,
                ))
                .build(&ui.layout)?;
            Ok(ui)
        }
    }
    impl ShortcutUi for ShortcutGUIUi {
        fn preprocess_event(&self, _evt: &KeyCombo, _handle: ControlHandle) -> bool {
            let evt_ui = self;
            match _evt {
                KeyCombo {
                    modifiers: ModifierKeys::ALT,
                    key: KeyPress::A,
                } => {
                    if &_handle == &evt_ui.message_title {
                        ShortcutGUI::do_shortcut(&evt_ui);
                        true
                    } else {
                        false
                    }
                }
                KeyCombo {
                    modifiers: ModifierKeys::NONE,
                    key: KeyPress::U,
                } => {
                    if &_handle == &evt_ui.message_content {
                        ShortcutGUI::do_shortcut(&evt_ui);
                        true
                    } else {
                        false
                    }
                }
                KeyCombo {
                    modifiers: ModifierKeys::CTRL,
                    key: KeyPress::S,
                } => {
                    ShortcutGUI::do_global_shortcut(&evt_ui);
                    true
                }
                KeyCombo {
                    modifiers: ModifierKeys::ALT,
                    key: KeyPress::S,
                } => {
                    ShortcutGUI::do_global_shortcut(&evt_ui);
                    ShortcutGUI::do_shortcut(&evt_ui);
                    true
                }
                KeyCombo {
                    modifiers: ModifierKeys::CTRL,
                    key: KeyPress::M,
                } => {
                    if &_handle == &evt_ui.message_content {
                        ShortcutGUI::do_shortcut(&evt_ui);
                        true
                    } else {
                        ShortcutGUI::do_global_shortcut(&evt_ui);
                        true
                    }
                }
                KeyCombo {
                    modifiers: ModifierKeys::CTRL_SHIFT,
                    key: KeyPress::Plus,
                } => {
                    if &_handle == &evt_ui.message_content {
                        ShortcutGUI::do_shortcut(&evt_ui);
                        true
                    } else if &_handle == &evt_ui.add_message_btn {
                        ShortcutGUI::do_bonus_shortcut(&evt_ui, &evt_ui.add_message_btn);
                        true
                    } else {
                        false
                    }
                }
                KeyCombo {
                    modifiers: ModifierKeys::CTRL_ALT,
                    key: KeyPress::S,
                } => {
                    if &_handle == &evt_ui.message_title {
                        ShortcutGUI::do_text_shortcut(&evt_ui, &evt_ui.message_title);
                        true
                    } else {
                        false
                    }
                }
                KeyCombo {
                    modifiers: ModifierKeys::CTRL,
                    key: KeyPress::P,
                } => {
                    if &_handle == &evt_ui.message_title {
                        ShortcutGUI::do_shortcut(&evt_ui);
                        true
                    } else if &_handle == &evt_ui.add_message_btn {
                        ShortcutGUI::do_shortcut(&evt_ui);
                        true
                    } else if &_handle == &evt_ui.message_content {
                        ShortcutGUI::do_shortcut(&evt_ui);
                        true
                    } else {
                        ShortcutGUI::do_global_shortcut(&evt_ui);
                        true
                    }
                }
                KeyCombo {
                    modifiers: ModifierKeys::CTRL_ALT,
                    key: KeyPress::P,
                } => {
                    if &_handle == &evt_ui.message_content {
                        ShortcutGUI::do_text_shortcut(&evt_ui, &evt_ui.message_content);
                        true
                    } else {
                        false
                    }
                }
                KeyCombo {
                    modifiers: ModifierKeys::CTRL_SHIFT,
                    key: KeyPress::S,
                } => {
                    if &_handle == &evt_ui.message_content {
                        ShortcutGUI::do_bonus_shortcut(&evt_ui, &evt_ui.message_content);
                        true
                    } else if &_handle == &evt_ui.message_title {
                        ShortcutGUI::do_bonus_shortcut(&evt_ui, &evt_ui.message_title);
                        true
                    } else {
                        false
                    }
                }
                KeyCombo {
                    modifiers: ModifierKeys::NONE,
                    key: KeyPress::Key0,
                } => {
                    if &_handle == &evt_ui.add_message_btn {
                        ShortcutGUI::do_shortcut(&evt_ui);
                        true
                    } else {
                        false
                    }
                }
                KeyCombo {
                    modifiers: ModifierKeys::NONE,
                    key: KeyPress::NumpadPlus,
                } => {
                    if &_handle == &evt_ui.message_content {
                        ShortcutGUI::do_shortcut(&evt_ui);
                        true
                    } else {
                        false
                    }
                }
                KeyCombo {
                    modifiers: ModifierKeys::CTRL,
                    key: KeyPress::G,
                } => {
                    ShortcutGUI::do_global_shortcut(&evt_ui);
                    true
                }
                KeyCombo {
                    modifiers: ModifierKeys::NONE,
                    key: KeyPress::NumpadMinus,
                } => {
                    if &_handle == &evt_ui.message_title {
                        ShortcutGUI::do_shortcut(&evt_ui);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
    }

    impl Drop for ShortcutGUIUi {
        /// To make sure that everything is freed without issues, the default handler must be unbound.
        fn drop(&mut self) {
            let mut handlers = self.default_handlers.borrow_mut();
            for handler in handlers.drain(0..) {
                nwg::unbind_event_handler(&handler);
            }
        }
    }
    impl Deref for ShortcutGUIUi {
        type Target = ShortcutGUI;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl fmt::Debug for ShortcutGUIUi {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(format_args!("[#ui_struct_name Ui]"))
        }
    }
}

impl ShortcutGUI {
    fn func_0(&self) {
        println!("A function!");
    }

    fn click(&self) {
        println!("Click.");
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn do_shortcut(&self) {
        println!("shortcut press!");
    }

    fn do_text_shortcut(&self, control: &nwg::TextInput) {
        println!("Tezt shortcut press: {}!", control.text());
    }

    fn do_bonus_shortcut<C: Into<nwg::ControlHandle>>(&self, control: C) {
        println!("BBBBONUS shortcut press from {:?}!", control.into());
    }

    fn do_global_shortcut(&self) {
        println!(" GLOBAL shortcut press!");
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let ui = ShortcutGUI::build_ui(Default::default()).expect("Failed to build UI");
    ui.dispatch_thread_events(); // requires ShortcutUi trait
}
