use anyhow::Result;
use clap::{ArgMatches, Command};

fn one_line_snippet(script: &str) -> String {
    script
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn cli() -> Command {
    Command::new("shellinit")
        .about("Print shell initialization snippet for aucpl command integration")
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    _ = args;

    let is_fish = std::env::var_os("FISH_VERSION").is_some();

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

        println!("{}", one_line_snippet(fish_script));
        return Ok(());
    }

    let sh_script = r#"aucpl() {
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

_aucpl_complete_zsh() {
    local -a suggestions;
    suggestions=("${(@f)$(command aucpl __complete --cword "$((CURRENT-1))" -- "${words[@]}" 2>/dev/null)}");
    compadd -a suggestions;
};

if [ -n "${BASH_VERSION-}" ] && command -v complete >/dev/null 2>&1; then
    complete -o default -F _aucpl_complete_bash aucpl;
fi;

if [ -n "${ZSH_VERSION-}" ] && command -v compdef >/dev/null 2>&1; then
    compdef _aucpl_complete_zsh aucpl;
fi;"#;

    println!("{}", one_line_snippet(sh_script));

    Ok(())
}
