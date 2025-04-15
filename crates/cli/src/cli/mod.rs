use clap::Command;

pub mod comp;
pub mod problem;
pub mod publish;
pub mod sync;

pub fn builtin() -> Vec<Command> {
    vec![comp::cli(), problem::cli(), publish::cli(), sync::cli()]
}
