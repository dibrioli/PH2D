#!/usr/bin/env bash
# check-workflow-packages.sh — todo nome de pacote que os workflows do CI citam EXISTE na workspace?
#
# ⛔ A classe de defeito (medida no envio de 2026-09-13, run 34757814212): o `spike.yml` corre
# `cargo nextest run -p <26 nomes>`, e um deles (`ph2d-editor`, o shim deprecado apagado em
# `d94e4155c` a 12/09) já não existia. O cargo recusa a especificação ANTES de compilar — os três
# sistemas reprovaram em segundos, e nenhum teste correu. O `ship.sh` estava verde (13 de 13) porque
# corre `--workspace`, que não nomeia pacote nenhum: *apagar uma crate não acorda portão local
# nenhum, só o CI*.
#
# ⇒ a lista de membros sai do `cargo metadata` (nunca à mão) e os nomes saem dos workflows, com
# piso nas duas pontas — uma varredura que não achasse `-p` nenhum ficaria verde a medir nada.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2

# Medido em 2026-09-13: 359 membros; 32 nomes distintos citados pelos workflows.
PISO_MEMBROS=300
PISO_NOMES=20

members=$(cargo metadata --no-deps --format-version 1 --offline 2>/dev/null |
    python3 -c "import json, sys; print('\n'.join(p['name'] for p in json.load(sys.stdin)['packages']))")
n_members=$(printf '%s\n' "$members" | grep -c .)
if [ "$n_members" -lt "$PISO_MEMBROS" ]; then
    echo "✗ o cargo metadata devolveu $n_members membros (piso $PISO_MEMBROS) — a lista não foi lida"
    exit 1
fi

names=$(grep -hoE '(-p|--package)[= ]+[a-z0-9_-]+' .github/workflows/*.yml |
    awk '{ print $NF }' | sed 's/^--package=//' | sort -u)
n_names=$(printf '%s\n' "$names" | grep -c .)
if [ "$n_names" -lt "$PISO_NOMES" ]; then
    echo "✗ os workflows citam $n_names nomes de pacote (piso $PISO_NOMES) — o padrão deixou de casar?"
    exit 1
fi

fail=0
while read -r name; do
    [ -z "$name" ] && continue
    if ! printf '%s\n' "$members" | grep -qx "$name"; then
        echo "  ✗ $name — citado em $(grep -lE "(-p|--package)[= ]+$name([^a-z0-9_-]|\$)" .github/workflows/*.yml | tr '\n' ' ')mas não é membro da workspace"
        fail=1
    fi
done <<< "$names"
echo "  ($n_names nomes citados pelos workflows contra $n_members membros)"
exit "$fail"
