//! What the 3D view draws: every panel as a flat outline, where it sits in
//! space, and how thick it is. The interface extrudes these for display and
//! computes nothing else.

use crate::id::PanelId;
use crate::model::{Document, ThicknessSide};
use crate::outline::outline;
use serde::Serialize;

/// Everything in a design that can be drawn.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Scene {
    pub panels: Vec<PanelView>,
}

/// One panel, ready to draw.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PanelView {
    pub id: PanelId,
    pub name: String,
    pub material: String,
    /// Millimeters, measured along `normal` from `origin`.
    pub thickness: f64,
    /// A point on the panel's lower face: the face the thickness grows from.
    pub origin: [f64; 3],
    /// Unit vectors placing the outline's x and y in space.
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
    /// The direction the thickness grows in: `x_axis` × `y_axis`.
    pub normal: [f64; 3],
    /// The outside edge, counter-clockwise, in the panel's own x and y.
    pub outline: Vec<[f64; 2]>,
    /// Holes, each clockwise.
    pub holes: Vec<Vec<[f64; 2]>>,
    /// Why the panel cannot be drawn, if it cannot. The outline is then empty.
    pub problem: Option<String>,
}

/// Work out what to draw for a document.
pub fn scene(document: &Document) -> Scene {
    let panels = document
        .panels
        .iter()
        .filter_map(|panel| {
            // An edit is refused if it would leave a panel without these.
            let sketch = document.sketch(panel.sketch)?;
            let material = document.material(panel.material)?;
            let plane = sketch.plane;
            let normal = plane.normal();
            let origin = match panel.thickness_side {
                ThicknessSide::Positive => plane.origin,
                ThicknessSide::Negative => [0, 1, 2].map(|i| plane.origin[i] - normal[i] * material.thickness),
            };
            let (outer, holes, problem) = match outline(sketch) {
                Ok(o) => (o.outer, o.holes, None),
                Err(e) => (Vec::new(), Vec::new(), Some(e.to_string())),
            };
            Some(PanelView {
                id: panel.id,
                name: panel.name.clone(),
                material: material.name.clone(),
                thickness: material.thickness,
                origin,
                x_axis: plane.x_axis,
                y_axis: plane.y_axis,
                normal,
                outline: outer,
                holes,
                problem,
            })
        })
        .collect();
    Scene { panels }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::{Edit, Item};
    use crate::sample;

    #[test]
    fn the_sample_panel_sits_on_the_ground_at_its_real_thickness() {
        for thickness in [0.8, 3.0, 6.35, 12.0] {
            let history = sample::one_panel(200.0, 120.0, thickness, 0.15).unwrap();
            let scene = scene(history.document());
            assert_eq!(scene.panels.len(), 1);
            let p = &scene.panels[0];
            assert_eq!(p.thickness, thickness);
            assert_eq!(p.origin, [0.0, 0.0, 0.0]);
            assert_eq!(p.normal, [0.0, 0.0, 1.0]);
            assert_eq!(p.outline, [[0.0, 0.0], [200.0, 0.0], [200.0, 120.0], [0.0, 120.0]]);
            assert!(p.holes.is_empty() && p.problem.is_none());
        }
    }

    #[test]
    fn a_panel_growing_the_other_way_starts_one_thickness_lower() {
        let mut history = sample::one_panel(200.0, 120.0, 3.0, 0.15).unwrap();
        let mut panel = history.document().panels[0].clone();
        panel.thickness_side = ThicknessSide::Negative;
        history.apply("Flip", Edit::Replace(Item::Panel(panel))).unwrap();
        let p = &scene(history.document()).panels[0];
        assert_eq!(p.origin, [0.0, 0.0, -3.0]);
        assert_eq!(p.normal, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn a_panel_with_an_open_outline_is_listed_with_its_problem() {
        let mut history = sample::one_panel(200.0, 120.0, 3.0, 0.15).unwrap();
        let mut sketch = history.document().sketches[0].clone();
        sketch.entities.pop();
        history.apply("Break outline", Edit::Replace(Item::Sketch(sketch))).unwrap();
        let p = &scene(history.document()).panels[0];
        assert!(p.outline.is_empty());
        assert!(p.problem.as_deref().is_some_and(|m| m.contains("open outline")));
    }
}
