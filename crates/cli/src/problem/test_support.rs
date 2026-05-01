use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use tempfile::TempDir;

use crate::config::SETTINGS_FILE_DEFAULT_CONTENTS;

static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) fn cwd_lock() -> &'static Mutex<()> {
    CWD_LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    pub(crate) fn enter(path: &Path) -> Self {
        let previous = env::current_dir().expect("current dir should be readable");
        env::set_current_dir(path).expect("current dir should be changeable");
        Self { previous }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.previous);
    }
}

pub fn with_test_project(test: impl FnOnce(&Path)) {
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

pub fn create_problem_dir(problems_dir: &Path, status: &str, category: &str, problem_name: &str) {
    let problem_dir = problems_dir.join(status).join(category).join(problem_name);
    fs::create_dir_all(problem_dir.join("solutions")).expect("solutions dir should be created");
    fs::create_dir_all(problem_dir.join("tests")).expect("tests dir should be created");
    fs::write(problem_dir.join("problem.md"), "# Problem\n")
        .expect("problem statement should be created");
}
