//! Identifiers. Every material, sketch, sketch entity, panel and parameter in a
//! document has a number that is unique across the whole document and is never
//! reused, so a reference stays valid however the thing it names is edited.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! id_type {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub u64);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

id_type!(
    /// Names a material in the document.
    MaterialId
);
id_type!(
    /// Names a sketch.
    SketchId
);
id_type!(
    /// Names one line, arc or circle inside a sketch.
    EntityId
);
id_type!(
    /// Names a panel.
    PanelId
);
id_type!(
    /// Names a parameter.
    ParameterId
);
