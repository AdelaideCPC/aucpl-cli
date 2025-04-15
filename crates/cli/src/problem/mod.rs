pub mod archive;
pub mod check;
pub mod create;
pub mod sync_mappings;
pub mod test;

pub const PROBLEM_MAPPINGS_FILE: &str = "problem-mappings.json";
pub const PROBLEM_NAME_REGEX_PATTERN: &str = r"^[A-Za-z0-9_-]+$";
