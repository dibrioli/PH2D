#!/bin/bash
# winetricks no prefixo da app (sem root). Uso: bash wt.sh <app> <prazo_s> <verbos...>
source "$HOME/Apps/pincel-oraculos/env.sh" "$1"; P="$2"; shift 2
export WINEDLLOVERRIDES="winemenubuilder.exe=d"
exec timeout "$P" winetricks -q --unattended "$@"
