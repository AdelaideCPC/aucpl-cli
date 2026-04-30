// Provides compile-time inclusion of the shell init scripts.

/// Macro to include a script file from `src/cli/shellinit_scripts` at compile time.
/// Usage: `include_shell!("aucpl.fish")`.
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

pub const FISH: &str = include_shell!("aucpl.fish");
pub const ZSH: &str = include_shell!("aucpl.zsh");
pub const BASH: &str = include_shell!("aucpl.bash");
