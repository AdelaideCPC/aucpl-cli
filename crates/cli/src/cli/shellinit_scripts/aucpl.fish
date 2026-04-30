function aucpl
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

complete -c aucpl -f -a "(__aucpl_complete_fish)";
