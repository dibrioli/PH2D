#!/usr/bin/env bash
# vez_do_app.sh — a VEZ de um oráculo exclusivo entre os agentes que trabalham em paralelo (00_MISSAO §4):
# o Rebelle e o ArtRage têm UMA tela virtual, o Krita um perfil partilhado. Espera a vez (até 2 h) e corre o comando.
#
# uso: vez_do_app.sh <rebelle|artrage|krita|mypaint> <comando> [args…]
#   ex.: bash docs/Painter/Pinceis/ferramentas/vez_do_app.sh rebelle bash docs/Painter/Pinceis/ferramentas/lote_bateria.sh <lista> <saida>
set -euo pipefail
APP=${1:?app}
shift
case "$APP" in rebelle | artrage | krita | mypaint) ;; *) echo "vez_do_app.sh: app desconhecido: $APP" >&2; exit 2 ;; esac
LOCK=${XDG_RUNTIME_DIR:-/tmp}/ph2d_pinceis_vez_$APP.lock
exec 8>"$LOCK"
echo "▸ à espera da vez do $APP ($LOCK)…" >&2
flock -w 7200 8 || { echo "vez_do_app.sh: a vez do $APP não chegou em 2 h" >&2; exit 3; }
echo "▸ a vez do $APP é minha" >&2
"$@" 8>&-
