use clap::Command;

pub mod problem;
pub mod sync;

pub fn builtin() -> Vec<Command> {
    vec![problem::cli(), sync::cli()]
}
