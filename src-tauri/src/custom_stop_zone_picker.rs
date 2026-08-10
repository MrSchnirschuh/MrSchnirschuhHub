use tauri::{AppHandle, Manager};

pub fn start_custom_stop_zone_pick_inner(app: AppHandle) -> Result<(), String> {
    log::info!("[StopZonePicker] Interactive picking not available on Linux; use settings panel.");
    app.state::<crate::ClickerState>()
        .custom_stop_zone_pick_active
        .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

pub fn cancel_custom_stop_zone_pick_inner(app: &AppHandle) {
    app.state::<crate::ClickerState>()
        .custom_stop_zone_pick_active
        .store(false, std::sync::atomic::Ordering::SeqCst);
}
