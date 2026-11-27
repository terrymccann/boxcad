//! Box CAD's core: the document model and all cut geometry.
//!
//! This crate has no UI dependency. The app and the command-line tool both call
//! it, and the interface never computes geometry of its own. Every length is in
//! millimeters; inches exist only as a display setting.

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
