/*!
    A very simple application that show how to use a flexbox layout.

    Requires the following features: `cargo run --example groupbox_d --features "flexbox"`
*/

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use nwd::NwgUi;
use nwg::{
    NativeUi,
    taffy::{
        LengthPercentage, LengthPercentageAuto,
        geometry::{Rect, Size},
        style::{Dimension, FlexDirection},
        style_helpers::{auto, length, percent},
    },
};

// Flexbox style
// this obviously will be font dependent.
// TODO add a builder flag to auto-size client area.
const PT_35: LengthPercentage = LengthPercentage::length(35.0);

const FIFTY_PC: Dimension = Dimension::percent(0.5);
const PT_10: LengthPercentage = LengthPercentage::length(10.0);
const PT_5: LengthPercentageAuto = LengthPercentageAuto::length(5.0);
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

#[derive(Default, NwgUi)]
pub struct GroupBoxApp {
    #[nwg_control(size: (500, 300), position: (300, 300), title: "GroupBox example")]
    #[nwg_events( OnWindowClose: [nwg::stop_thread_dispatch()] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, flex_direction: FlexDirection::Row, padding: PADDING)]
    layout: nwg::FlexboxLayout,

    #[nwg_layout(parent: groupbox, flex_direction: FlexDirection::Column, padding: GROUP_PADDING)]
    sub_layout: nwg::FlexboxLayout,

    #[nwg_control(text: "Btn 0")]
    #[nwg_layout_item(layout: layout, margin: MARGIN,
        max_size: Size { width: length(200.0), height: auto() },
        size: Size { width: FIFTY_PC, height: auto() }
    )]
    button0: nwg::Button,

    #[nwg_control(text: "Group!Box")]
    #[nwg_layout_item(layout: layout, margin: MARGIN,
        flex_grow: 2.0,
        size: Size { width: auto(), height: auto() }
    )]
    groupbox: nwg::GroupBox,

    #[nwg_control(parent: groupbox, text: "Btn 1")]
    #[nwg_layout_item(layout: sub_layout,
        margin: MARGIN,
        size: Size { width:  auto(), height: percent(0.25) }
    )]
    button1: nwg::Button,

    #[nwg_control(parent: groupbox, text: "Btn 2")]
    #[nwg_layout_item(layout: sub_layout,
        margin: MARGIN,
        size: Size { width:  auto(), height: percent(0.25) }
    )]
    button2: nwg::Button,

    #[nwg_control(parent: groupbox, text: "Btn 3")]
    #[nwg_layout_item(layout: sub_layout,
        margin: MARGIN,
        flex_grow: 2.0,
        size: Size { width: auto(),height: percent(0.25)}
    )]
    button3: nwg::Button,

    #[nwg_control(parent: groupbox, text: "Btn 4")]
    #[nwg_layout_item(layout: sub_layout,
        margin: MARGIN,
        size: Size { width:  auto(), height: percent(0.25) }
    )]
    button4: nwg::Button,

    #[nwg_control(parent: groupbox, text: "Btn 5")]
    #[nwg_layout_item(layout: sub_layout,
        margin: MARGIN,
        size: Size { width:  auto(), height: percent(0.25) }
    )]
    button5: nwg::Button,
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _ui = GroupBoxApp::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
