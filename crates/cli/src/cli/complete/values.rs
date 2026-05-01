//! Completion value providers for clap arguments.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use clap::{Arg, ValueHint};
use serde_json::Value;

use crate::cli::arg_builders::{COMPETITION_VALUE_NAME, PROBLEM_VALUE_NAME};
use crate::comp::COMPETITIONS_FILE;
use crate::config::get_settings;
use crate::problem::sync_mappings::get_all_problem_names;
use crate::util::get_project_root;

/// Dynamic completion categories backed by project data.
enum CompletionKind {
    /// Complete problem names from the problem mapping file.
    Problem,
    /// Complete competition names from the competitions file.
    Competition,
}

/// The source used to produce values for an argument.
enum ValueProvider {
    /// Values come from project-specific dynamic data.
    Dynamic(CompletionKind),
    /// Values are statically declared in clap.
    Static(Vec<String>),
    /// Values should be completed as filesystem paths.
    Path(ValueHint),
    /// No completions are available for this argument.
    None,
}

fn filter_prefix_matches(mut values: Vec<String>, current: &str) -> Vec<String> {
    if current.is_empty() {
        return values;
    }

    values.retain(|value| value.starts_with(current));
    values
}

/// Load competition names from the project's competitions metadata file.
fn get_competition_names(problems_dir: &Path) -> Vec<String> {
    let comp_file_path = problems_dir.join(COMPETITIONS_FILE);
    if !fs::exists(&comp_file_path).unwrap_or(false) {
        return vec![];
    }

    let comp_file = match File::open(&comp_file_path) {
        Ok(file) => file,
        Err(_) => return vec![],
    };

    let data: BTreeMap<String, Value> = match serde_json::from_reader(comp_file) {
        Ok(data) => data,
        Err(_) => return vec![],
    };

    data.keys().cloned().collect()
}

/// Resolve the configured problems directory for the current project, if any.
fn project_problems_dir() -> Option<PathBuf> {
    let project_root = get_project_root().ok()?;
    let settings = get_settings().ok()?;
    Some(project_root.join(&settings.problems_dir))
}

/// Infer the dynamic value category from an argument's configured value name.
fn completion_kind(arg: &Arg) -> Option<CompletionKind> {
    let value_name = arg
        .get_value_names()
        .and_then(|names| names.first())
        .map(|name| name.as_str());

    match value_name {
        Some(PROBLEM_VALUE_NAME) => Some(CompletionKind::Problem),
        Some(COMPETITION_VALUE_NAME) => Some(CompletionKind::Competition),
        _ => None,
    }
}

/// Extract visible static possible values declared in clap for an argument.
fn static_possible_values(arg: &Arg) -> Vec<String> {
    let mut values: Vec<String> = arg
        .get_possible_values()
        .into_iter()
        .filter(|value| !value.is_hide_set())
        .map(|value| value.get_name().to_owned())
        .collect();

    values.sort();
    values.dedup();
    values
}

/// Determine how completions should be produced for a clap argument.
fn value_provider(arg: &Arg) -> ValueProvider {
    if let Some(kind) = completion_kind(arg) {
        return ValueProvider::Dynamic(kind);
    }

    let values = static_possible_values(arg);
    if !values.is_empty() {
        return ValueProvider::Static(values);
    }

    match arg.get_value_hint() {
        ValueHint::AnyPath | ValueHint::FilePath | ValueHint::DirPath => {
            ValueProvider::Path(arg.get_value_hint())
        }
        _ => ValueProvider::None,
    }
}

/// Load filtered dynamic completion values for a project-backed completion kind.
fn dynamic_values(kind: CompletionKind, current: &str) -> Vec<String> {
    let Some(problems_dir) = project_problems_dir() else {
        return vec![];
    };

    let mut values = match kind {
        CompletionKind::Problem => get_all_problem_names(&problems_dir).unwrap_or_default(),
        CompletionKind::Competition => get_competition_names(&problems_dir),
    };

    values.sort();
    filter_prefix_matches(values, current)
}

/// Complete filesystem path candidates according to the provided [`ValueHint`].
fn path_candidates(current: &str, hint: ValueHint) -> Vec<String> {
    let allow_files = matches!(hint, ValueHint::AnyPath | ValueHint::FilePath);
    let allow_dirs = matches!(
        hint,
        ValueHint::AnyPath | ValueHint::FilePath | ValueHint::DirPath
    );

    if !allow_files && !allow_dirs {
        return vec![];
    }

    let (base_dir, display_prefix, leaf_prefix) = if current.is_empty() {
        (PathBuf::from("."), String::new(), "")
    } else if current.ends_with('/') {
        (PathBuf::from(current), current.to_owned(), "")
    } else if let Some(idx) = current.rfind('/') {
        (
            PathBuf::from(&current[..=idx]),
            current[..=idx].to_owned(),
            &current[idx + 1..],
        )
    } else {
        (PathBuf::from("."), String::new(), current)
    };

    let entries = match fs::read_dir(&base_dir) {
        Ok(entries) => entries,
        Err(_) => return vec![],
    };

    let mut values = BTreeSet::new();

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };

        if !leaf_prefix.is_empty() && !file_name.starts_with(leaf_prefix) {
            continue;
        }

        if leaf_prefix.is_empty() && !current.starts_with('.') && file_name.starts_with('.') {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            if allow_dirs {
                values.insert(format!("{display_prefix}{file_name}/"));
            }
            continue;
        }

        if allow_files {
            values.insert(format!("{display_prefix}{file_name}"));
        }
    }

    values.into_iter().collect()
}

