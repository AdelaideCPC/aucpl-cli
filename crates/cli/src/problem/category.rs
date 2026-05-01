//! Category validation utilities for problem management.

use std::sync::OnceLock;

use anyhow::{bail, Result};
use regex::Regex;

const CATEGORY_REGEX_PATTERN: &str = r"^[a-z0-9_-]+$";

fn category_regex() -> &'static Regex {
    static CATEGORY_REGEX: OnceLock<Regex> = OnceLock::new();
    CATEGORY_REGEX
        .get_or_init(|| Regex::new(CATEGORY_REGEX_PATTERN).expect("category regex should be valid"))
}

pub fn validate_category(category: &str) -> Result<()> {
    if !category_regex().is_match(category) {
        bail!(
            "Invalid category '{category}'. Categories may only contain lowercase letters, numbers, dashes, and underscores."
        );
    }

    if matches!(category, "new" | "archive") {
        bail!("Invalid category '{category}'. 'new' and 'archive' are reserved status names.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_category;

    #[test]
    fn accepts_valid_categories() {
        for category in [
            "dp",
            "graphs",
            "0800",
            "unrated",
            "regional_2026",
            "two-pointers",
        ] {
            assert!(
                validate_category(category).is_ok(),
                "{category} should be valid"
            );
        }
    }

    #[test]
    fn rejects_invalid_categories() {
        for category in [
            "Graphs",
            "DP",
            "graph theory",
            "graphs/dp",
            "..",
            ".",
            "",
            "graphs!",
        ] {
            assert!(
                validate_category(category).is_err(),
                "{category:?} should be invalid"
            );
        }
    }

    #[test]
    fn rejects_reserved_status_names() {
        assert!(validate_category("new").is_err());
        assert!(validate_category("archive").is_err());
    }
}
