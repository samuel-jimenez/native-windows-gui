/*!
    An application that saves messages into buttons.
    Demonstrate the dynamic functions of NWG.

    `cargo run --example shortcuts_d`
*/

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
#[nwg_shortcuts( Ctrl+G: [ShortcutGUI::do_global_shortcut], Ctrl+S: [ShortcutGUI::do_global_shortcut], Alt+S: [ShortcutGUI::do_global_shortcut, ShortcutGUI::do_shortcut], Ctrl+P: [ShortcutGUI::do_global_shortcut], Ctrl+M: [ShortcutGUI::do_global_shortcut] )]
pub struct ShortcutGUI {
    #[nwg_control(size:(400, 300), position:(800, 300), title: "Shortcuts Demo")]
    #[nwg_events( OnWindowClose: [ShortcutGUI::exit], OnKeyPress: [ShortcutGUI::func_0,ShortcutGUI::do_shortcut,]  )]
    window: nwg::Window,

    #[nwg_layout(parent: window, max_row: Some(6), spacing: 3)]
    layout: nwg::GridLayout,

    // the `&` symbol in the text field can be used to automatically tie Alt events to button presses.
    #[nwg_control(text: "&Save", focus: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 0)]
    #[nwg_events( OnButtonClick: [ShortcutGUI::click] )]
    #[nwg_shortcuts( Ctrl+Shift+Plus: [ShortcutGUI::do_bonus_shortcut(SELF,CTRL)], Ctrl+P: [ShortcutGUI::do_shortcut(SELF)], Key0: [ShortcutGUI::do_shortcut(SELF)] )]
    add_message_btn: nwg::Button,

    #[nwg_control(text:"Title")]
    #[nwg_layout_item(layout: layout, col: 1, row: 0, col_span: 2)]
    #[nwg_shortcuts(  NumpadMinus: [ShortcutGUI::do_shortcut], Ctrl+Shift+S: [ShortcutGUI::do_bonus_shortcut(SELF,CTRL)], Ctrl+Alt+S: [ShortcutGUI::do_text_shortcut(SELF,CTRL)], Ctrl+P: [ShortcutGUI::do_shortcut] , Alt+A: [ShortcutGUI::do_shortcut]  )]
    message_title: nwg::TextInput,

    #[nwg_control(text:"Hello World!")]
    #[nwg_layout_item(layout: layout, col: 3, row: 0, col_span: 3)]
    #[nwg_shortcuts( NumpadPlus: [ShortcutGUI::do_shortcut], Ctrl+U: [ShortcutGUI::do_shortcut],Ctrl+M: [ShortcutGUI::do_shortcut], Ctrl+Plus+Shift: [ShortcutGUI::do_shortcut], Ctrl+P: [ShortcutGUI::do_shortcut], Ctrl+Shift+S: [ShortcutGUI::do_bonus_shortcut(SELF,CTRL)], Ctrl+Alt+P: [ShortcutGUI::do_text_shortcut(SELF,CTRL)])]
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

    let _ui = ShortcutGUI::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
