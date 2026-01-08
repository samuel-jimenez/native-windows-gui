/*!
    Low level tabs utility
*/
use std::ptr;

use winapi::shared::{
    minwindef::{LPARAM, LRESULT, UINT, WPARAM},
    windef::HWND,
};

use super::window::build_sysclass;
use crate::NwgError;

pub const TAB_CLASS_ID: &'static str = "NWG_TAB";

/// Create the NWG tab classes
pub fn create_tab_classes() -> Result<(), NwgError> {
    use winapi::{
        shared::windef::HBRUSH,
        um::{libloaderapi::GetModuleHandleW, winuser::COLOR_BTNFACE},
    };

    let hmod = unsafe { GetModuleHandleW(ptr::null_mut()) };
    if hmod.is_null() {
        return Err(NwgError::initialization("GetModuleHandleW failed"));
    }

    unsafe {
        build_sysclass(
            hmod,
            TAB_CLASS_ID,
            Some(tab_proc),
            Some(COLOR_BTNFACE as HBRUSH),
            None,
        )?;
    }

    Ok(())
}

unsafe extern "system" fn tab_proc(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        use winapi::um::winuser::{DefWindowProcW, WM_CREATE};

        let handled = match msg {
            WM_CREATE => Some(0),
            _ => None,
        };

        if let Some(result) = handled {
            result
        } else {
            DefWindowProcW(hwnd, msg, w, l)
        }
    }
}
