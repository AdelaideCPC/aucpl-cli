//! Path utilities for cross-platform compatibility.
//!
//! This module provides functions to normalize paths to Unix-style format
//! for storage, and to convert them back to platform-native paths when reading.
//! This ensures compatibility between Windows and Unix systems.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Convert a path to Unix-style format for storage.
///
/// This function takes any path and converts it to use forward slashes
/// regardless of the platform. This is useful for storing paths in JSON
/// files that need to be shared between Windows and Unix systems.
///
/// # Examples
///
/// On Windows:
///
/// ```rust,ignore
/// let path = Path::new("problems\\new\\unrated\\my-problem");
/// assert_eq!(to_unix_path(path), "problems/new/unrated/my-problem");
/// ```
///
/// On Unix:
///
/// ```rust,ignore
/// let path = Path::new("problems/new/unrated/my-problem");
/// assert_eq!(to_unix_path(path), "problems/new/unrated/my-problem");
/// ```
pub fn to_unix_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Convert a stored Unix-style path to a platform-native PathBuf.
///
/// This function takes a path string that uses forward slashes (as stored
/// by [`to_unix_path`]) and converts it to a PathBuf that is appropriate
/// for the current platform.
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
/// use aucpl_cli::paths::from_unix_path;
///
/// let path = from_unix_path("problems/new/easy/my-problem");
/// // On Windows: PathBuf::from("problems/new/easy/my-problem")
/// // On Unix: PathBuf::from("problems/new/easy/my-problem")
/// ```
pub fn from_unix_path(path_str: &str) -> PathBuf {
    path_str.split('/').collect()
}

/// Normalize a path for storage in JSON files.
///
/// This is a convenience wrapper around [`to_unix_path`] that also handles
/// relative path calculation from a base directory.
pub fn normalize_for_storage(base: &Path, target: &Path) -> Result<String> {
    let relative_path = target
        .strip_prefix(base)
        .with_context(|| {
            format!(
                "Path '{}' is not within base directory '{}'",
                target.display(),
                base.display()
            )
        })?
        .to_path_buf();

    Ok(to_unix_path(&relative_path))
}

/// Resolve a stored Unix-style path against a base directory.
///
/// This function takes a base directory and a stored Unix-style path string
/// and returns a complete PathBuf that is appropriate for the current platform.
pub fn resolve_stored_path(base: &Path, relative_unix_path: &str) -> PathBuf {
    base.join(from_unix_path(relative_unix_path))
}

/// Convert a potentially Windows-style path to Unix format.
///
/// This function handles the case where a path might contain backslashes
/// (from an older version of the tool or from manual editing) and converts
/// them to forward slashes. This is useful for backward compatibility.
pub fn convert_legacy_path(path_str: &str) -> String {
    path_str.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_unix_path_simple() {
        let path = Path::new("problems/new/easy/my-problem");
        assert_eq!(to_unix_path(path), "problems/new/easy/my-problem");
    }

    #[test]
    fn test_to_unix_path_with_empty_components() {
        let path = Path::new("problems//new");
        assert_eq!(to_unix_path(path), "problems/new");
    }

    #[test]
    fn test_from_unix_path() {
        let result = from_unix_path("problems/new/easy/my-problem");
        assert_eq!(result, PathBuf::from("problems/new/easy/my-problem"));
    }

    #[test]
    fn test_from_unix_path_single() {
        let result = from_unix_path("problems");
        assert_eq!(result, PathBuf::from("problems"));
    }

    #[test]
    fn test_normalize_for_storage() {
        let base = Path::new("/home/user/project");
        let target = Path::new("/home/user/project/problems/new/easy/my-problem");
        let result = normalize_for_storage(base, target).unwrap();
        assert_eq!(result, "problems/new/easy/my-problem");
    }

    #[test]
    fn test_normalize_for_storage_failure() {
        let base = Path::new("/home/user/project");
        let target = Path::new("/other/path");
        assert!(normalize_for_storage(base, target).is_err());
    }

    #[test]
    fn test_resolve_stored_path() {
        let base = Path::new("/home/user/project");
        let result = resolve_stored_path(base, "problems/new/easy/my-problem");
        assert_eq!(
            result,
            PathBuf::from("/home/user/project/problems/new/easy/my-problem")
        );
    }

    #[test]
    fn test_convert_legacy_path() {
        let windows_path = "problems\\new\\easy\\my-problem";
        assert_eq!(
            convert_legacy_path(windows_path),
            "problems/new/easy/my-problem"
        );
    }

    #[test]
    fn test_convert_legacy_path_mixed() {
        let mixed_path = "problems/new\\easy/my-problem";
        assert_eq!(
            convert_legacy_path(mixed_path),
            "problems/new/easy/my-problem"
        );
    }

    #[test]
    fn test_roundtrip() {
        let original = Path::new("problems/new/easy/my-problem");
        let unix_path = to_unix_path(original);
        let restored = from_unix_path(&unix_path);
        assert_eq!(original, restored.as_path());
    }
}
