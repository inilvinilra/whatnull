#compdef whatnull

_whatnull() {
    _arguments \
        '--help[Print help information]' \
        '--version[Print version information]' \
        '--minimized[Start application minimized to tray]' \
        '--profile=[Specify active profile ID]:profile'
}

_whatnull "$@"
