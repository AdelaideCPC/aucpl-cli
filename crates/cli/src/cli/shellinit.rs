use anyhow::Result;
use clap::{ArgMatches, Command};
use crate::cli::shellinit_scripts;

pub fn cli() -> Command {
    Command::new("shellinit")
        .about("Print shell initialization snippet for aucpl command integration")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    _ = args;

    let is_fish = std::env::var_os("FISH_VERSION").is_some();
    let shell = std::env::var("SHELL").unwrap_or_default();
    let is_zsh = std::env::var_os("ZSH_VERSION").is_some() || shell.ends_with("/zsh");

    if is_fish {
        let fish_script = shellinit_scripts::FISH;
        println!("{}", fish_script);
        return Ok(());
    }

    if is_zsh {
        let zsh_script = shellinit_scripts::ZSH;
        println!("{}", zsh_script);
        return Ok(());
    }

    let bash_script = shellinit_scripts::BASH;
    println!("{}", bash_script);

    Ok(())
}
