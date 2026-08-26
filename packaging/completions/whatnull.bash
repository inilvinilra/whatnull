# bash completion for whatnull

_whatnull_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="--help --version --minimized --profile"

    case "${prev}" in
        --profile)
            COMPREPLY=( $(compgen -W "default work personal" -- ${cur}) )
            return 0
            ;;
        *)
            ;;
    esac

    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
    return 0
}

complete -F _whatnull_completion whatnull
