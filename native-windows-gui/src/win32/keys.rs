pub fn get_key_state(virt_keycode: u32) -> u8 {
    use winapi::um::winuser::GetKeyState;

    if unsafe { GetKeyState(virt_keycode as i32) } < 0 {
        1u8
    } else {
        0u8
    }
}
