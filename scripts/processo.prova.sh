#!/usr/bin/env bash
# Provas das regras de PROCESSO da comissão de velocidade (29/10): cada regra nova dos scripts de
# teste/integração/envio tem aqui a linha que a vê VERMELHA quando a lei sai. Sem compilar (< 5 s).
#     bash scripts/processo.prova.sh
# O `ship.sh` corre-a; quem mexe num destes scripts corre-a antes de comitar.
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 1
falhas=0
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
ok() { printf '  ✓ %s\n' "$1"; }
ko() { printf '  ✗ %s\n' "$1"; falhas=$((falhas + 1)); }
igual() { if [ "$2" = "$3" ]; then ok "$1"; else ko "$1 (esperava «$3», veio «$2»)"; fi; }

# Cada item tem o seu ficheiro em `processo.prova.d/` (usa `ok`/`ko`/`igual`, `$tmp`, a raiz como cwd).
for _p in scripts/processo.prova.d/*.sh; do
  # shellcheck disable=SC1090
  source "$_p"
  echo
done
if [ "$falhas" -eq 0 ]; then echo "✓ processo.prova: tudo verde"; else echo "✗ processo.prova: $falhas falha(s)"; fi
exit "$falhas"
