use crate::engine::worker::now_epoch_ms;
use crate::engine::worker::start_clicker_inner;
use crate::engine::worker::stop_clicker_inner;
use crate::engine::worker::toggle_clicker_inner;
use crate::AppHandle;
use crate::ClickerState;
use std::sync::atomic::Ordering;
use tauri::Manager;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
    pub main_vk: i32,
    pub key_token: String,
}

/// Register the hotkey with tauri-plugin-global-shortcut (Wayland-compatible).
pub fn register_hotkey_inner(app: &AppHandle, hotkey: String) -> Result<String, String> {
    let binding = parse_hotkey_binding(&hotkey)?;
    let state = app.state::<ClickerState>();
    state
        .suppress_hotkey_until_ms
        .store(now_epoch_ms().saturating_add(250), Ordering::SeqCst);
    state
        .suppress_hotkey_until_release
        .store(true, Ordering::SeqCst);
    *state.registered_hotkey.lock().unwrap() = Some(binding.clone());

    // Register via tauri-plugin-global-shortcut (Wayland-compatible)
    let shortcut = format_hotkey_for_global_shortcut(&binding);
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    use tauri_plugin_global_shortcut::ShortcutState;

    let app_handle = app.app_handle();
    // Unregister all previous shortcuts first
    let _ = app_handle.global_shortcut().unregister_all();

    // Register the new shortcut with a handler
    let s = shortcut.clone();
    let _ = app_handle
        .global_shortcut()
        .on_shortcut(s.as_str(), move |h, _s, ev| {
            if ev.state == ShortcutState::Pressed {
                handle_hotkey_pressed(h);
            } else if ev.state == ShortcutState::Released {
                handle_hotkey_released(h);
            }
        });

    Ok(format_hotkey_binding(&binding))
}

/// Convert our HotkeyBinding to the accelerator string tauri-plugin-global-shortcut expects.
fn format_hotkey_for_global_shortcut(binding: &HotkeyBinding) -> String {
    let mut parts: Vec<String> = Vec::new();
    if binding.ctrl {
        parts.push(String::from("CommandOrControl"));
    }
    if binding.alt {
        parts.push(String::from("Alt"));
    }
    if binding.shift {
        parts.push(String::from("Shift"));
    }
    if binding.super_key {
        parts.push(String::from("Super"));
    }

    // Map vk to accelerator key name
    parts.push(vk_to_accelerator(binding.main_vk));
    parts.join("+")
}

fn vk_to_accelerator(vk: i32) -> String {
    match vk {
        0x41..=0x5A => ((vk as u8) as char).to_string(), // A-Z
        0x30..=0x39 => ((vk as u8) as char).to_string(), // 0-9
        0x20 => "Space".into(),
        0x0D => "Return".into(),
        0x09 => "Tab".into(),
        0x08 => "Backspace".into(),
        0x2E => "Delete".into(),
        0x1B => "Escape".into(),
        0x26 => "Up".into(),
        0x28 => "Down".into(),
        0x25 => "Left".into(),
        0x27 => "Right".into(),
        0x70 => "F1".into(),
        0x71 => "F2".into(),
        0x72 => "F3".into(),
        0x73 => "F4".into(),
        0x74 => "F5".into(),
        0x75 => "F6".into(),
        0x76 => "F7".into(),
        0x77 => "F8".into(),
        0x78 => "F9".into(),
        0x79 => "F10".into(),
        0x7A => "F11".into(),
        0x7B => "F12".into(),
        _ => format!("Key{}", vk),
    }
}

