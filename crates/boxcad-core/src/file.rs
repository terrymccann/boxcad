//! The project file: versioned JSON that describes the design and nothing else.
//!
//! ```json
//! { "format": "boxcad-project", "version": 1, "app_version": "0.1.0", "document": { } }
//! ```
//!
//! `version` counts changes to the file's layout, not app releases. A file with
//! an older version is brought up to date when opened; one with a newer version
//! is refused, because this app cannot know what it would be discarding.

use crate::model::{Document, ModelError};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

/// Marks a JSON file as a Box CAD project.
pub const FORMAT: &str = "boxcad-project";
/// The layout version this app writes.
pub const VERSION: u64 = 1;
/// The file extension for projects, without the dot.
pub const EXTENSION: &str = "boxcad";

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("this is not a Box CAD project file")]
    NotAProject,
    #[error(
        "this project was saved by a newer Box CAD (file version {found}; this app reads up to {supported}). Update Box CAD to open it"
    )]
    NewerVersion { found: u64, supported: u64 },
    #[error("the project file is damaged: {0}")]
    Damaged(#[from] serde_json::Error),
    #[error("the project file describes an invalid design: {0}")]
    Invalid(#[from] ModelError),
    #[error("could not read or write the file: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Serialize)]
struct Envelope<'a> {
    format: &'static str,
    version: u64,
    /// The app that wrote the file. For support questions only; never read back.
    app_version: &'static str,
    document: &'a Document,
}

/// The project file's text for a document.
pub fn to_json(document: &Document) -> Result<String, FileError> {
    document.validate()?;
    let envelope =
        Envelope { format: FORMAT, version: VERSION, app_version: crate::version(), document };
    let mut text = serde_json::to_string_pretty(&envelope)?;
    text.push('\n');
    Ok(text)
}

/// Read a document from a project file's text.
pub fn from_json(text: &str) -> Result<Document, FileError> {
    let mut file: Value = serde_json::from_str(text)?;
    if file.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(FileError::NotAProject);
    }
    let found = file.get("version").and_then(Value::as_u64).ok_or(FileError::NotAProject)?;
    if found > VERSION {
        return Err(FileError::NewerVersion { found, supported: VERSION });
    }
    let stored = file.get_mut("document").map(Value::take).ok_or(FileError::NotAProject)?;
    let document: Document = serde_json::from_value(migrate(stored, found))?;
    document.validate()?;
    Ok(document)
}

/// Bring a stored document from an older layout up to the current one. Each
/// future layout change adds one step here, so any old file still opens.
fn migrate(document: Value, _from: u64) -> Value {
    // Version 1 is the first layout; there is nothing older to convert yet.
    document
}

/// Write a document to a project file. The file is written beside its
/// destination and then moved into place, so a failure part-way cannot leave a
/// half-written project.
pub fn save(path: &Path, document: &Document) -> Result<(), FileError> {
    let text = to_json(document)?;
    let mut partial = path.as_os_str().to_owned();
    partial.push(".saving");
    let partial = Path::new(&partial);
    std::fs::write(partial, text)?;
    std::fs::rename(partial, path).inspect_err(|_| {
        let _ = std::fs::remove_file(partial);
    })?;
    Ok(())
}

/// Read a document from a project file.
pub fn load(path: &Path) -> Result<Document, FileError> {
    from_json(&std::fs::read_to_string(path)?)
}
