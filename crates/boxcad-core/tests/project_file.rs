//! The project file must round-trip exactly, and files already saved must keep
//! opening as the app changes.

use boxcad_core::file::{self, FileError};
use boxcad_core::{
    DisplayUnits, Document, Edit, Entity, EntityId, Grain, Item, Material, MaterialId, ModelError,
    Panel, PanelId, Parameter, ParameterId, ParameterUnit, Plane, Shape, SheetSize, Sketch,
    SketchId, ThicknessSide,
};
use serde_json::Value;

/// A file saved by version 1 of the layout. Never edit it: it stands for
/// projects on users' disks.
const V1_FIXTURE: &str = include_str!("fixtures/one-panel-v1.boxcad");

/// A document that uses every kind of thing the model has, with numbers that
/// are awkward to write as decimals.
fn busy_document() -> Document {
    let mut doc = Document::new("Round trip");
    let mut id = || doc.allocate_id();

    let birch = Material {
        id: MaterialId(id()),
        name: "Birch ply".into(),
        thickness: 2.8,
        kerf: 0.1 + 0.2,
        sheet: SheetSize { width: 600.0, height: 400.0 },
        grain: Grain::AlongWidth,
    };
    let acrylic = Material {
        id: MaterialId(id()),
        name: "Acrylic \"clear\" 3 mm".into(),
        thickness: 1.0 / 3.0 * 9.0,
        kerf: 1e-7,
        sheet: SheetSize { width: 304.8, height: 25.4 * 12.0 },
        grain: Grain::None,
    };

    let outline_id = SketchId(id());
    let sides = [(); 4].map(|_| EntityId(id()));
    let outline =
        Sketch::rectangle(outline_id, "Base outline", Plane::ground(), [-10.5, 0.1], 200.0, 120.0, sides);

    let upright = Plane { origin: [0.0, 0.0, 12.7], x_axis: [0.0, 1.0, 0.0], y_axis: [0.0, 0.0, 1.0] };
    let shapes = Sketch {
        id: SketchId(id()),
        name: "Every shape".into(),
        plane: upright,
        entities: vec![
            Entity { id: EntityId(id()), shape: Shape::Line { from: [0.0, 0.0], to: [std::f64::consts::PI, -1.0] } },
            Entity {
                id: EntityId(id()),
                shape: Shape::Arc { center: [5.0, 5.0], radius: 2.5, start_angle: 0.0, end_angle: 90.0 },
            },
            Entity { id: EntityId(id()), shape: Shape::Circle { center: [1.0 / 3.0, 2.0 / 3.0], radius: 0.75 } },
        ],
    };

    let base = Panel {
        id: PanelId(id()),
        name: "Base".into(),
        sketch: outline_id,
        material: birch.id,
        thickness_side: ThicknessSide::Positive,
    };
    let lid = Panel {
        id: PanelId(id()),
        name: "Lid".into(),
        sketch: outline_id,
        material: acrylic.id,
        thickness_side: ThicknessSide::Negative,
    };

    let width = Parameter { id: ParameterId(id()), name: "width".into(), value: 200.0, unit: ParameterUnit::Length };
    let tilt = Parameter { id: ParameterId(id()), name: "tilt".into(), value: 22.5, unit: ParameterUnit::Angle };
    let count = Parameter { id: ParameterId(id()), name: "dividers".into(), value: 3.0, unit: ParameterUnit::Number };

    doc.apply(Edit::Group(vec![
        Edit::add(Item::Material(birch)),
        Edit::add(Item::Material(acrylic)),
        Edit::add(Item::Sketch(outline)),
        Edit::add(Item::Sketch(shapes)),
        Edit::add(Item::Panel(base)),
        Edit::add(Item::Panel(lid)),
        Edit::add(Item::Parameter(width)),
        Edit::add(Item::Parameter(tilt)),
        Edit::add(Item::Parameter(count)),
        Edit::SetDisplayUnits(DisplayUnits::Inches),
    ]))
    .unwrap();
    doc
}