/// Produce completion candidates for the argument value under the cursor.
pub(super) fn complete_arg_values(arg: &Arg, current: &str) -> Vec<String> {
    match value_provider(arg) {
        ValueProvider::Dynamic(kind) => dynamic_values(kind, current),
        ValueProvider::Static(values) => filter_prefix_matches(values, current),
        ValueProvider::Path(hint) => path_candidates(current, hint),
        ValueProvider::None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};

    use clap::{Arg, ArgAction, ValueHint};
    use tempfile::TempDir;

    use super::*;
    use crate::cli::arg_builders::{competition_arg_optional, problem_arg_optional};
    use crate::config::SETTINGS_FILE_DEFAULT_CONTENTS;
    use crate::problem::PROBLEM_MAPPINGS_FILE;

    static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn cwd_lock() -> &'static Mutex<()> {
        CWD_LOCK.get_or_init(|| Mutex::new(()))
    }

    struct CurrentDirGuard {
        previous: PathBuf,
    }

    impl CurrentDirGuard {
        fn enter(path: &Path) -> Self {
            let previous = env::current_dir().expect("current dir should be readable");
            env::set_current_dir(path).expect("current dir should be changeable");
            Self { previous }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            env::set_current_dir(&self.previous).expect("current dir should be restorable");
        }
    }

    fn with_test_project(test: impl FnOnce(&Path)) {
        let _guard = cwd_lock().lock().expect("cwd lock should not be poisoned");
        let tempdir = TempDir::new().expect("tempdir should be created");
        let project_root = tempdir.path();
        let problems_dir = project_root.join("problems");

        fs::write(
            project_root.join(crate::config::SETTINGS_FILE_NAME),
            SETTINGS_FILE_DEFAULT_CONTENTS,
        )
        .expect("settings file should be written");
        fs::create_dir_all(&problems_dir).expect("problems dir should be created");

        let _cwd = CurrentDirGuard::enter(project_root);
        test(&problems_dir);
    }

    #[test]
    fn problem_names_are_prefix_filtered() {
        with_test_project(|problems_dir| {
            fs::write(
                problems_dir.join(PROBLEM_MAPPINGS_FILE),
                r#"{"alpha":"problems/new/0800/alpha","beta":"problems/new/0800/beta","alpine":"problems/new/1000/alpine"}"#,
            )
            .expect("problem mappings should be written");

            let values = complete_arg_values(&problem_arg_optional(), "al");

            assert_eq!(values, vec!["alpha", "alpine"]);
        });
    }

    #[test]
    fn competition_names_are_prefix_filtered() {
        with_test_project(|problems_dir| {
            fs::write(
                problems_dir.join(COMPETITIONS_FILE),
                r#"{"acpc-warmup":{"finished":false,"problems":[]},"beta-round":{"finished":true,"problems":["beta"]},"acpc-finals":{"finished":false,"problems":["alpha"]}}"#,
            )
            .expect("competitions file should be written");

            let values = complete_arg_values(&competition_arg_optional(), "acpc-");

            assert_eq!(values, vec!["acpc-finals", "acpc-warmup"]);
        });
    }

    #[test]
    fn static_possible_values_are_prefix_filtered() {
        let arg = Arg::new("lang")
            .action(ArgAction::Set)
            .value_parser(["cpp", "py", "java"]);

        let values = complete_arg_values(&arg, "p");

        assert_eq!(values, vec!["py"]);
    }

    #[test]
    fn path_completion_keeps_directory_trailing_slashes() {
        let _guard = cwd_lock().lock().expect("cwd lock should not be poisoned");
        let tempdir = TempDir::new().expect("tempdir should be created");
        fs::create_dir(tempdir.path().join("alpha")).expect("dir should be created");
        fs::write(tempdir.path().join("beta.txt"), "x").expect("file should be created");
        let _cwd = CurrentDirGuard::enter(tempdir.path());

        let arg = Arg::new("path")
            .action(ArgAction::Set)
            .value_hint(ValueHint::AnyPath);
        let values = complete_arg_values(&arg, "a");

        assert_eq!(values, vec!["alpha/"]);
    }
}
