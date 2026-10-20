#!/usr/bin/env bash
# vez_pesada.sh — no máximo DUAS corridas pesadas da linha ao mesmo tempo (00_MISSAO §4: a fatia é UMA, 24 G),
# entre os agentes de família que trabalham em paralelo. Espera uma das duas vagas (até 2 h) e corre o comando.
#
# uso: vez_pesada.sh <comando> [args…]
#   ex.: PH2D_EXAME_DIR=/tmp/ph2d_exame_grafite bash docs/Painter/Pinceis/ferramentas/vez_pesada.sh \
#          bash docs/Painter/Pinceis/ferramentas/exame.sh "Graphite HB" docs/Painter/Pinceis/fixtures/rebelle_bateria/PENCIL/graphite_pencil_hb
set -euo pipefail
[[ $# -gt 0 ]] || { sed -n '2,7p' "$0"; exit 2; }
BASE=${XDG_RUNTIME_DIR:-/tmp}/ph2d_pinceis_pesado
exec 6>"$BASE.1.lock" 7>"$BASE.2.lock"
fim=$((SECONDS + 7200))
avisou=""
while :; do
  if flock -n 6; then exec 7>&-; break; fi
  if flock -n 7; then exec 6>&-; break; fi
  [[ -z $avisou ]] && { echo "▸ as duas vagas pesadas estão ocupadas; à espera…" >&2; avisou=1; }
  ((SECONDS < fim)) || { echo "vez_pesada.sh: nenhuma vaga em 2 h" >&2; exit 3; }
  flock -w 20 6 && { exec 7>&-; break; }
done
"$@" 6>&- 7>&-
