#!/usr/bin/env bash

set -eu -o pipefail

which "palettematch" >/dev/null 2>&1 || {
    echo "could not find palettematch in PATH" >&2
    exit 1
}

declare -r PALETTE="$(dirname "$(realpath "$0")")/xterm-colors.txt"

coproc { palettematch "$PALETTE"; }

function draw-square {
    local zero=00
    local ix color xh yh r=zero g=zero b=zero
    case "$1" in
        "r") r=xh;;
        "g") g=xh;;
        "b") b=xh;;
        *)
            echo "Invalid color" >&2
            return 1
            ;;
    esac
    case "$2" in
        "r") r=yh;;
        "g") g=yh;;
        "b") b=yh;;
        *)
            echo "Invalid color" >&2
            return 1
            ;;
    esac
    for ((x=0; x<255; x++)); do
        for ((y=0; y<255; y++)); do
            xh="$(printf "%0.2x" "$x")"
            yh="$(printf "%0.2x" "$y")"
            echo "#${!r}${!g}${!b}" >&${COPROC[1]}
            IFS=" " read -ru ${COPROC[0]} ix color
            printf "\e[38;5;%dm%s\e[0m" "$ix" "█"
        done
        printf "\n"
    done
}

draw-square "${1:-g}" "${2:-r}"

