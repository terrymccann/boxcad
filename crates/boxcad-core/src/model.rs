//! The document: plain data describing a design. It holds what the user drew
//! and chose, never the cut geometry worked out from it.

use crate::id::{EntityId, MaterialId, PanelId, ParameterId, SketchId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// How lengths are shown and typed. The stored numbers are always millimeters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DisplayUnits {
    #[default]
    #[serde(rename = "mm")]
    Millimeters,
    #[serde(rename = "in")]
    Inches,
}

/// A sheet's cuttable size, in millimeters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SheetSize {
    pub width: f64,
    pub height: f64,
}

/// Which way the grain runs on a sheet, if it matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grain {
    #[default]
    None,
    AlongWidth,
    AlongHeight,
}

/// A sheet stock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub id: MaterialId,
    pub name: String,
    /// Measured thickness, which every panel of this material takes.
    pub thickness: f64,
    /// Width of material the laser removes along a cut.
    pub kerf: f64,
    pub sheet: SheetSize,
    #[serde(default)]
    pub grain: Grain,
}

/// A flat plane in space that a sketch is drawn on. `x_axis` and `y_axis` are
/// unit vectors at right angles; the plane's normal is `x_axis` × `y_axis`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    pub origin: [f64; 3],
    pub x_axis: [f64; 3],
    pub y_axis: [f64; 3],
}

impl Plane {
    /// The ground: the XY plane through the origin, with Z up as its normal.
    pub fn ground() -> Self {
        Plane { origin: [0.0; 3], x_axis: [1.0, 0.0, 0.0], y_axis: [0.0, 1.0, 0.0] }
    }

    /// The direction the plane faces.
    pub fn normal(&self) -> [f64; 3] {
        let (a, b) = (self.x_axis, self.y_axis);
        [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
    }
}

/// The geometry of one sketch entity, in the sketch plane's own x and y.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Shape {
    Line { from: [f64; 2], to: [f64; 2] },
    /// Runs counter-clockwise from `start_angle` to `end_angle`, in degrees.
    Arc { center: [f64; 2], radius: f64, start_angle: f64, end_angle: f64 },
    Circle { center: [f64; 2], radius: f64 },
}

/// One line, arc or circle in a sketch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    #[serde(flatten)]
    pub shape: Shape,
}

/// 2D geometry drawn on a plane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sketch {
    pub id: SketchId,
    pub name: String,
    pub plane: Plane,
    pub entities: Vec<Entity>,
}

impl Sketch {
    /// A rectangle with one corner at `corner`, drawn as four lines
    /// counter-clockwise. `entity_ids` names the bottom, right, top and left
    /// sides in that order.
    pub fn rectangle(
        id: SketchId,
        name: impl Into<String>,
        plane: Plane,
        corner: [f64; 2],
        width: f64,
        depth: f64,
        entity_ids: [EntityId; 4],
    ) -> Self {
        let [x, y] = corner;
        let points = [[x, y], [x + width, y], [x + width, y + depth], [x, y + depth]];
        let entities = (0..4)
            .map(|i| Entity {
                id: entity_ids[i],
                shape: Shape::Line { from: points[i], to: points[(i + 1) % 4] },
            })
            .collect();
        Sketch { id, name: name.into(), plane, entities }
    }
}

/// Which side of its sketch plane a panel's thickness grows to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThicknessSide {
    /// Along the plane's normal. On the ground this is upward.
    #[default]
    Positive,
    /// Against the plane's normal.
    Negative,
}

/// A flat part: a sketch outline, as thick as its material.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Panel {
    pub id: PanelId,
    pub name: String,
    pub sketch: SketchId,
    pub material: MaterialId,
    #[serde(default)]
    pub thickness_side: ThicknessSide,
}

/// What kind of number a parameter is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterUnit {
    /// Millimeters.
    #[default]
    Length,
    /// Degrees.
    Angle,
    /// A plain count or ratio.
    Number,
}

/// A named number the design can refer to, such as `width`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub id: ParameterId,
    pub name: String,
    pub value: f64,
    #[serde(default)]
    pub unit: ParameterUnit,
}

/// A whole design. Change it through [`crate::Edit`] so that every change can
/// be undone; the fields are public for reading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub name: String,
    #[serde(default)]
    pub display_units: DisplayUnits,
    #[serde(default)]
    pub materials: Vec<Material>,
    #[serde(default)]
    pub sketches: Vec<Sketch>,
    #[serde(default)]
    pub panels: Vec<Panel>,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    /// The next unused id. It only ever goes up, so ids are never reused.
    pub next_id: u64,
}

/// A way a document can be wrong.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ModelError {
    #[error("id {0} is used more than once")]
    DuplicateId(u64),
    #[error("id {0} was never issued by this document")]
    UnissuedId(u64),
    #[error("{what} needs a name")]
    MissingName { what: &'static str },
    #[error(
        "\"{0}\" is not a usable parameter name: use letters, digits and underscores, not starting with a digit"
    )]
    BadParameterName(String),
    #[error("two parameters are both named \"{0}\"")]
    DuplicateParameterName(String),
    #[error("{what} must be {rule}, but is {value}")]
    BadNumber { what: String, rule: &'static str, value: f64 },
    #[error("sketch \"{0}\" has a plane whose axes are not unit length and at right angles")]
    BadPlane(String),
    #[error("panel \"{panel}\" refers to sketch {sketch}, which does not exist")]
    MissingSketch { panel: String, sketch: SketchId },
    #[error("panel \"{panel}\" refers to material {material}, which does not exist")]
    MissingMaterial { panel: String, material: MaterialId },
}

