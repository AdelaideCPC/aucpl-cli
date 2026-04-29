use anyhow::Result;
use clap::{ArgMatches, Command};

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
        let fish_script = r#"function aucpl
    if test (count $argv) -gt 0; and test "$argv[1]" = "cd"
        set -e argv[1];
        set -l target (command aucpl cd $argv); or return $status;
        builtin cd -- $target;
    else
        command aucpl $argv;
    end;
end;

function __aucpl_complete_fish
    set -l tokens (commandline -opc);
    set -l current (commandline -ct);
    set -l cword (count $tokens);
    command aucpl __complete --cword $cword -- $tokens $current 2>/dev/null;
end;

complete -c aucpl -f -a "(__aucpl_complete_fish)";"#;

        println!("{}", fish_script);
        return Ok(());
    }

    if is_zsh {
        let zsh_script = r#"aucpl() {
    if [ "$1" = "cd" ]; then
        shift;
        local target;
        target="$(command aucpl cd "$@")" || return $?;
        builtin cd -- "$target";
    else
        command aucpl "$@";
    fi;
};

_aucpl_complete_zsh() {
    local -a suggestions;
    suggestions=("${(@f)$(command aucpl __complete --cword "$((CURRENT-1))" -- "${words[@]}" 2>/dev/null)}");
    compadd -a suggestions;
};

compdef _aucpl_complete_zsh aucpl;"#;

        println!("{}", zsh_script);
        return Ok(());
    }

    let bash_script = r#"aucpl() {
    if [ "$1" = "cd" ]; then
        shift;
        local target;
        target="$(command aucpl cd "$@")" || return $?;
        builtin cd -- "$target";
    else
        command aucpl "$@";
    fi;
};

_aucpl_complete_bash() {
    COMPREPLY=();
    while IFS= read -r reply; do
        COMPREPLY+=("$reply");
    done < <(command aucpl __complete --cword "$COMP_CWORD" -- "${COMP_WORDS[@]}" 2>/dev/null);
};

complete -o default -F _aucpl_complete_bash aucpl;"#;

    println!("{}", bash_script);

    Ok(())
}
