//! Edits: the only way a document changes. Applying one returns the edit that
//! undoes it, so undo is a list of edits and nothing else.

use crate::id::{MaterialId, PanelId, ParameterId, SketchId};
use crate::model::{DisplayUnits, Document, Material, ModelError, Panel, Parameter, Sketch};

/// Something a document holds a list of.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Material(Material),
    Sketch(Sketch),
    Panel(Panel),
    Parameter(Parameter),
}

/// Names one item in a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemId {
    Material(MaterialId),
    Sketch(SketchId),
    Panel(PanelId),
    Parameter(ParameterId),
}

impl Item {
    pub fn id(&self) -> ItemId {
        match self {
            Item::Material(m) => ItemId::Material(m.id),
            Item::Sketch(s) => ItemId::Sketch(s.id),
            Item::Panel(p) => ItemId::Panel(p.id),
            Item::Parameter(p) => ItemId::Parameter(p.id),
        }
    }
}

impl std::fmt::Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemId::Material(id) => write!(f, "material {id}"),
            ItemId::Sketch(id) => write!(f, "sketch {id}"),
            ItemId::Panel(id) => write!(f, "panel {id}"),
            ItemId::Parameter(id) => write!(f, "parameter {id}"),
        }
    }
}

/// One reversible change to a document.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    /// Add an item. `at` is its position in its list; `None` adds it at the end.
    Insert { item: Item, at: Option<usize> },
    /// Take an item out.
    Remove(ItemId),
    /// Swap an item for a changed copy with the same id.
    Replace(Item),
    /// Rename the design.
    SetName(String),
    SetDisplayUnits(DisplayUnits),
    /// Several edits that happen, and are undone, as one.
    Group(Vec<Edit>),
}

impl Edit {
    /// Add an item at the end of its list.
    pub fn add(item: Item) -> Self {
        Edit::Insert { item, at: None }
    }
}

/// Why an edit was refused. A refused edit leaves the document untouched.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EditError {
    #[error("there is no {0}")]
    NotFound(ItemId),
    #[error("cannot put {id} at position {at} in a list of {len}")]
    BadPosition { id: ItemId, at: usize, len: usize },
    #[error(transparent)]
    Invalid(#[from] ModelError),
}

impl Document {
    /// Apply an edit and return the edit that undoes it. If the edit cannot be
    /// made, or would leave the document invalid, nothing changes.
    pub fn apply(&mut self, edit: Edit) -> Result<Edit, EditError> {
        let inverse = self.apply_unchecked(edit)?;
        if let Err(problem) = self.validate() {
            self.apply_unchecked(inverse).expect("an edit just made can be reversed");
            return Err(problem.into());
        }
        Ok(inverse)
    }

    /// Make the change without checking the result is a valid document.
    fn apply_unchecked(&mut self, edit: Edit) -> Result<Edit, EditError> {
        match edit {
            Edit::Insert { item, at } => {
                let id = item.id();
                match item {
                    Item::Material(m) => insert(&mut self.materials, m, at, id)?,
                    Item::Sketch(s) => insert(&mut self.sketches, s, at, id)?,
                    Item::Panel(p) => insert(&mut self.panels, p, at, id)?,
                    Item::Parameter(p) => insert(&mut self.parameters, p, at, id)?,
                }
                Ok(Edit::Remove(id))
            }
            Edit::Remove(id) => {
                let found = match id {
                    ItemId::Material(k) => remove(&mut self.materials, |m| m.id == k, Item::Material),
                    ItemId::Sketch(k) => remove(&mut self.sketches, |s| s.id == k, Item::Sketch),
                    ItemId::Panel(k) => remove(&mut self.panels, |p| p.id == k, Item::Panel),
                    ItemId::Parameter(k) => {
                        remove(&mut self.parameters, |p| p.id == k, Item::Parameter)
                    }
                };
                let (item, at) = found.ok_or(EditError::NotFound(id))?;
                Ok(Edit::Insert { item, at: Some(at) })
            }
            Edit::Replace(item) => {
                let id = item.id();
                let old = match item {
                    Item::Material(m) => {
                        replace(&mut self.materials, m, |a, b| a.id == b.id, Item::Material)
                    }
                    Item::Sketch(s) => replace(&mut self.sketches, s, |a, b| a.id == b.id, Item::Sketch),
                    Item::Panel(p) => replace(&mut self.panels, p, |a, b| a.id == b.id, Item::Panel),
                    Item::Parameter(p) => {
                        replace(&mut self.parameters, p, |a, b| a.id == b.id, Item::Parameter)
                    }
                };
                Ok(Edit::Replace(old.ok_or(EditError::NotFound(id))?))
            }
            Edit::SetName(name) => Ok(Edit::SetName(std::mem::replace(&mut self.name, name))),
            Edit::SetDisplayUnits(units) => {
                Ok(Edit::SetDisplayUnits(std::mem::replace(&mut self.display_units, units)))
            }
            Edit::Group(edits) => {
                let mut inverses = Vec::with_capacity(edits.len());
                for edit in edits {
                    match self.apply_unchecked(edit) {
                        Ok(inverse) => inverses.push(inverse),
                        Err(problem) => {
                            for inverse in inverses.into_iter().rev() {
                                self.apply_unchecked(inverse)
                                    .expect("an edit just made can be reversed");
                            }
                            return Err(problem);
                        }
                    }
                }
                inverses.reverse();
                Ok(Edit::Group(inverses))
            }
        }
    }
}

