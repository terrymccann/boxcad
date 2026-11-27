//! Undo and redo: a document together with the edits that step it backward and
//! forward, and a note of which state was last saved.

use crate::edit::{Edit, EditError};
use crate::model::Document;

/// One entry in the undo or redo list.
#[derive(Debug, Clone)]
struct Step {
    /// What the user did, such as "Draw panel", for the history list.
    label: String,
    /// The edit that takes this step.
    edit: Edit,
    /// The state number the document has once the step is taken.
    leads_to: u64,
}

/// A document with its undo history. The history is kept in memory only; it is
/// not written to the project file.
#[derive(Debug, Clone)]
pub struct History {
    document: Document,
    undo: Vec<Step>,
    redo: Vec<Step>,
    /// Every distinct state of the document gets its own number.
    state: u64,
    next_state: u64,
    saved_state: Option<u64>,
}

impl History {
    /// Start a history at `document`, which counts as saved: use this for a new
    /// design or one just opened.
    pub fn new(document: Document) -> Self {
        History {
            document,
            undo: Vec::new(),
            redo: Vec::new(),
            state: 0,
            next_state: 1,
            saved_state: Some(0),
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Issue an id for something about to be added. See [`Document::allocate_id`].
    pub fn allocate_id(&mut self) -> u64 {
        self.document.allocate_id()
    }

    /// Make an edit under a label for the history list. Anything that had been
    /// undone can no longer be redone.
    pub fn apply(&mut self, label: impl Into<String>, edit: Edit) -> Result<(), EditError> {
        let inverse = self.document.apply(edit)?;
        self.undo.push(Step { label: label.into(), edit: inverse, leads_to: self.state });
        self.redo.clear();
        self.state = self.next_state;
        self.next_state += 1;
        Ok(())
    }

    /// Step back one edit. Returns its label, or `None` if there is nothing to undo.
    pub fn undo(&mut self) -> Option<String> {
        Self::step(&mut self.document, &mut self.state, &mut self.undo, &mut self.redo)
    }

    /// Step forward one undone edit. Returns its label, or `None` if there is
    /// nothing to redo.
    pub fn redo(&mut self) -> Option<String> {
        Self::step(&mut self.document, &mut self.state, &mut self.redo, &mut self.undo)
    }

    fn step(
        document: &mut Document,
        state: &mut u64,
        from: &mut Vec<Step>,
        to: &mut Vec<Step>,
    ) -> Option<String> {
        let step = from.pop()?;
        let back = document
            .apply(step.edit)
            .expect("a recorded edit applies to the state it was recorded for");
        to.push(Step { label: step.label.clone(), edit: back, leads_to: *state });
        *state = step.leads_to;
        Some(step.label)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Labels of the edits that can be undone, oldest first.
    pub fn undo_labels(&self) -> impl Iterator<Item = &str> {
        self.undo.iter().map(|s| s.label.as_str())
    }

    /// Labels of the edits that can be redone, next one first.
    pub fn redo_labels(&self) -> impl Iterator<Item = &str> {
        self.redo.iter().rev().map(|s| s.label.as_str())
    }

    /// Record that the document as it is now has been written to its file.
    pub fn mark_saved(&mut self) {
        self.saved_state = Some(self.state);
    }

    /// Whether the document differs from what was last saved. Undoing back to
    /// the saved state makes this false again.
    pub fn is_modified(&self) -> bool {
        self.saved_state != Some(self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::Item;
    use crate::id::ParameterId;
    use crate::model::Parameter;

    fn parameter(history: &mut History, name: &str, value: f64) -> Parameter {
        Parameter {
            id: ParameterId(history.allocate_id()),
            name: name.into(),
            value,
            unit: Default::default(),
        }
    }

    fn add(history: &mut History, name: &str) {
        let p = parameter(history, name, 1.0);
        history.apply(format!("Add {name}"), Edit::add(Item::Parameter(p))).unwrap();
    }

    fn names(history: &History) -> Vec<&str> {
        history.document().parameters.iter().map(|p| p.name.as_str()).collect()
    }

    #[test]
    fn undo_and_redo_walk_the_list() {
        let mut h = History::new(Document::new("Test"));
        add(&mut h, "a");
        add(&mut h, "b");
        add(&mut h, "c");
        assert_eq!(h.undo_labels().collect::<Vec<_>>(), ["Add a", "Add b", "Add c"]);

        assert_eq!(h.undo().as_deref(), Some("Add c"));
        assert_eq!(h.undo().as_deref(), Some("Add b"));
        assert_eq!(names(&h), ["a"]);
        assert_eq!(h.redo_labels().collect::<Vec<_>>(), ["Add b", "Add c"]);

        assert_eq!(h.redo().as_deref(), Some("Add b"));
        assert_eq!(names(&h), ["a", "b"]);
        assert_eq!(h.redo().as_deref(), Some("Add c"));
        assert_eq!(names(&h), ["a", "b", "c"]);
        assert_eq!(h.redo(), None);

        while h.undo().is_some() {}
        assert!(names(&h).is_empty());
        assert_eq!(h.undo(), None);
    }

    #[test]
    fn a_new_edit_discards_what_was_undone() {
        let mut h = History::new(Document::new("Test"));
        add(&mut h, "a");
        add(&mut h, "b");
        h.undo();
        assert!(h.can_redo());
        add(&mut h, "c");
        assert!(!h.can_redo());
        assert_eq!(names(&h), ["a", "c"]);
    }

    #[test]
    fn a_refused_edit_is_not_recorded() {
        let mut h = History::new(Document::new("Test"));
        add(&mut h, "a");
        let clash = parameter(&mut h, "a", 2.0);
        assert!(h.apply("Add a again", Edit::add(Item::Parameter(clash))).is_err());
        assert_eq!(h.undo_labels().count(), 1);
    }

    #[test]
    fn modified_tracks_the_saved_state() {
        let mut h = History::new(Document::new("Test"));
        assert!(!h.is_modified());
        add(&mut h, "a");
        assert!(h.is_modified());
        h.mark_saved();
        assert!(!h.is_modified());

        add(&mut h, "b");
        assert!(h.is_modified());
        h.undo();
        assert!(!h.is_modified(), "undoing back to the saved state is not a change");
        h.undo();
        assert!(h.is_modified());
        h.redo();
        assert!(!h.is_modified());

        // Undo past the save, then edit: the saved state can no longer be reached.
        h.undo();
        add(&mut h, "c");
        h.undo();
        assert!(h.is_modified());
    }
}