impl Default for Document {
    fn default() -> Self {
        Document::new("Untitled")
    }
}

impl Document {
    /// An empty design.
    pub fn new(name: impl Into<String>) -> Self {
        Document {
            name: name.into(),
            display_units: DisplayUnits::default(),
            materials: Vec::new(),
            sketches: Vec::new(),
            panels: Vec::new(),
            parameters: Vec::new(),
            next_id: 1,
        }
    }

    /// Issue an id for something about to be added. Issuing is not an edit and
    /// is not undone: an id that was handed out is never handed out again.
    pub fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn material(&self, id: MaterialId) -> Option<&Material> {
        self.materials.iter().find(|m| m.id == id)
    }

    pub fn sketch(&self, id: SketchId) -> Option<&Sketch> {
        self.sketches.iter().find(|s| s.id == id)
    }

    pub fn panel(&self, id: PanelId) -> Option<&Panel> {
        self.panels.iter().find(|p| p.id == id)
    }

    pub fn parameter(&self, id: ParameterId) -> Option<&Parameter> {
        self.parameters.iter().find(|p| p.id == id)
    }

    /// Check the whole document. Every edit is checked this way before it is
    /// kept, and so is every file that is opened.
    pub fn validate(&self) -> Result<(), ModelError> {
        let mut seen = HashSet::new();
        let mut claim = |id: u64| {
            if id == 0 || id >= self.next_id {
                Err(ModelError::UnissuedId(id))
            } else if !seen.insert(id) {
                Err(ModelError::DuplicateId(id))
            } else {
                Ok(())
            }
        };

        for m in &self.materials {
            claim(m.id.0)?;
            named(&m.name, "a material")?;
            let what = |field: &str| format!("the {field} of material \"{}\"", m.name);
            positive(m.thickness, || what("thickness"))?;
            not_negative(m.kerf, || what("kerf"))?;
            positive(m.sheet.width, || what("sheet width"))?;
            positive(m.sheet.height, || what("sheet height"))?;
        }

        for s in &self.sketches {
            claim(s.id.0)?;
            named(&s.name, "a sketch")?;
            if !plane_is_sound(&s.plane) {
                return Err(ModelError::BadPlane(s.name.clone()));
            }
            for e in &s.entities {
                claim(e.id.0)?;
                check_shape(&e.shape, &s.name)?;
            }
        }

        for p in &self.panels {
            claim(p.id.0)?;
            named(&p.name, "a panel")?;
            if self.sketch(p.sketch).is_none() {
                return Err(ModelError::MissingSketch { panel: p.name.clone(), sketch: p.sketch });
            }
            if self.material(p.material).is_none() {
                return Err(ModelError::MissingMaterial {
                    panel: p.name.clone(),
                    material: p.material,
                });
            }
        }

        let mut names = HashSet::new();
        for p in &self.parameters {
            claim(p.id.0)?;
            if !is_parameter_name(&p.name) {
                return Err(ModelError::BadParameterName(p.name.clone()));
            }
            if !names.insert(p.name.as_str()) {
                return Err(ModelError::DuplicateParameterName(p.name.clone()));
            }
            finite(p.value, || format!("parameter \"{}\"", p.name))?;
        }
        Ok(())
    }
}

fn named(name: &str, what: &'static str) -> Result<(), ModelError> {
    if name.trim().is_empty() { Err(ModelError::MissingName { what }) } else { Ok(()) }
}

/// Parameter names will appear in expressions, so they are kept to identifiers.
fn is_parameter_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn finite(value: f64, what: impl Fn() -> String) -> Result<(), ModelError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(ModelError::BadNumber { what: what(), rule: "a real number", value })
    }
}

fn positive(value: f64, what: impl Fn() -> String) -> Result<(), ModelError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(ModelError::BadNumber { what: what(), rule: "greater than zero", value })
    }
}

fn not_negative(value: f64, what: impl Fn() -> String) -> Result<(), ModelError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(ModelError::BadNumber { what: what(), rule: "zero or more", value })
    }
}

fn plane_is_sound(plane: &Plane) -> bool {
    const TOLERANCE: f64 = 1e-9;
    let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let all = plane.origin.iter().chain(&plane.x_axis).chain(&plane.y_axis);
    all.copied().all(f64::is_finite)
        && (dot(plane.x_axis, plane.x_axis) - 1.0).abs() < TOLERANCE
        && (dot(plane.y_axis, plane.y_axis) - 1.0).abs() < TOLERANCE
        && dot(plane.x_axis, plane.y_axis).abs() < TOLERANCE
}

fn check_shape(shape: &Shape, sketch: &str) -> Result<(), ModelError> {
    let what = |part: &str| format!("{part} in sketch \"{sketch}\"");
    let point = |p: &[f64; 2], part: &str| {
        finite(p[0], || what(part))?;
        finite(p[1], || what(part))
    };
    match shape {
        Shape::Line { from, to } => {
            point(from, "a line's start")?;
            point(to, "a line's end")?;
            let length = (to[0] - from[0]).hypot(to[1] - from[1]);
            positive(length, || what("a line's length"))
        }
        Shape::Arc { center, radius, start_angle, end_angle } => {
            point(center, "an arc's center")?;
            positive(*radius, || what("an arc's radius"))?;
            finite(*start_angle, || what("an arc's start angle"))?;
            finite(*end_angle, || what("an arc's end angle"))
        }
        Shape::Circle { center, radius } => {
            point(center, "a circle's center")?;
            positive(*radius, || what("a circle's radius"))
        }
    }
}
