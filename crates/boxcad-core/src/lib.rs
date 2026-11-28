//! Box CAD's core: the document model and all cut geometry.
//!
//! This crate has no UI dependency. The app and the command-line tool both call
//! it, and the interface never computes geometry of its own.
//!
//! Conventions, everywhere in the core:
//! - Lengths are millimeters. Inches exist only as a display setting.
//! - Angles are degrees, counter-clockwise.
//! - Space is right-handed with Z up. The ground is the XY plane.

pub mod edit;
pub mod file;
pub mod history;
pub mod id;
pub mod model;
pub mod outline;
pub mod sample;
pub mod scene;
pub mod session;

pub use edit::{Edit, EditError, Item, ItemId};
pub use history::History;
pub use scene::{PanelView, Scene, scene};
pub use session::Snapshot;
pub use id::{EntityId, MaterialId, PanelId, ParameterId, SketchId};
pub use model::{
    DisplayUnits, Document, Entity, Grain, Material, ModelError, Panel, Parameter, ParameterUnit,
    Plane, Shape, SheetSize, Sketch, ThicknessSide,
};

/// The core library's version, which follows the app's.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_three_numbers() {
        let parts: Vec<&str> = version().split('.').collect();
        assert_eq!(parts.len(), 3, "expected major.minor.patch, got {}", version());
        assert!(parts.iter().all(|p| p.parse::<u32>().is_ok()));
    }
}
