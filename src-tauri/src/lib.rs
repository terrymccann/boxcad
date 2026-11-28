//! The Tauri shell. It owns the window and the open design, and passes requests
//! from the interface to boxcad-core; it works out no geometry of its own.

use boxcad_core::{History, Scene};
use std::sync::{Mutex, MutexGuard};
use tauri::State;

/// The design that is open, with its undo history.
type Open = Mutex<History>;

fn open<'a>(state: &'a State<'_, Open>) -> MutexGuard<'a, History> {
    // A panic elsewhere must not lock the user out of their design.
    state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The core library's version.
#[tauri::command]
fn core_version() -> &'static str {
    boxcad_core::version()
}

/// What the 3D view should draw for the open design.
#[tauri::command]
fn scene(state: State<'_, Open>) -> Scene {
    boxcad_core::scene(open(&state).document())
}

pub fn run() {
    // Until the app can draw and open designs, it starts with one sample panel.
    let sample = boxcad_core::sample::one_panel(200.0, 120.0, 3.0, 0.15)
        .expect("the built-in sample is a valid design");

    tauri::Builder::default()
        .manage(Mutex::new(sample))
        .invoke_handler(tauri::generate_handler![core_version, scene])
        .run(tauri::generate_context!())
        .expect("Box CAD could not start");
}