pub fn normalize_hotkey(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

pub fn parse_hotkey_binding(hotkey: &str) -> Result<HotkeyBinding, String> {
    let normalized = normalize_hotkey(hotkey);
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut super_key = false;
    let mut main_key: Option<(i32, String)> = None;

    for token in normalized.split('+').map(str::trim) {
        if token.is_empty() {
            return Err(format!("Invalid hotkey '{hotkey}': found empty key token"));
        }

        match normalize_modifier_token(token) {
            Some("ctrl") => ctrl = true,
            Some("alt") => alt = true,
            Some("shift") => shift = true,
            Some("super") => super_key = true,
            Some(_) => {}
            None => {
                if main_key
                    .replace(parse_hotkey_main_key(token, hotkey)?)
                    .is_some()
                {
                    return Err(format!(
                        "Invalid hotkey '{hotkey}': use modifiers first and only one main key"
                    ));
                }
            }
        }
    }

    let (main_vk, key_token) =
        main_key.ok_or_else(|| format!("Invalid hotkey '{hotkey}': missing main key"))?;

    Ok(HotkeyBinding {
        ctrl,
        alt,
        shift,
        super_key,
        main_vk,
        key_token,
    })
}

pub fn parse_hotkey_main_key(token: &str, original_hotkey: &str) -> Result<(i32, String), String> {
    let lower = token.trim().to_ascii_lowercase();

    if let Some(binding) = parse_named_key_token(&lower) {
        return Ok(binding);
    }

    if let Some(binding) = parse_mouse_button_token(&lower) {
        return Ok(binding);
    }

    if let Some(binding) = parse_numpad_token(&lower) {
        return Ok(binding);
    }

    if let Some(binding) = parse_function_key_token(&lower) {
        return Ok(binding);
    }

    if let Some(letter) = lower.strip_prefix("key") {
        if letter.len() == 1 {
            return parse_hotkey_main_key(letter, original_hotkey);
        }
    }

    if let Some(digit) = lower.strip_prefix("digit") {
        if digit.len() == 1 {
            return parse_hotkey_main_key(digit, original_hotkey);
        }
    }

    if lower.len() == 1 {
        let ch = lower.as_bytes()[0];
        if ch.is_ascii_lowercase() {
            return Ok((ch.to_ascii_uppercase() as i32, lower));
        }
        if ch.is_ascii_digit() {
            return Ok((ch as i32, lower));
        }
    }

    Err(format!(
        "Couldn't recognize '{token}' as a valid key in '{original_hotkey}'"
    ))
}

pub fn format_hotkey_binding(binding: &HotkeyBinding) -> String {
    let mut parts: Vec<String> = Vec::new();

    if binding.ctrl {
        parts.push(String::from("ctrl"));
    }
    if binding.alt {
        parts.push(String::from("alt"));
    }
    if binding.shift {
        parts.push(String::from("shift"));
    }
    if binding.super_key {
        parts.push(String::from("super"));
    }

    parts.push(binding.key_token.clone());
    parts.join("+")
}

/// Global shortcut handler setup (using tauri-plugin-global-shortcut).
/// The actual shortcut + handler registration happens in register_hotkey_inner.
/// This function does nothing now — kept for compatibility.
pub fn start_hotkey_listener(_app: AppHandle) {
    // Handled entirely via tauri-plugin-global-shortcut now.
}

pub fn handle_hotkey_pressed(app: &AppHandle) {
    let mode = {
        let state = app.state::<ClickerState>();
        let suppress = state.suppress_hotkey_until_ms.load(Ordering::SeqCst);
        if now_epoch_ms() < suppress {
            return;
        }
        let mode = state.settings.lock().unwrap().mode.clone();
        mode
    };

    if mode == "Toggle" {
        let _ = toggle_clicker_inner(app);
    } else if mode == "Hold" {
        let _ = start_clicker_inner(app);
    }
}

pub fn handle_hotkey_released(app: &AppHandle) {
    let mode = {
        let state = app.state::<ClickerState>();
        let mode = state.settings.lock().unwrap().mode.clone();
        mode
    };

    if mode == "Hold" {
        let _ = stop_clicker_inner(app, Some(String::from("Stopped from hold hotkey")));
    }
}

// ---------------------------------------------------------------------------
fn normalize_modifier_token(token: &str) -> Option<&'static str> {
    match token {
        "alt" | "option" => Some("alt"),
        "ctrl" | "control" => Some("ctrl"),
        "shift" => Some("shift"),
        "super" | "command" | "cmd" | "meta" | "win" => Some("super"),
        _ => None,
    }
}

fn binding(vk: i32, token: &str) -> (i32, String) {
    (vk, token.to_string())
}