#[test]
fn a_document_survives_saving_and_opening_unchanged() {
    let doc = busy_document();
    let text = file::to_json(&doc).unwrap();
    let reopened = file::from_json(&text).unwrap();
    assert_eq!(reopened, doc);
    // Saving what was opened gives the same bytes, so a save with no changes is a no-op in a diff.
    assert_eq!(file::to_json(&reopened).unwrap(), text);
}

#[test]
fn saving_to_disk_and_loading_round_trips() {
    let doc = busy_document();
    let path = std::env::temp_dir().join(format!("boxcad-test-{}.boxcad", std::process::id()));
    file::save(&path, &doc).unwrap();
    let loaded = file::load(&path);
    let leftovers = path.with_extension("boxcad.saving").exists();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(loaded.unwrap(), doc);
    assert!(!leftovers, "the partial file was left behind");
}

#[test]
fn a_version_1_file_still_opens() {
    let doc = file::from_json(V1_FIXTURE).unwrap();
    assert_eq!(doc.name, "Sample panel");
    assert_eq!(doc.display_units, DisplayUnits::Millimeters);
    assert_eq!(doc.materials.len(), 1);
    assert_eq!(doc.materials[0].thickness, 3.0);
    assert_eq!(doc.materials[0].kerf, 0.15);
    assert_eq!(doc.sketches[0].entities.len(), 4);
    assert_eq!(
        doc.sketches[0].entities[1].shape,
        Shape::Line { from: [200.0, 0.0], to: [200.0, 120.0] }
    );
    assert_eq!(doc.panels[0].sketch, doc.sketches[0].id);
    assert_eq!(doc.panels[0].material, doc.materials[0].id);
}

#[test]
fn the_file_layout_has_not_drifted_from_version_1() {
    // If this fails, the layout changed: raise file::VERSION, add a migration
    // step, and add a new fixture. Do not edit the old one.
    let doc = file::from_json(V1_FIXTURE).unwrap();
    let mut written: Value = serde_json::from_str(&file::to_json(&doc).unwrap()).unwrap();
    let mut fixture: Value = serde_json::from_str(V1_FIXTURE).unwrap();
    for file in [&mut written, &mut fixture] {
        file.as_object_mut().unwrap().remove("app_version");
    }
    assert_eq!(written, fixture);
    assert_eq!(file::VERSION, 1);
}

#[test]
fn a_file_from_a_newer_app_is_refused_with_a_clear_message() {
    let newer = V1_FIXTURE.replacen("\"version\": 1", "\"version\": 2", 1);
    let problem = file::from_json(&newer).unwrap_err();
    assert!(matches!(problem, FileError::NewerVersion { found: 2, supported: 1 }));
    assert!(problem.to_string().contains("Update Box CAD"));
}

#[test]
fn other_files_are_refused() {
    assert!(matches!(file::from_json("{}"), Err(FileError::NotAProject)));
    assert!(matches!(file::from_json(r#"{"format":"something-else","version":1}"#), Err(FileError::NotAProject)));
    assert!(matches!(file::from_json(r#"{"format":"boxcad-project","version":1}"#), Err(FileError::NotAProject)));
    assert!(matches!(file::from_json("not json at all"), Err(FileError::Damaged(_))));
    assert!(matches!(file::from_json(&V1_FIXTURE[..V1_FIXTURE.len() / 2]), Err(FileError::Damaged(_))));
}

#[test]
fn a_file_describing_an_invalid_design_is_refused() {
    let broken = V1_FIXTURE.replacen("\"thickness\": 3.0", "\"thickness\": -3.0", 1);
    assert_ne!(broken, V1_FIXTURE);
    assert!(matches!(
        file::from_json(&broken),
        Err(FileError::Invalid(ModelError::BadNumber { .. }))
    ));
}

#[test]
fn an_invalid_document_is_never_written() {
    let mut doc = busy_document();
    doc.materials[0].thickness = f64::NAN;
    assert!(matches!(file::to_json(&doc), Err(FileError::Invalid(_))));
}
