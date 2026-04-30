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

_aucpl_complete_bash() {
    COMPREPLY=();
    while IFS= read -r reply; do
        COMPREPLY+=("$reply");
    done < <(command aucpl __complete --cword "$COMP_CWORD" -- "${COMP_WORDS[@]}" 2>/dev/null);
};

complete -o default -F _aucpl_complete_bash aucpl;
