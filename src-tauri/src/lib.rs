//! The Tauri shell. It owns the window and the open design, and passes requests
//! from the interface to boxcad-core; it works out nothing of its own. Every
//! command answers with a fresh snapshot for the interface to draw.

use boxcad_core::{DisplayUnits, History, MaterialId, Snapshot};
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

#[tauri::command]
fn snapshot(state: State<'_, Open>) -> Snapshot {
    open(&state).snapshot()
}

#[tauri::command]
fn undo(state: State<'_, Open>) -> Snapshot {
    let mut design = open(&state);
    design.undo();
    design.snapshot()
}

#[tauri::command]
fn redo(state: State<'_, Open>) -> Snapshot {
    let mut design = open(&state);
    design.redo();
    design.snapshot()
}

/// Set a material's thickness and kerf, in millimeters.
#[tauri::command]
fn set_material_numbers(
    state: State<'_, Open>,
    id: u64,
    thickness: f64,
    kerf: f64,
) -> Result<Snapshot, String> {
    let mut design = open(&state);
    design.set_material_numbers(MaterialId(id), thickness, kerf).map_err(|e| e.to_string())?;
    Ok(design.snapshot())
}

#[tauri::command]
fn set_display_units(state: State<'_, Open>, units: DisplayUnits) -> Result<Snapshot, String> {
    let mut design = open(&state);
    design.set_display_units(units).map_err(|e| e.to_string())?;
    Ok(design.snapshot())
}

pub fn run() {
    // Until the app can draw and open designs, it starts with one sample panel.
    let sample = boxcad_core::sample::one_panel(200.0, 120.0, 3.0, 0.15)
        .expect("the built-in sample is a valid design");
    // Start its history clean, so building the sample is not something to undo.
    let design = History::new(sample.document().clone());

    tauri::Builder::default()
        .manage(Mutex::new(design))
        .invoke_handler(tauri::generate_handler![
            core_version,
            snapshot,
            undo,
            redo,
            set_material_numbers,
            set_display_units
        ])
        .run(tauri::generate_context!())
        .expect("Box CAD could not start");
}
