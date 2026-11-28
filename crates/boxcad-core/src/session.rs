//! What the app asks of an open design: a snapshot of everything the interface
//! shows, and the changes its controls make. Each change is one labelled edit,
//! so it can be undone. The shell passes these straight through.

use crate::edit::{Edit, EditError, Item};
use crate::history::History;
use crate::id::MaterialId;
use crate::model::{DisplayUnits, Material};
use crate::scene::{Scene, scene};
use serde::Serialize;

/// Everything the interface needs to draw itself for the open design.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Snapshot {
    pub name: String,
    /// Whether the design differs from what was last saved.
    pub modified: bool,
    /// What Undo and Redo would act on, if anything.
    pub undo: Option<String>,
    pub redo: Option<String>,
    pub display_units: DisplayUnits,
    pub materials: Vec<Material>,
    pub scene: Scene,
}

impl History {
    pub fn snapshot(&self) -> Snapshot {
        let document = self.document();
        Snapshot {
            name: document.name.clone(),
            modified: self.is_modified(),
            undo: self.next_undo().map(str::to_owned),
            redo: self.next_redo().map(str::to_owned),
            display_units: document.display_units,
            materials: document.materials.clone(),
            scene: scene(document),
        }
    }

    /// Set a material's measured thickness and kerf, in millimeters.
    pub fn set_material_numbers(&mut self, id: MaterialId, thickness: f64, kerf: f64) -> Result<(), EditError> {
        let Some(current) = self.document().material(id) else {
            return Err(EditError::NotFound(crate::edit::ItemId::Material(id)));
        };
        if current.thickness == thickness && current.kerf == kerf {
            return Ok(());
        }
        let label = match (current.thickness != thickness, current.kerf != kerf) {
            (true, false) => "Change thickness",
            (false, true) => "Change kerf",
            _ => "Change material",
        };
        let changed = Material { thickness, kerf, ..current.clone() };
        self.apply(label, Edit::Replace(Item::Material(changed)))
    }

    /// Choose how lengths are shown. The design itself does not change size.
    pub fn set_display_units(&mut self, units: DisplayUnits) -> Result<(), EditError> {
        if self.document().display_units == units {
            return Ok(());
        }
        self.apply("Change units", Edit::SetDisplayUnits(units))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModelError;
    use crate::sample;

    fn open() -> History {
        History::new(sample::one_panel(200.0, 120.0, 3.0, 0.15).unwrap().document().clone())
    }

    #[test]
    fn a_fresh_design_has_nothing_to_undo_and_is_not_modified() {
        let s = open().snapshot();
        assert_eq!(s.name, "Sample panel");
        assert!(!s.modified);
        assert_eq!((s.undo, s.redo), (None, None));
        assert_eq!(s.materials.len(), 1);
        assert_eq!(s.scene.panels[0].thickness, 3.0);
    }

    #[test]
    fn changing_thickness_redraws_the_panel_and_can_be_undone() {
        let mut h = open();
        let id = h.document().materials[0].id;
        for thickness in [0.5, 2.8, 6.0, 18.0] {
            h.set_material_numbers(id, thickness, 0.15).unwrap();
            let s = h.snapshot();
            assert_eq!(s.scene.panels[0].thickness, thickness);
            assert_eq!(s.undo.as_deref(), Some("Change thickness"));
            assert!(s.modified);
            h.undo();
            assert_eq!(h.snapshot().scene.panels[0].thickness, 3.0);
            assert_eq!(h.snapshot().redo.as_deref(), Some("Change thickness"));
            assert!(!h.snapshot().modified);
        }
    }

    #[test]
    fn labels_say_what_changed() {
        let mut h = open();
        let id = h.document().materials[0].id;
        h.set_material_numbers(id, 3.0, 0.2).unwrap();
        assert_eq!(h.next_undo(), Some("Change kerf"));
        h.set_material_numbers(id, 4.0, 0.1).unwrap();
        assert_eq!(h.next_undo(), Some("Change material"));
    }

    #[test]
    fn setting_the_same_values_is_not_an_edit() {
        let mut h = open();
        let id = h.document().materials[0].id;
        h.set_material_numbers(id, 3.0, 0.15).unwrap();
        h.set_display_units(DisplayUnits::Millimeters).unwrap();
        assert!(!h.can_undo());
        assert!(!h.is_modified());
    }

    #[test]
    fn bad_numbers_are_refused_with_a_reason() {
        let mut h = open();
        let id = h.document().materials[0].id;
        for (thickness, kerf) in [(0.0, 0.15), (-3.0, 0.15), (3.0, -0.1), (f64::NAN, 0.15), (3.0, f64::INFINITY)] {
            let problem = h.set_material_numbers(id, thickness, kerf).unwrap_err();
            assert!(matches!(problem, EditError::Invalid(ModelError::BadNumber { .. })), "{thickness}, {kerf}");
        }
        let message = h.set_material_numbers(id, 0.0, 0.15).unwrap_err().to_string();
        assert_eq!(message, "the thickness of material \"Birch ply\" must be greater than zero, but is 0");
        assert!(matches!(h.set_material_numbers(MaterialId(99), 3.0, 0.1), Err(EditError::NotFound(_))));
        assert!(!h.can_undo());
    }

    #[test]
    fn switching_units_changes_only_the_display() {
        let mut h = open();
        let before = h.snapshot().scene;
        h.set_display_units(DisplayUnits::Inches).unwrap();
        let s = h.snapshot();
        assert_eq!(s.display_units, DisplayUnits::Inches);
        assert_eq!(s.scene, before);
        assert_eq!(s.undo.as_deref(), Some("Change units"));
    }
}
