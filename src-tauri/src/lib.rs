//! The Tauri shell. It owns the window and passes requests from the interface
//! to boxcad-core; it holds no geometry of its own.

/// The core library's version, so the interface can show it is talking to the core.
#[tauri::command]
fn core_version() -> &'static str {
    boxcad_core::version()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![core_version])
        .run(tauri::generate_context!())
        .expect("Box CAD could not start");
}
