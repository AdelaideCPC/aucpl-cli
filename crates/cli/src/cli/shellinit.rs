use anyhow::Result;
use clap::{ArgMatches, Command};

use crate::cli::shellinit_scripts::{BASH, FISH, ZSH};

pub fn cli() -> Command {
    Command::new("shellinit")
        .about("Print shell initialization snippet for aucpl command integration")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    _ = args;

    let shell = std::env::var("SHELL").unwrap_or_default();
    let is_fish = std::env::var_os("FISH_VERSION").is_some();
    let is_zsh = std::env::var_os("ZSH_VERSION").is_some() || shell.ends_with("/zsh");

    if is_fish {
        let fish_script = FISH;
        println!("{}", fish_script);
        return Ok(());
    }

    if is_zsh {
        let zsh_script = ZSH;
        println!("{}", zsh_script);
        return Ok(());
    }

    let bash_script = BASH;
    println!("{}", bash_script);

    Ok(())
}
