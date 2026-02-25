/*!
    An application that load different interfaces using the partial feature.
    Partials can be used to split large GUI application into smaller bits.

    Requires the following features: `cargo run --example partials --features "listbox frame combobox flexbox"`
*/

extern crate native_windows_gui as nwg;
use nwg::{NativeUi, ShortcutUi};

#[derive(Default)]
pub struct PartialDemo {
    window: nwg::Window,
    layout: nwg::FlexboxLayout,
    menu: nwg::ListBox<&'static str>,
    frame1: nwg::Frame,
    frame2: nwg::Frame,
    frame3: nwg::Frame,

    people_ui: PeopleUi,
    animal_ui: AnimalUi,
    food_ui: FoodUi,
}

//
// ALL of this stuff is handled by native-windows-derive
//
mod partial_demo_ui {
    extern crate native_windows_gui as nwg;
    use std::{cell::RefCell, fmt, ops::Deref, rc::Rc};

    use nwg::*;

    use super::*;

    pub struct PartialDemoUi {
        inner: Rc<PartialDemo>,
        default_handlers: RefCell<Vec<EventHandler>>,
    }
    impl NativeUi<PartialDemoUi> for PartialDemo {
        fn build_ui(mut data: Self) -> Result<PartialDemoUi, NwgError> {
            // Controls
            Window::builder()
                .size((500, 400))
                .position((300, 300))
                .title("Many UI")
                .build(&mut data.window)?;

            ListBox::builder()
                .collection(vec!["People", "Animals", "Food"])
                .focus(true)
                .parent(&data.window)
                .build(&mut data.menu)?;

            Frame::builder()
                .parent(&data.window)
                .build(&mut data.frame1)?;

            Frame::builder()
                .flags(FrameFlags::BORDER)
                .parent(&data.window)
                .build(&mut data.frame2)?;

            Frame::builder()
                .flags(FrameFlags::BORDER)
                .parent(&data.window)
                .build(&mut data.frame3)?;

            // Partials
            PeopleUi::build_partial(&mut data.people_ui, Some(&data.frame1), false)?;
            AnimalUi::build_partial(&mut data.animal_ui, Some(&data.frame2), false)?;
            FoodUi::build_partial(&mut data.food_ui, Some(&data.frame3), false)?;
            let inner = Rc::new(data);
            let ui = PartialDemoUi {
                inner: inner.clone(),
                default_handlers: Default::default(),
            };
            let window_handles: &[&ControlHandle] = &[&ui.window.handle];
            for handle in window_handles.iter() {
                let evt_ui = Rc::downgrade(&inner);
                let handle_events = move |_evt, _evt_data, _handle| {
                    if let Some(evt_ui) = evt_ui.upgrade() {
                        evt_ui.people_ui.process_event(_evt, &_evt_data, _handle);
                        evt_ui.animal_ui.process_event(_evt, &_evt_data, _handle);
                        evt_ui.food_ui.process_event(_evt, &_evt_data, _handle);
                        match _evt {
                            Event::OnButtonClick => {
                                if &_handle == &evt_ui.animal_ui.save_btn
                                    || &_handle == &evt_ui.food_ui.save_btn
                                    || &_handle == &evt_ui.people_ui.save_btn
                                {
                                    PartialDemo::save(&evt_ui);
                                }
                            }
                            Event::OnListBoxSelect => {
                                if &_handle == &evt_ui.menu {
                                    PartialDemo::change_interface(&evt_ui);
                                }
                            }
                            Event::OnWindowClose => {
                                if &_handle == &evt_ui.window {
                                    PartialDemo::exit(&evt_ui);
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
                .child(&ui.menu)
                .child(&ui.frame1)
                .build(&ui.layout)?;
            Ok(ui)
        }
    }
    impl ShortcutUi for PartialDemoUi {
        fn preprocess_event(&self, _evt: &KeyCombo, _handle: ControlHandle) -> bool {
            let evt_ui = self;
            evt_ui.people_ui.preprocess_event(_evt, _handle)
                || evt_ui.animal_ui.preprocess_event(_evt, _handle)
                || evt_ui.food_ui.preprocess_event(_evt, _handle)
                || match _evt {
                    KeyCombo {
                        modifiers: ModifierKeys::CTRL,
                        key: KeyPress::Key0,
                    } => {
                        if &_handle == &evt_ui.people_ui.save_btn {
                            PartialDemo::do_shortcut(&evt_ui);
                            true
                        } else {
                            false
                        }
                    }
                    KeyCombo {
                        modifiers: ModifierKeys::NONE,
                        key: KeyPress::Key0,
                    } => {
                        if &_handle == &evt_ui.people_ui.save_btn {
                            PartialDemo::do_shortcut(&evt_ui);
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
        }
    }
    impl Drop for PartialDemoUi {
        /// To make sure that everything is freed without issues, the default handler must be unbound.
        fn drop(&mut self) {
            let mut handlers = self.default_handlers.borrow_mut();
            for handler in handlers.drain(0..) {
                nwg::unbind_event_handler(&handler);
            }
        }
    }
    impl Deref for PartialDemoUi {
        type Target = PartialDemo;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl fmt::Debug for PartialDemoUi {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(format_args!("[#ui_struct_name Ui]"))
        }
    }
}

impl PartialDemo {
    fn change_interface(&self) {
        self.frame1.set_visible(false);
        self.frame2.set_visible(false);
        self.frame3.set_visible(false);

        let layout = &self.layout;
        if layout.has_child(&self.frame1) {
            layout.remove_child(&self.frame1);
        }
        if layout.has_child(&self.frame2) {
            layout.remove_child(&self.frame2);
        }
        if layout.has_child(&self.frame3) {
            layout.remove_child(&self.frame3);
        }

        use nwg::taffy::{
            geometry::Size,
            style::Style,
            style_helpers::{auto, percent},
        };
        let mut style = Style::default();
        style.size = Size {
            width: percent(1.0),
            height: auto(),
        };

        match self.menu.selection() {
            None | Some(0) => {
                layout.add_child(&self.frame1, style).unwrap();
                self.frame1.set_visible(true);
            }
            Some(1) => {
                layout.add_child(&self.frame2, style).unwrap();
                self.frame2.set_visible(true);
            }
            Some(2) => {
                layout.add_child(&self.frame3, style).unwrap();
                self.frame3.set_visible(true);
            }
            Some(_) => unreachable!(),
        }
    }

    fn save(&self) {
        nwg::simple_message("Saved!", "Data saved!");
    }

    fn do_shortcut(&self) {
        println!("Partial shortcut press!");
    }
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

#[derive(Default)]
pub struct PeopleUi {
    layout: nwg::GridLayout,
    layout2: nwg::GridLayout,

    label1: nwg::Label,
    label2: nwg::Label,
    label3: nwg::Label,

    name_input: nwg::TextInput,
    age_input: nwg::TextInput,
    job_input: nwg::TextInput,

    save_btn: nwg::Button,
}

//
// ALL of this stuff is handled by native-windows-derive
//
mod partial_people_ui_ui {
    extern crate native_windows_gui as nwg;
    use nwg::*;

    use super::*;
    impl PartialUi for PeopleUi {
        #[allow(unused)]
        fn build_partial<W: Into<ControlHandle>>(
            data: &mut Self,
            _parent: Option<W>,
            expand_layout_p: bool,
        ) -> Result<(), NwgError> {
            let parent = _parent.map(|p| p.into());
            let parent_ref = parent.as_ref();
            Label::builder()
                .text("Name:")
                .h_align(HTextAlign::Right)
                .parent(parent_ref.unwrap())
                .build(&mut data.label1)?;
            Label::builder()
                .text("Age:")
                .h_align(HTextAlign::Right)
                .parent(parent_ref.unwrap())
                .build(&mut data.label2)?;
            Label::builder()
                .text("Job:")
                .h_align(HTextAlign::Right)
                .parent(parent_ref.unwrap())
                .build(&mut data.label3)?;
            TextInput::builder()
                .text("John Doe")
                .parent(parent_ref.unwrap())
                .build(&mut data.name_input)?;
            TextInput::builder()
                .text("75")
                .number(true)
                .visible(true)
                .parent(parent_ref.unwrap())
                .build(&mut data.age_input)?;
            TextInput::builder()
                .text("Programmer")
                .parent(parent_ref.unwrap())
                .build(&mut data.job_input)?;
            Button::builder()
                .text("Save")
                .parent(parent_ref.unwrap())
                .build(&mut data.save_btn)?;

            let ui = data;
            GridLayout::builder()
                .min_size([100, 200])
                .max_column(Some(2))
                .max_row(Some(6))
                .parent(parent_ref.unwrap())
                .child(1, 5, &ui.save_btn)
                .build(&ui.layout2)?;
            GridLayout::builder()
                .max_size([1000, 150])
                .min_size([100, 120])
                .parent(parent_ref.unwrap())
                .child(0, 0, &ui.label1)
                .child(0, 1, &ui.label2)
                .child(0, 2, &ui.label3)
                .child(1, 0, &ui.name_input)
                .child(1, 1, &ui.age_input)
                .child(1, 2, &ui.job_input)
                .build(&ui.layout)?;
            Ok(())
        }
        fn preprocess_event(&self, _evt: &KeyCombo, _handle: ControlHandle) -> bool {
            let evt_ui = self;
            match _evt {
                KeyCombo {
                    modifiers: ModifierKeys::NONE,
                    key: KeyPress::Key0,
                } => {
                    if &_handle == &evt_ui.job_input {
                        PeopleUi::do_shortcut(&evt_ui);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }
        fn process_event<'a>(&self, _evt: Event, _evt_data: &EventData, _handle: ControlHandle) {
            let evt_ui = self;
            match _evt {
                Event::OnChar => {
                    if &_handle == &evt_ui.name_input {
                        print_char(&_evt_data);
                    }
                }
                _ => {}
            }
        }
        fn handles(&self) -> Vec<&ControlHandle> {
            Vec::new()
        }
    }
}

impl PeopleUi {
    fn do_shortcut(&self) {
        println!("shortcut press!");
    }
}

#[derive(Default)]
pub struct AnimalUi {
    layout: nwg::GridLayout,
    layout2: nwg::GridLayout,

    label1: nwg::Label,
    label2: nwg::Label,
    label3: nwg::Label,

    name_input: nwg::TextInput,
    race_input: nwg::ComboBox<&'static str>,
    is_soft_input: nwg::CheckBox,

    save_btn: nwg::Button,
}

//
// ALL of this stuff is handled by native-windows-derive
//
mod partial_animal_ui_ui {
    extern crate native_windows_gui as nwg;
    use nwg::*;

    use super::*;
    impl PartialUi for AnimalUi {
        #[allow(unused)]
        fn build_partial<W: Into<ControlHandle>>(
            data: &mut Self,
            _parent: Option<W>,
            expand_layout_p: bool,
        ) -> Result<(), NwgError> {
            let parent = _parent.map(|p| p.into());
            let parent_ref = parent.as_ref();
            Label::builder()
                .text("Name:")
                .h_align(HTextAlign::Right)
                .parent(parent_ref.unwrap())
                .build(&mut data.label1)?;
            Label::builder()
                .text("Race:")
                .h_align(HTextAlign::Right)
                .parent(parent_ref.unwrap())
                .build(&mut data.label2)?;
            Label::builder()
                .text("Is fluffy:")
                .h_align(HTextAlign::Right)
                .parent(parent_ref.unwrap())
                .build(&mut data.label3)?;
            TextInput::builder()
                .text("Mittens")
                .parent(parent_ref.unwrap())
                .build(&mut data.name_input)?;
            ComboBox::builder()
                .collection(vec!["Cat", "Dog", "Pigeon", "Monkey"])
                .selected_index(Some(0))
                .parent(parent_ref.unwrap())
                .build(&mut data.race_input)?;
            CheckBox::builder()
                .text("")
                .check_state(CheckBoxState::Checked)
                .parent(parent_ref.unwrap())
                .build(&mut data.is_soft_input)?;
            Button::builder()
                .text("Save")
                .parent(parent_ref.unwrap())
                .build(&mut data.save_btn)?;
            let ui = data;
            GridLayout::builder()
                .min_size([100, 200])
                .max_column(Some(2))
                .max_row(Some(6))
                .parent(parent_ref.unwrap())
                .child_item(GridLayoutItem::new(&ui.save_btn, 1u32, 5u32, 1u32, 1u32))
                .build_conditional(&ui.layout2, expand_layout_p)?;
            GridLayout::builder()
                .max_size([1000, 150])
                .min_size([100, 120])
                .parent(parent_ref.unwrap())
                .child_item(GridLayoutItem::new(&ui.label1, 0u32, 0u32, 1u32, 1u32))
                .child_item(GridLayoutItem::new(&ui.label2, 0u32, 1u32, 1u32, 1u32))
                .child_item(GridLayoutItem::new(&ui.label3, 0u32, 2u32, 1u32, 1u32))
                .child_item(GridLayoutItem::new(&ui.name_input, 1u32, 0u32, 1u32, 1u32))
                .child_item(GridLayoutItem::new(&ui.race_input, 1u32, 1u32, 1u32, 1u32))
                .child_item(GridLayoutItem::new(
                    &ui.is_soft_input,
                    1u32,
                    2u32,
                    1u32,
                    1u32,
                ))
                .build_conditional(&ui.layout, expand_layout_p)?;
            Ok(())
        }
        fn process_event<'a>(&self, _evt: Event, _evt_data: &EventData, _handle: ControlHandle) {
            let evt_ui = self;
            match _evt {
                Event::OnChar => {
                    if &_handle == &evt_ui.name_input {
                        print_char(&_evt_data);
                    }
                }
                _ => {}
            }
        }
        fn handles(&self) -> Vec<&ControlHandle> {
            Vec::new()
        }
    }
}

#[derive(Default)]
pub struct FoodUi {
    layout: nwg::GridLayout,
    layout2: nwg::GridLayout,

    label1: nwg::Label,
    label2: nwg::Label,

    name_input: nwg::TextInput,
    tasty_input: nwg::CheckBox,

    save_btn: nwg::Button,
}

//
// ALL of this stuff is handled by native-windows-derive
//
mod partial_food_ui_ui {
    extern crate native_windows_gui as nwg;
    use nwg::*;

    use super::*;
    impl PartialUi for FoodUi {
        #[allow(unused)]
        fn build_partial<W: Into<ControlHandle>>(
            data: &mut Self,
            _parent: Option<W>,
            expand_layout_p: bool,
        ) -> Result<(), NwgError> {
            let parent = _parent.map(|p| p.into());
            let parent_ref = parent.as_ref();
            Label::builder()
                .text("Name:")
                .h_align(HTextAlign::Right)
                .parent(parent_ref.unwrap())
                .build(&mut data.label1)?;
            Label::builder()
                .text("Tasty:")
                .h_align(HTextAlign::Right)
                .parent(parent_ref.unwrap())
                .build(&mut data.label2)?;
            TextInput::builder()
                .text("Banana")
                .parent(parent_ref.unwrap())
                .build(&mut data.name_input)?;
            CheckBox::builder()
                .text("")
                .check_state(CheckBoxState::Checked)
                .parent(parent_ref.unwrap())
                .build(&mut data.tasty_input)?;
            Button::builder()
                .text("Save")
                .parent(parent_ref.unwrap())
                .build(&mut data.save_btn)?;

            let ui = data;
            GridLayout::builder()
                .min_size([100, 200])
                .max_column(Some(2))
                .max_row(Some(6))
                .parent(parent_ref.unwrap())
                .child(1, 5, &ui.save_btn)
                .build(&ui.layout2)?;
            GridLayout::builder()
                .max_size([1000, 90])
                .min_size([100, 80])
                .parent(parent_ref.unwrap())
                .child(0, 0, &ui.label1)
                .child(0, 1, &ui.label2)
                .child(1, 0, &ui.name_input)
                .child(1, 1, &ui.tasty_input)
                .build(&ui.layout)?;
            Ok(())
        }
        fn process_event<'a>(&self, _evt: Event, _evt_data: &EventData, _handle: ControlHandle) {
            let evt_ui = self;
            match _evt {
                Event::OnChar => {
                    if &_handle == &evt_ui.name_input {
                        print_char(&_evt_data);
                    }
                }
                _ => {}
            }
        }
        fn handles(&self) -> Vec<&ControlHandle> {
            Vec::new()
        }
    }
}

fn print_char(data: &nwg::EventData) {
    println!("{:?}", data.on_char());
}
fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let ui = PartialDemo::build_ui(Default::default()).expect("Failed to build UI");
    ui.dispatch_thread_events(); // requires ShortcutUi trait
}
