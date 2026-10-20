#!/bin/bash
# xdotool na tela da app. Uso: bash x.sh <app> <args do xdotool...>
source "$HOME/Apps/pincel-oraculos/env.sh" "$1"; shift
exec timeout 60 xdotool "$@"
