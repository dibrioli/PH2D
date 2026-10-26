#!/usr/bin/env bash
# Bancada da W4: uma cena por processo, cada uma com prazo próprio (um solver que pendura só perde a sua cena).
# fiksi_lbfgs fica fora das cenas «misto»: pendurou >25 min numa só chamada `solve` em misto_50 (medido 26/10).
set -u
cd "$(git rev-parse --show-toplevel)"
B=target/bancada_solvers/release/bancada_solvers_w4
F=docs/BlenderConstraints/fixtures
OUT=docs/BlenderConstraints/reports/w4_medicao
mkdir -p "$OUT"
bash scripts/ph2d-run.sh cargo build --release --manifest-path docs/BlenderConstraints/ferramentas/bancada_solvers/Cargo.toml \
    --target-dir target/bancada_solvers || exit 2
for cena in ${W4_SO:-escada_50 escada_202 escada_802 misto_50 misto_200 misto_800}; do
    sem=""
    case "$cena" in misto_*) sem="fiksi_lbfgs" ;; esac
    W4_SEM="$sem" timeout "${W4_PRAZO:-900}" "$B" mede "$F/w4_cenas" "$F/freecad_1_1_4_sketch_arrasto.json" "$OUT" "$cena" \
        || echo "✗ $cena: saiu com $? (prazo ${W4_PRAZO:-900}s?)"
done
