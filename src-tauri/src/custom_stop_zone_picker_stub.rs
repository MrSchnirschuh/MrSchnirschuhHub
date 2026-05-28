use tauri::AppHandle;

pub fn start_custom_stop_zone_pick_inner(_app: AppHandle) -> Result<(), String> {
    Err(String::from(
        "Custom stop-zone picking is currently only supported on Windows",
    ))
}

pub fn cancel_custom_stop_zone_pick_inner(_app: &AppHandle) {}
