use anyhow::Result;
use clap::{ArgMatches, Command};

pub fn cli() -> Command {
    Command::new("shellinit")
        .about("Print shell initialization snippet for aucpl command integration")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    _ = args;

    println!(
        r#"aucpl() {{ if [ "$1" = "cd" ]; then shift; local target; target="$(command aucpl cd "$@")" || return $?; builtin cd -- "$target"; else command aucpl "$@"; fi; }}"#
    );

    Ok(())
}
