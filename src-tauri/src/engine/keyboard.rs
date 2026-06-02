#[cfg(target_os = "windows")]
use super::cycle::ClickCycleKind;
use super::cycle::{execute_click_cycle, ClickCyclePlan};
#[cfg(not(target_os = "windows"))]
use std::process::Command;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, MapVirtualKeyW, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC_EX, VK_CAPITAL,
    VK_SHIFT,
};

use super::worker::{sleep_interruptible, RunControl};

#[cfg(target_os = "windows")]
#[inline]
fn vk_to_scan(vk: u16) -> (u16, bool) {
    // MAPVK_VK_TO_VSC_EX returns the scan code in the low byte and, for
    // extended keys (arrows, Ins/Del/Home/End/PgUp/PgDn, numpad Enter, etc.),
    // a 0xE0/0xE1 prefix byte in the high byte. A non-zero high byte means
    // KEYEVENTF_EXTENDEDKEY must be set so apps that key off the extended
    // bit (or use raw input) see the correct key.
    let raw = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC_EX) };
    ((raw & 0xFF) as u16, (raw >> 8) != 0)
}

#[cfg(target_os = "windows")]
#[inline]
pub fn make_keyboard_input(vk: u16, flags: u32) -> INPUT {
    let (scan, extended) = vk_to_scan(vk);
    let ext_flag = if extended { KEYEVENTF_EXTENDEDKEY } else { 0 };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags | KEYEVENTF_SCANCODE | ext_flag,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(target_os = "windows")]
#[inline]
pub fn send_key_event(vk: u16, flags: u32) {
    let input = make_keyboard_input(vk, flags);
    unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
}

pub fn is_alphabetic_vk(vk: u16) -> bool {
    (b'A' as u16..=b'Z' as u16).contains(&vk)
}

fn caps_lock_enabled() -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        (GetKeyState(VK_CAPITAL as i32) & 1) != 0
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

fn should_hold_shift_for_case(vk: u16, uppercase: bool) -> bool {
    is_alphabetic_vk(vk) && (caps_lock_enabled() != uppercase)
}

#[cfg(target_os = "windows")]
fn push_key_press(inputs: &mut Vec<INPUT>, vk: u16, use_shift: bool) {
    if use_shift {
        inputs.push(make_keyboard_input(VK_SHIFT, 0));
    }

    inputs.push(make_keyboard_input(vk, 0));
    inputs.push(make_keyboard_input(vk, KEYEVENTF_KEYUP));

    if use_shift {
        inputs.push(make_keyboard_input(VK_SHIFT, KEYEVENTF_KEYUP));
    }
}

#[cfg(target_os = "windows")]
fn send_key_down(vk: u16, use_shift: bool) {
    if use_shift {
        send_key_event(VK_SHIFT, 0);
    }
    send_key_event(vk, 0);
}

#[cfg(target_os = "windows")]
fn send_key_up(vk: u16, use_shift: bool) {
    send_key_event(vk, KEYEVENTF_KEYUP);
    if use_shift {
        send_key_event(VK_SHIFT, KEYEVENTF_KEYUP);
    }
}

#[cfg(target_os = "windows")]
pub fn send_key_batch(vk: u16, n: usize, uppercase: bool) {
    let use_shift = should_hold_shift_for_case(vk, uppercase);
    let inputs_per_press = if use_shift { 4 } else { 2 };
    let mut inputs: Vec<INPUT> = Vec::with_capacity(n * inputs_per_press);
    for _ in 0..n {
        push_key_press(&mut inputs, vk, use_shift);
    }
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };
}

#[cfg(target_os = "windows")]
pub fn send_key_presses(
    vk: u16,
    count: usize,
    uppercase: bool,
    plan: ClickCyclePlan,
    control: &RunControl,
) {
    if count == 0 {
        return;
    }

    if plan.kind == ClickCycleKind::Single && count > 1 && plan.first_hold_ms == 0 {
        send_key_batch(vk, count, uppercase);
        return;
    }

    let use_shift = should_hold_shift_for_case(vk, uppercase);
    let is_active = || control.is_active();
    let mut sleep_for = |duration| sleep_interruptible(duration, control);

    for _ in 0..count {
        if !execute_click_cycle(
            plan,
            &mut || send_key_down(vk, use_shift),
            &mut || send_key_up(vk, use_shift),
            &mut sleep_for,
            &is_active,
        ) {
            return;
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn send_key_presses(
    vk: u16,
    count: usize,
    uppercase: bool,
    plan: ClickCyclePlan,
    control: &RunControl,
) {
    if count == 0 {
        return;
    }

    let is_active = || control.is_active();
    let mut sleep_for = |duration| sleep_interruptible(duration, control);
    let key = non_windows_key_name(vk, uppercase);

    for _ in 0..count {
        if !execute_click_cycle(
            plan,
            &mut || {
                if let Some(ref key) = key {
                    let _ = Command::new("xdotool").args(["keydown", key]).status();
                }
            },
            &mut || {
                if let Some(ref key) = key {
                    let _ = Command::new("xdotool").args(["keyup", key]).status();
                }
            },
            &mut sleep_for,
            &is_active,
        ) {
            return;
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn non_windows_key_name(vk: u16, uppercase: bool) -> Option<String> {
    if (b'A' as u16..=b'Z' as u16).contains(&vk) {
        let mut ch = char::from_u32(vk as u32)?;
        if !uppercase {
            ch = ch.to_ascii_lowercase();
        }
        return Some(ch.to_string());
    }
    if (b'0' as u16..=b'9' as u16).contains(&vk) {
        return Some(char::from_u32(vk as u32)?.to_string());
    }
    match vk as i32 {
        0x20 => Some(String::from("space")),
        0x09 => Some(String::from("Tab")),
        0x0D => Some(String::from("Return")),
        0x1B => Some(String::from("Escape")),
        0x08 => Some(String::from("BackSpace")),
        0x2E => Some(String::from("Delete")),
        0x2D => Some(String::from("Insert")),
        0x24 => Some(String::from("Home")),
        0x23 => Some(String::from("End")),
        0x21 => Some(String::from("Prior")),
        0x22 => Some(String::from("Next")),
        0x25 => Some(String::from("Left")),
        0x27 => Some(String::from("Right")),
        0x26 => Some(String::from("Up")),
        0x28 => Some(String::from("Down")),
        code @ 0x70..=0x7B => Some(format!("F{}", code - 0x6F)),
        _ => None,
    }
}
