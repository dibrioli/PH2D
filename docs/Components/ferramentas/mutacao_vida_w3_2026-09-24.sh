#!/bin/bash
# ⭐ PROVA DE MUTAÇÃO da W3 do plano 28 (§9): as 2 mutações da CURA (o registo) + as 5 do gate da
# secção HEALTH/DAMAGE. Corre-se por `bash scripts/ph2d-run.sh bash <este ficheiro>`.
#
# Resultado de 2026-09-24: CONTROLO verde · as 7 sangram · RESTAURADO verde.
# ⚠️ Controlo sobre o próprio filtro: uma corrida que casa ZERO testes é acusada como defeito do
# ARNÊS, nunca lida como «sobreviveu» (e uma mutação que não compila cai no mesmo braço).
set -u
cd "$(dirname "$0")/../../.."

corre() { # rótulo cargo-args...
  local rot=$1; shift
  out=$(cargo test "$@" 2>&1)
  r=$(echo "$out" | grep -oE "test result: [a-zA-Z]+\. [0-9]+ passed; [0-9]+ failed" | head -1)
  n=$(echo "$r" | awk '{print $4+$6}')
  if [ "${n:-0}" -eq 0 ]; then
    echo "  $rot -> ARNES: zero testes (filtro ou compilacao)"
    echo "$out" | grep -E "^error" | head -3
    return
  fi
  echo "  $rot -> $r"
}
muta() { # ficheiro agulha substituto rótulo cargo-args...
  local f=$1 a=$2 b=$3 rot=$4; shift 4
  cp "$f" /tmp/mut_w3.bak
  if ! python3 - "$f" "$a" "$b" <<'P'
import sys; p,a,b=sys.argv[1:4]; s=open(p).read()
assert s.count(a)==1, ("agulha", a, s.count(a)); open(p,'w').write(s.replace(a,b))
P
  then echo "  $rot -> ABORTADO (agulha)"; cp /tmp/mut_w3.bak "$f"; return; fi
  corre "$rot" "$@"
  cp /tmp/mut_w3.bak "$f"; touch "$f"
}

CENA=(-p ph2d-app-components --lib vida_smoke)
PAINEL=(-p ph2d-panel-inspector --test it a_seccao_vida)

echo "== a CURA: sem o registo, a cópia da fábrica nasce sem vida/dano =="
corre CONTROLO "${CENA[@]}"
muta crates/ph2d-physics-ecs/src/lib.rs 'reg.register_default::<Health>("ph2d::physics::Health");' '' \
  "C1 sem Health registada" "${CENA[@]}"
muta crates/ph2d-physics-ecs/src/lib.rs 'reg.register_default::<Damage>("ph2d::physics::Damage");' '' \
  "C2 sem Damage registado" "${CENA[@]}"

echo "== o gate da secção =="
corre CONTROLO "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/sync_sections.rs \
  "    crate::sync_vida::sync(host, inspector_state, entity_changed);" "" \
  "M1 sem sync_vida" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/sections/vida.rs "    if h.regen > 0.0 {" "    if true {" \
  "M2 atraso sempre pintado" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/populate_vida.rs "    for id in TEXTOS {" \
  "    for id in TEXTOS.into_iter().filter(|i| *i != ids::INSP_VIDA_ON_HEAL) {" \
  "M3 populate sem on_heal" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/event_vida.rs "=> E::Vanish(!d?.vanish)" "=> E::Vanish(d?.vanish)" \
  "M4 caixa nao inverte" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/event_vida.rs "ids::INSP_VIDA_ON_HEAL => E::OnHeal(t)" \
  "ids::INSP_VIDA_ON_HEAL => E::OnDamage(t)" "M5 variante trocada" "${PAINEL[@]}"

corre RESTAURADO "${PAINEL[@]}"
corre RESTAURADO "${CENA[@]}"
