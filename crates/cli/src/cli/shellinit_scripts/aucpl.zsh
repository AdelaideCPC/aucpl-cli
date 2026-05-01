aucpl() {
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

compdef _aucpl_complete_zsh aucpl;
