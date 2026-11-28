//! A ready-made design for tests, the command-line tool, and the app before it
//! has drawing tools.

use crate::edit::{Edit, EditError, Item};
use crate::history::History;
use crate::id::{EntityId, MaterialId, PanelId, SketchId};
use crate::model::{Document, Grain, Material, Panel, Plane, SheetSize, Sketch, ThicknessSide};

/// One rectangular panel lying on the ground with a corner at the origin,
/// built the way the app builds things: through edits. Sizes are millimeters.
pub fn one_panel(width: f64, depth: f64, thickness: f64, kerf: f64) -> Result<History, EditError> {
    let mut history = History::new(Document::new("Sample panel"));

    let material = Material {
        id: MaterialId(history.allocate_id()),
        name: "Birch ply".into(),
        thickness,
        kerf,
        sheet: SheetSize { width: 600.0, height: 400.0 },
        grain: Grain::None,
    };
    let material_id = material.id;
    history.apply("Add material", Edit::add(Item::Material(material)))?;

    let sketch_id = SketchId(history.allocate_id());
    let sides = std::array::from_fn(|_| EntityId(history.allocate_id()));
    let outline =
        Sketch::rectangle(sketch_id, "Panel outline", Plane::ground(), [0.0, 0.0], width, depth, sides);
    let panel = Panel {
        id: PanelId(history.allocate_id()),
        name: "Panel".into(),
        sketch: sketch_id,
        material: material_id,
        thickness_side: ThicknessSide::Positive,
    };
    history.apply(
        "Draw panel",
        Edit::Group(vec![Edit::add(Item::Sketch(outline)), Edit::add(Item::Panel(panel))]),
    )?;
    Ok(history)
}
