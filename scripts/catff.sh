#!/usr/bin/env bash

set -eu -o pipefail

declare -r PALETTE="$(dirname "$(realpath "$0")")/xterm-colors.txt"

function get-dimension {
    od -An --endian big --format=u4 -w4 -j "$1" -N 4 "$2" | tr -d " "
}

function is-farbfeld {
    [[ "$(head -c8 "$1")" = "farbfeld" ]]
}

function term-wide-enough {
    local cols="$(tput cols)"
    [[ "$1" -lt "$cols" ]]
}

function display-image {
    local -i width ix i=0
    local r g b a color
    width="$(get-dimension 8 "$1")"
    if ! term-wide-enough "$width" && [[ -z "${FORCE+x}" ]]; then
        echo "terminal not wide enough (override with FORCE=1)" >&2
        exit 4
    fi
    while IFS=" " read r g b a; do
        echo "#${r:0:2}${g:0:2}${b:0:2}" >&${COPROC[1]}
        IFS=" " read -u ${COPROC[0]} ix color
        printf "\e[38;5;%dm%s\e[0m" "$ix" "█"
        i=$((i + 1))
        if [[ "$i" -eq "$width" ]]; then
            printf "\n" 
            i=0
        fi
    done < <(od -An --endian=big --format=x2 -v -j16 -w8 "$1")
}

which "palettematch" >/dev/null 2>&1 || {
    echo "could not find palettematch in PATH" >&2
    exit 1
}

if [[ -z "${1+x}" ]]; then
    echo "usage: $0 IMAGE" >&2
    exit 2
fi

if ! is-farbfeld "$1"; then
    echo "input file has to be a farbfeld" >&2
    exit 3
fi

coproc { palettematch "$PALETTE"; }

display-image "$1"