fn parse_named_key_token(token: &str) -> Option<(i32, String)> {
    match token {
        "<" | ">" | "intlbackslash" | "oem102" | "nonusbackslash" => {
            Some(binding(0xE2, "IntlBackslash"))
        }
        "space" | "spacebar" => Some(binding(0x20, "space")),
        "tab" => Some(binding(0x09, "tab")),
        "enter" | "return" => Some(binding(0x0D, "enter")),
        "backspace" => Some(binding(0x08, "backspace")),
        "delete" | "del" => Some(binding(0x2E, "delete")),
        "insert" | "ins" => Some(binding(0x2D, "insert")),
        "home" => Some(binding(0x24, "home")),
        "end" => Some(binding(0x23, "end")),
        "pageup" | "pgup" => Some(binding(0x21, "pageup")),
        "pagedown" | "pgdn" => Some(binding(0x22, "pagedown")),
        "up" | "arrowup" => Some(binding(0x26, "up")),
        "down" | "arrowdown" => Some(binding(0x28, "down")),
        "left" | "arrowleft" => Some(binding(0x25, "left")),
        "right" | "arrowright" => Some(binding(0x27, "right")),
        "esc" | "escape" => Some(binding(0x1B, "escape")),
        "capslock" => Some(binding(0x14, "capslock")),
        "numlock" => Some(binding(0x90, "numlock")),
        "scrolllock" => Some(binding(0x91, "scrolllock")),
        "menu" | "apps" | "contextmenu" => Some(binding(0x5D, "menu")),
        "printscreen" | "prtsc" | "snapshot" => Some(binding(0x2C, "printscreen")),
        "pause" | "break" => Some(binding(0x13, "pause")),
        "/" | "slash" => Some(binding(0xBF, "/")),
        "\\" | "backslash" => Some(binding(0xDC, "\\")),
        ";" | "semicolon" => Some(binding(0xBA, ";")),
        "'" | "quote" | "apostrophe" => Some(binding(0xDE, "'")),
        "[" | "bracketleft" => Some(binding(0xDB, "[")),
        "]" | "bracketright" => Some(binding(0xDD, "]")),
        "-" | "minus" => Some(binding(0xBD, "-")),
        "=" | "equal" => Some(binding(0xBB, "=")),
        "`" | "backquote" | "grave" => Some(binding(0xC0, "`")),
        "," | "comma" => Some(binding(0xBC, ",")),
        "." | "period" | "dot" => Some(binding(0xBE, ".")),
        _ => None,
    }
}

fn parse_mouse_button_token(token: &str) -> Option<(i32, String)> {
    match token {
        "mouseleft" | "leftmouse" | "leftbutton" | "mouse1" | "lmb" => {
            Some(binding(1, "mouseleft"))
        }
        "mouseright" | "rightmouse" | "rightbutton" | "mouse2" | "rmb" => {
            Some(binding(2, "mouseright"))
        }
        "mousemiddle" | "middlemouse" | "middlebutton" | "mouse3" | "mmb" | "scrollbutton"
        | "middleclick" => Some(binding(3, "mousemiddle")),
        _ => None,
    }
}

fn parse_numpad_token(token: &str) -> Option<(i32, String)> {
    match token {
        "numpad0" | "num0" => Some(binding(0x60, "numpad0")),
        "numpad1" | "num1" => Some(binding(0x61, "numpad1")),
        "numpad2" | "num2" => Some(binding(0x62, "numpad2")),
        "numpad3" | "num3" => Some(binding(0x63, "numpad3")),
        "numpad4" | "num4" => Some(binding(0x64, "numpad4")),
        "numpad5" | "num5" => Some(binding(0x65, "numpad5")),
        "numpad6" | "num6" => Some(binding(0x66, "numpad6")),
        "numpad7" | "num7" => Some(binding(0x67, "numpad7")),
        "numpad8" | "num8" => Some(binding(0x68, "numpad8")),
        "numpad9" | "num9" => Some(binding(0x69, "numpad9")),
        "numpadadd" | "numadd" | "numpadplus" | "numplus" => Some(binding(0x6B, "numpadadd")),
        "numpadsubtract" | "numsubtract" | "numsub" | "numpadminus" | "numminus" => {
            Some(binding(0x6D, "numpadsubtract"))
        }
        "numpadmultiply" | "nummultiply" | "nummul" | "numpadmul" => {
            Some(binding(0x6A, "numpadmultiply"))
        }
        "numpaddivide" | "numdivide" | "numdiv" | "numpaddiv" => {
            Some(binding(0x6F, "numpaddivide"))
        }
        "numpaddecimal" | "numdecimal" | "numdot" | "numdel" | "numpadpoint" => {
            Some(binding(0x6E, "numpaddecimal"))
        }
        _ => None,
    }
}

fn parse_function_key_token(token: &str) -> Option<(i32, String)> {
    if !token.starts_with('f') || token.len() > 3 {
        return None;
    }

    let number = token[1..].parse::<i32>().ok()?;
    let vk = match number {
        1..=24 => 0x70 + (number - 1),
        _ => return None,
    };

    Some(binding(vk, token))
}
