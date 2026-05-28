use tauri::AppHandle;

pub fn start_sequence_point_pick_inner(_app: AppHandle) -> Result<(), String> {
    Err(String::from(
        "Sequence point picking is currently only supported on Windows",
    ))
}

pub fn cancel_sequence_point_pick_inner(_app: &AppHandle) {}
