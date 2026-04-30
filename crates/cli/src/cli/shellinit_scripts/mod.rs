//! Provides compile-time inclusion of the shell init scripts.

/// Macro to include a script file from `src/cli/shellinit_scripts` at compile time.
/// Usage example: `include_shell!("aucpl.bash")`.
#[macro_export]
macro_rules! include_shell {
    ($file:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/cli/shellinit_scripts/",
            $file
        ))
    };
}

pub const BASH: &str = include_shell!("aucpl.bash");
pub const FISH: &str = include_shell!("aucpl.fish");
pub const ZSH: &str = include_shell!("aucpl.zsh");
