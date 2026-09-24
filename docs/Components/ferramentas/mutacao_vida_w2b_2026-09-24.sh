#!/bin/bash
# ⭐ PROVA DE MUTAÇÃO da W2b do plano 28 (§10): os verbos `Damage`/`Heal` da tabela, a fita da vida,
# o `On Heal`, a cena do veneno e a ligação da shell. Corre-se por
# `bash scripts/ph2d-run.sh bash <este ficheiro>`.
#
# Resultado de 2026-09-24: ver o §10.6 do plano (a 1.ª corrida deixou F1, F2 e F4 VIVAS — a
# fixtura pedia sobre o passo do anel, e o golpe de zero não era olhado; o §10.6 conta a cura).
# ⚠️ Controlo sobre o próprio filtro: uma corrida que casa ZERO testes é acusada como defeito do
# ARNÊS, nunca lida como «sobreviveu» (e uma mutação que não compila cai no mesmo braço). A agulha
# tem de casar EXACTAMENTE uma vez, senão o caso ABORTA (uma mutação que não entra lê-se como uma
# que sobreviveu).
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
  cp "$f" /tmp/mut_w2b.bak
  if ! python3 - "$f" "$a" "$b" <<'P'
import sys; p,a,b=sys.argv[1:4]; s=open(p).read()
assert s.count(a)==1, ("agulha", a, s.count(a)); open(p,'w').write(s.replace(a,b))
P
  then echo "  $rot -> ABORTADO (agulha)"; cp /tmp/mut_w2b.bak "$f"; touch "$f"; return; fi
  corre "$rot" "$@"
  cp /tmp/mut_w2b.bak "$f"; touch "$f"
}

FIS=(-p ph2d-physics-ecs --test it vida_pedida)
TAB=(-p ph2d-app-components --lib signal_actions_bridge)
CENA=(-p ph2d-app-components --lib vida_smoke)
ECS=(-p ph2d-ecs --lib so_estes_verbos_leem_o_argumento)
SHELL=(-p ph2d-host-desktop --test it os_pedidos_de_vida)
H=crates/ph2d-physics-ecs/src/bridge/health.rs
B=crates/ph2d-app-components/src/signal_actions_bridge.rs
V=crates/ph2d-app-components/src/vida_smoke.rs

echo "== a PONTE da vida: a fita, a recusa, a cura, o On Heal =="
corre CONTROLO "${FIS[@]}"
muta $H "self.fita_da_vida.get(&tick).cloned().unwrap_or_default()" "Vec::new()" \
  "F1 o replay nao le a fita" "${FIS[@]}"
muta $H "self.fita_da_vida.insert(tick, fila.clone());" "let _ = &fila;" \
  "F2 a fita nao grava" "${FIS[@]}"
muta $H "self.fita_da_vida.remove(&tick);" "" \
  "F7 o tique vivo nao grava por cima" "${FIS[@]}"
muta $H "PedidoDeVida::Cura(quanto) => st.vida.cura(&cfg, Regras::CASA, quanto)," \
  "PedidoDeVida::Cura(_) => {}" "F3 a cura nao cura" "${FIS[@]}"
muta $H "x.is_finite() && x > 0.0," "x.is_finite()," "F4 a ponte aceita <= 0" "${FIS[@]}"
muta $H "    if depois.pontos > antes.pontos {" "    if false {" \
  "F5 sem facto Healed" "${FIS[@]}"
muta crates/ph2d-physics-ecs/src/bridge/signals.rs \
  "HealthEventKind::Healed { .. } => &vida.on_heal," "HealthEventKind::Healed { .. } => continue," \
  "F6 sem sinal On Heal" "${FIS[@]}"

echo "== a TABELA: o verbo anuncia o pedido certo, e recusa o que nao sabe ler =="
corre CONTROLO "${TAB[@]}"
muta $B "SignalVerb::Heal => ph2d_physics_ecs::PedidoDeVida::Cura(quanto)," \
  "SignalVerb::Heal => ph2d_physics_ecs::PedidoDeVida::Dano(quanto)," \
  "T1 Heal vira Dano" "${TAB[@]}"
muta $B "    if !(quanto.is_finite() && quanto > 0.0) {" "    if !quanto.is_finite() {" \
  "T2 a tabela aceita <= 0" "${TAB[@]}"
muta $B "    sim.world().get::<ph2d_physics_ecs::Health>(fx.target)?;" "" \
  "T3 sem exigir Health" "${TAB[@]}"
corre CONTROLO "${ECS[@]}"
muta crates/ph2d-ecs/src/signal_actions.rs "                | SignalVerb::Damage
                | SignalVerb::Heal
        )" "        )" "E1 Damage/Heal nao leem o argumento" "${ECS[@]}"

echo "== a CENA do veneno =="
corre CONTROLO "${CENA[@]}"
muta $V "SignalFrom::Myself," "SignalFrom::Anyone," "C1 a cura ouve qualquer um" "${CENA[@]}"
muta $V "                on_heal: CUROU.to_owned(),
" "" "C2 o roxo nao grita ao curar" "${CENA[@]}"
muta $V "    if nome == ALVOS[ROXO].0 {" "    if false {" "C3 o roxo sem tabela nem relogio" \
  "${CENA[@]}"
muta $V "                autostart: false," "                autostart: true," \
  "C4 o relogio arranca sozinho (os avisos da foto)" "${CENA[@]}"

echo "== a SHELL entrega os pedidos =="
corre CONTROLO "${SHELL[@]}"
muta shells/desktop/src/render_loop/fase_tabela_de_accoes.rs \
  "            physics.pede_vida(alvo, pedido);" "            let _ = (alvo, pedido, &physics);" \
  "S1 a shell nao entrega" "${SHELL[@]}"

corre RESTAURADO "${FIS[@]}"
corre RESTAURADO "${TAB[@]}"
corre RESTAURADO "${CENA[@]}"
corre RESTAURADO "${ECS[@]}"
corre RESTAURADO "${SHELL[@]}"
