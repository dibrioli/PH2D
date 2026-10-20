#!/bin/bash
# Corre um comando wine no prefixo da app, na tela dela. Uso: bash w.sh <app> <prazo_s> <args do wine...>
source "$HOME/Apps/pincel-oraculos/env.sh" "$1"; P="$2"; shift 2
exec timeout "$P" wine "$@"
