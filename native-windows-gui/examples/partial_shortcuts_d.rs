/*!
    An application that saves messages into buttons.
    Demonstrate the dynamic functions of NWG.

    `cargo run --example partial_shortcuts_d`
*/

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use nwd::{NwgPartial, NwgUi};
use nwg::{NativeUi, ShortcutUi};

#[derive(Default, NwgUi)]
#[nwg_shortcuts( Ctrl+G: [ShortcutGUI::do_global_shortcut], Ctrl+S: [ShortcutGUI::do_global_shortcut], Alt+S: [ShortcutGUI::do_global_shortcut, ShortcutGUI::do_shortcut], Ctrl+P: [ShortcutGUI::do_global_shortcut], Ctrl+M: [ShortcutGUI::do_global_shortcut] )]

pub struct ShortcutGUI {
    #[nwg_control(size:(400, 300), position:(800, 300), title: "Shortcuts Demo")]
    #[nwg_events( OnWindowClose: [ShortcutGUI::exit], OnKeyPress: [ShortcutGUI::func_0,ShortcutGUI::do_shortcut,]  )]
    window: nwg::Window,

    #[nwg_layout(parent: window)]
    layout: nwg::FlexboxLayout,

    // the `&` symbol in the text field can be used to automatically tie Alt events to button presses.
    #[nwg_control(text: "&Save", focus: true)]
    #[nwg_layout_item(layout: layout)]
    #[nwg_events( OnButtonClick: [ShortcutGUI::click] )]
    #[nwg_shortcuts( Ctrl+Shift+Plus: [ShortcutGUI::do_bonus_shortcut(SELF,CTRL)], Ctrl+P: [ShortcutGUI::do_shortcut(SELF)], Key0: [ShortcutGUI::do_shortcut(SELF)] )]
    add_message_btn: nwg::Button,

    #[nwg_partial_control(parent: window)]
    #[nwg_layout_item(layout: layout)]
    #[nwg_events( (save_btn)OnButtonClick: [ShortcutGUI::click] )]
    #[nwg_shortcuts( (bard_ui.save_btn) Ctrl+Shift+S: [ShortcutGUI::do_bonus_shortcut(SELF,CTRL), ShortcutGUI::do_bonus_shortcut(SELF,TARGET)], (bard_ui.save_btn, save_btn, name_input) [Key0,Key1]: [ShortcutGUI::do_shortcut], (save_btn) Ctrl+Key0: [ShortcutGUI::do_shortcut])]
    food_ui: FoodUi,

    #[nwg_control(text:"Title")]
    #[nwg_layout_item(layout: layout)]
    #[nwg_shortcuts(  NumpadMinus: [ShortcutGUI::do_shortcut], Ctrl+Shift+S: [ShortcutGUI::do_bonus_shortcut(SELF,CTRL)], Ctrl+Alt+S: [ShortcutGUI::do_text_shortcut(SELF,CTRL)], Ctrl+P: [ShortcutGUI::do_shortcut] , Alt+A: [ShortcutGUI::do_shortcut], Key0: [ShortcutGUI::do_shortcut, ShortcutGUI::click]  )]
    message_title: nwg::TextInput,

    #[nwg_control(text:"Hello World!")]
    #[nwg_layout_item(layout: layout)]
    #[nwg_shortcuts( NumpadPlus: [ShortcutGUI::do_shortcut], U: [ShortcutGUI::do_shortcut],Ctrl+M: [ShortcutGUI::do_shortcut], Ctrl+Plus+Shift: [ShortcutGUI::do_shortcut], Ctrl+P: [ShortcutGUI::do_shortcut], Ctrl+Shift+S: [ShortcutGUI::do_bonus_shortcut(SELF,CTRL)], Ctrl+Alt+P: [ShortcutGUI::do_text_shortcut(SELF,CTRL)])]
    message_content: nwg::TextInput,
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
        println!("Text shortcut press: {}!", control.text());
    }

    fn do_bonus_shortcut<C: Into<nwg::ControlHandle>>(&self, control: C) {
        println!("BBBBONUS shortcut press from {:?}!", control.into());
    }

    fn do_global_shortcut(&self) {
        println!(" GLOBAL shortcut press!");
    }
}

#[derive(Default, NwgPartial)]
pub struct FoodUi {
    #[nwg_control]
    #[nwg_root]
    frame: nwg::Frame,

    #[nwg_layout]
    layout: nwg::FlexboxLayout,

    #[nwg_partial_control]
    #[nwg_layout_item(layout: layout)]
    bard_ui: BardUi,

    #[nwg_control(text: "Banana")]
    #[nwg_layout_item(layout: layout)]
    name_input: nwg::TextInput,

    #[nwg_control(text: "Save")]
    #[nwg_layout_item(layout: layout)]
    save_btn: nwg::Button,
}

#[derive(Default, NwgPartial)]
pub struct BardUi {
    #[nwg_control]
    #[nwg_root]
    frame: nwg::Frame,

    #[nwg_layout]
    layout: nwg::FlexboxLayout,

    #[nwg_control]
    #[nwg_layout_item(layout: layout)]
    name_input: nwg::TextInput,

    #[nwg_control(text: "", check_state: CheckBoxState::Checked)]
    #[nwg_layout_item(layout: layout)]
    tasty_input: nwg::CheckBox,

    #[nwg_control(text: "Save")]
    #[nwg_layout_item(layout: layout)]
    save_btn: nwg::Button,
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let ui = ShortcutGUI::build_ui(Default::default()).expect("Failed to build UI");
    ui.dispatch_thread_events(); // requires ShortcutUi trait
}
