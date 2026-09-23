#!/usr/bin/env bash
# oraculo.sh — UM comando: monta o vendor (se faltar), exporta o jogo com o exportador do
# GDevelop e corre os cenários no Chrome headless, gravando as fixtures.
#
#   bash docs/Components/ferramentas/gdevelop_health/oraculo.sh              # todos os cenários
#   bash docs/Components/ferramentas/gdevelop_health/oraculo.sh b_cooldown   # só alguns
#
# Nenhuma janela: o Chrome corre em `--headless` (modo novo) com DISPLAY e WAYLAND_DISPLAY
# esvaziados no ambiente dele. Tudo o que é temporário fica em vendor/ (ignorado pelo git):
# o código gerado pelo carregador do IDE (TMPDIR), o jogo exportado e o perfil do Chrome.
set -euo pipefail

AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
V="$AQUI/vendor"

bash "$AQUI/setup.sh"

mkdir -p "$V/tmp"
# o LocalEventsFunctionCodeWriter do IDE escreve em os.tmpdir(): aponta-o para o vendor
# SÓ no export (o Chrome recusa um TMPDIR comprido: «Socket path too long»).
TMPDIR="$V/tmp" node "$AQUI/exporta.mjs" "$V/jogo" > "$V/jogo.export.log"
echo "export: $(grep -o '"export_ok": [a-z]*' "$V/jogo.export.log") · $(grep -o '"gdevelop_libgd_version": "[^"]*"' "$V/jogo.export.log")"

node "$AQUI/corre.mjs" "$V/jogo" "$AQUI/fixtures" "$@"
