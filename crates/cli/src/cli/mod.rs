use clap::Command;

pub mod cd;
pub mod comp;
pub mod init;
pub mod problem;
pub mod publish;
pub mod shellinit;
pub mod sync;

pub fn builtin() -> Vec<Command> {
    vec![
        cd::cli(),
        comp::cli(),
        init::cli(),
        problem::cli(),
        publish::cli(),
        shellinit::cli(),
        sync::cli(),
    ]
}