fn insert<T>(list: &mut Vec<T>, item: T, at: Option<usize>, id: ItemId) -> Result<(), EditError> {
    let at = at.unwrap_or(list.len());
    if at > list.len() {
        return Err(EditError::BadPosition { id, at, len: list.len() });
    }
    list.insert(at, item);
    Ok(())
}

fn remove<T>(
    list: &mut Vec<T>,
    is_it: impl Fn(&T) -> bool,
    wrap: fn(T) -> Item,
) -> Option<(Item, usize)> {
    let at = list.iter().position(is_it)?;
    Some((wrap(list.remove(at)), at))
}

fn replace<T>(
    list: &mut [T],
    new: T,
    same: impl Fn(&T, &T) -> bool,
    wrap: fn(T) -> Item,
) -> Option<Item> {
    let slot = list.iter_mut().find(|old| same(old, &new))?;
    Some(wrap(std::mem::replace(slot, new)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::EntityId;
    use crate::model::{Grain, Plane, SheetSize, ThicknessSide};

    fn birch(doc: &mut Document) -> Material {
        Material {
            id: MaterialId(doc.allocate_id()),
            name: "Birch ply".into(),
            thickness: 3.0,
            kerf: 0.15,
            sheet: SheetSize { width: 600.0, height: 400.0 },
            grain: Grain::None,
        }
    }

    fn rectangle(doc: &mut Document) -> Sketch {
        let id = SketchId(doc.allocate_id());
        let sides = std::array::from_fn(|_| EntityId(doc.allocate_id()));
        Sketch::rectangle(id, "Base outline", Plane::ground(), [0.0, 0.0], 200.0, 120.0, sides)
    }

    fn panel(doc: &mut Document, sketch: SketchId, material: MaterialId) -> Panel {
        Panel {
            id: PanelId(doc.allocate_id()),
            name: "Base".into(),
            sketch,
            material,
            thickness_side: ThicknessSide::Positive,
        }
    }

    fn parameter(doc: &mut Document, name: &str) -> Parameter {
        Parameter {
            id: ParameterId(doc.allocate_id()),
            name: name.into(),
            value: 1.0,
            unit: Default::default(),
        }
    }

    /// A document with one material, one sketch and one panel.
    fn one_panel() -> Document {
        let mut doc = Document::new("Test");
        let m = birch(&mut doc);
        let s = rectangle(&mut doc);
        let p = panel(&mut doc, s.id, m.id);
        doc.apply(Edit::Group(vec![
            Edit::add(Item::Material(m)),
            Edit::add(Item::Sketch(s)),
            Edit::add(Item::Panel(p)),
        ]))
        .unwrap();
        doc
    }

    #[test]
    fn every_edit_is_undone_by_its_inverse() {
        let mut doc = one_panel();
        let mut thicker = doc.materials[0].clone();
        thicker.thickness = 6.0;
        let extra = parameter(&mut doc, "width");
        let start = doc.clone();

        let edits = [
            Edit::Replace(Item::Material(thicker)),
            Edit::add(Item::Parameter(extra)),
            Edit::Remove(ItemId::Panel(doc.panels[0].id)),
            Edit::SetName("Renamed".into()),
            Edit::SetDisplayUnits(DisplayUnits::Inches),
        ];
        for edit in edits {
            let inverse = doc.apply(edit.clone()).unwrap();
            assert_ne!(doc, start, "{edit:?} changed nothing");
            doc.apply(inverse).unwrap();
            assert_eq!(doc, start, "{edit:?} was not fully undone");
        }
    }

    #[test]
    fn a_removed_item_returns_to_its_place() {
        let mut doc = one_panel();
        for name in ["a", "b", "c"] {
            let p = parameter(&mut doc, name);
            doc.apply(Edit::add(Item::Parameter(p))).unwrap();
        }
        let middle = doc.parameters[1].id;
        let inverse = doc.apply(Edit::Remove(ItemId::Parameter(middle))).unwrap();
        assert_eq!(doc.parameters.len(), 2);
        doc.apply(inverse).unwrap();
        let names: Vec<&str> = doc.parameters.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["a", "b", "c"]);
    }

    #[test]
    fn a_group_is_undone_as_one() {
        let mut doc = Document::new("Test");
        let m = birch(&mut doc);
        let s = rectangle(&mut doc);
        let p = panel(&mut doc, s.id, m.id);
        let empty = doc.clone();
        let inverse = doc
            .apply(Edit::Group(vec![
                Edit::add(Item::Material(m)),
                Edit::add(Item::Sketch(s)),
                Edit::add(Item::Panel(p)),
            ]))
            .unwrap();
        assert_eq!((doc.materials.len(), doc.sketches.len(), doc.panels.len()), (1, 1, 1));
        doc.apply(inverse).unwrap();
        assert_eq!(doc, empty);
    }

    #[test]
    fn a_refused_edit_changes_nothing() {
        let mut doc = one_panel();
        let before = doc.clone();

        // The panel still needs its material and its sketch.
        let material = ItemId::Material(doc.materials[0].id);
        assert!(matches!(
            doc.apply(Edit::Remove(material)),
            Err(EditError::Invalid(ModelError::MissingMaterial { .. }))
        ));
        let sketch = ItemId::Sketch(doc.sketches[0].id);
        assert!(matches!(
            doc.apply(Edit::Remove(sketch)),
            Err(EditError::Invalid(ModelError::MissingSketch { .. }))
        ));

        // A material cannot have zero thickness.
        let mut flat = doc.materials[0].clone();
        flat.thickness = 0.0;
        assert!(matches!(
            doc.apply(Edit::Replace(Item::Material(flat))),
            Err(EditError::Invalid(ModelError::BadNumber { .. }))
        ));

        // There is nothing with this id to remove.
        let nothing = ItemId::Panel(PanelId(999));
        assert_eq!(doc.apply(Edit::Remove(nothing)), Err(EditError::NotFound(nothing)));

        // A group that fails part-way leaves no trace of its first half.
        let group = Edit::Group(vec![Edit::SetName("Half done".into()), Edit::Remove(nothing)]);
        assert!(doc.apply(group).is_err());

        assert_eq!(doc, before);
    }

    #[test]
    fn ids_must_be_issued_and_unique() {
        let mut doc = one_panel();
        let mut copy = doc.materials[0].clone();
        assert_eq!(
            doc.apply(Edit::add(Item::Material(copy.clone()))),
            Err(EditError::Invalid(ModelError::DuplicateId(copy.id.0)))
        );
        copy.id = MaterialId(doc.next_id + 5);
        assert_eq!(
            doc.apply(Edit::add(Item::Material(copy.clone()))),
            Err(EditError::Invalid(ModelError::UnissuedId(copy.id.0)))
        );
    }

    #[test]
    fn parameter_names_are_identifiers_and_unique() {
        let mut doc = Document::new("Test");
        let mut add = |name: &str| {
            let p = parameter(&mut doc, name);
            doc.apply(Edit::add(Item::Parameter(p)))
        };
        assert!(add("width").is_ok());
        assert!(add("inner_depth2").is_ok());
        let invalid = |r: Result<Edit, EditError>| match r {
            Err(EditError::Invalid(e)) => e,
            other => panic!("expected a refusal, got {other:?}"),
        };
        assert!(matches!(invalid(add("width")), ModelError::DuplicateParameterName(_)));
        assert!(matches!(invalid(add("2wide")), ModelError::BadParameterName(_)));
        assert!(matches!(invalid(add("box width")), ModelError::BadParameterName(_)));
        assert!(matches!(invalid(add("")), ModelError::BadParameterName(_)));
    }
}
