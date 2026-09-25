#!/bin/bash
# ⭐ PROVA DE MUTAÇÃO da W5 do plano 28 (§12): o IMPACTO — a pausa no golpe (lei + o que a pede +
# a shell que a consome), o piscar (a lei, a marca derivada, a porta de desenho), o empurrão (a
# direcção do toque, quem o aceita, os dois canais — velocidade do solver e o canal próprio do
# mover), os números de dano, o Inspector (instantâneo, dreno, secção, sementes) e a cena.
# Corre-se por `bash scripts/ph2d-run.sh bash <este ficheiro>`.
#
# ⚠️ Controlo sobre o próprio filtro: uma corrida que casa ZERO testes é acusada como defeito do
# ARNÊS, nunca lida como «sobreviveu» (e uma mutação que não compila cai no mesmo braço). A agulha
# tem de casar EXACTAMENTE uma vez, senão o caso ABORTA.
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
  cp "$f" /tmp/mut_w5.bak
  if ! python3 - "$f" "$a" "$b" <<'P'
import sys; p,a,b=sys.argv[1:4]; s=open(p).read()
assert s.count(a)==1, ("agulha", a, s.count(a)); open(p,'w').write(s.replace(a,b))
P
  then echo "  $rot -> ABORTADO (agulha)"; cp /tmp/mut_w5.bak "$f"; touch "$f"; return; fi
  corre "$rot" "$@"
  cp /tmp/mut_w5.bak "$f"; touch "$f"
}

LEI=(-p ph2d-health --lib impacto)
APP=(-p ph2d-app-components --lib impacto)
PONTE_BARRA=(-p ph2d-app-components --lib health_bar)
FIS=(-p ph2d-physics-ecs --test it impacto)
VIS=(-p ph2d-entity-visibility --lib blink)
INSP=(-p ph2d-app-components --lib vida_inspector)
PAINEL=(-p ph2d-panel-inspector --test it a_seccao)
TDSHELL=(-p ph2d-host-desktop --bin ph2d-host-desktop inspector_topdown)
CENA=(-p ph2d-app-components --lib vida_impacto_smoke)
SHELL=(-p ph2d-host-desktop --test it o_impacto_esta_fiado)
H=crates/ph2d-health/src/impacto.rs
A=crates/ph2d-app-components/src/impacto.rs
F=crates/ph2d-physics-ecs/src/bridge/health.rs
I=crates/ph2d-app-components/src/vida_inspector.rs
V=crates/ph2d-app-components/src/vida_impacto_smoke.rs

echo "== a LEI da pausa e do piscar =="
corre CONTROLO "${LEI[@]}"
muta $H "            self.resta_s = s;" "            self.resta_s += s;" "H1 dois pedidos somam" "${LEI[@]}"
muta $H "        let retido = wall_dt.min(self.resta_s);" "        let retido = 0.0_f64.min(self.resta_s);" \
  "H2 a pausa nunca retém" "${LEI[@]}"
muta $H "    metade % 2 == 0" "    metade % 2 == 1" "H3 o piscar comeca escondido" "${LEI[@]}"

echo "== o que PEDE a pausa e os NUMEROS =="
corre CONTROLO "${APP[@]}"
muta $A "            pedido = f64::from(s);" "            pedido += f64::from(s);" "A1 os pedidos somam" "${APP[@]}"
muta $A "                w.get::<Damage>(ev.source).map_or(0.0, |d| d.hitstop_s)" \
  "                w.get::<Damage>(ev.target).map_or(0.0, |d| d.hitstop_s)" \
  "A2 o golpe pesa pelo alvo" "${APP[@]}"
muta $A "                .map_or(0.0, |h| h.death_hitstop_s)," "                .map_or(0.0, |_| 0.0)," \
  "A3 a morte nao pesa" "${APP[@]}"
muta $A "            self.pausa.consome(wall_dt)" "            wall_dt" "A4 a shell nao consome" "${APP[@]}"
muta $A "            if !h.numbers || !(h.numbers_size.is_finite() && h.numbers_size > 0.0) {" \
  "            if false || !(h.numbers_size.is_finite() && h.numbers_size > 0.0) {" \
  "A5 os numeros ignoram a caixa" "${APP[@]}"
muta $A "                nasceu: [pos[0], pos[1] + h.numbers_size]," "                nasceu: [pos[0], pos[1]]," \
  "A6 o numero nasce no centro" "${APP[@]}"
muta $A "        self.numeros.retain(|n| n.idade < VIDA_DO_NUMERO_S);" "" "A7 os numeros nunca saem" "${APP[@]}"
muta $A "        self.pausa.limpa();" "" "A8 rebobinar deixa a pausa" "${APP[@]}"
muta $A "    let n = quanto.round().max(1.0);" "    let n = quanto.round();" "A9 um golpe diz zero" "${APP[@]}"

echo "== a PONTE da barra envelhece e rebobina o impacto =="
corre CONTROLO "${PONTE_BARRA[@]}"
muta crates/ph2d-app-components/src/health_bar_bridge.rs "        self.impacto.anda(dt_s);" "" \
  "P1 o quadro nao envelhece os numeros" "${PONTE_BARRA[@]}"
muta crates/ph2d-app-components/src/health_bar_bridge.rs \
  "        let n = self.rastos.len() + self.impacto.rewind();" "        let n = self.rastos.len();" \
  "P2 o renascer esquece o impacto" "${PONTE_BARRA[@]}"

echo "== a FISICA do empurrao e do piscar =="
corre CONTROLO "${FIS[@]}"
muta $F "            self.empurra(corpo, dv);" "" "F1 ninguem e' empurrado" "${FIS[@]}"
muta $F "                toques.push(((a, a), (b, b), Some(amostra.normal)));" \
  "                toques.push(((a, a), (b, b), None));" "F2 a normal do toque perdida" "${FIS[@]}"
muta $F "    let aceita = if vida.knockback_taken.is_finite() {" "    let aceita = if false {" \
  "F3 quem leva nao decide" "${FIS[@]}"
muta $F "    if !entrou {
        return None;
    }" "" "F4 uma esquiva empurra" "${FIS[@]}"
muta $F "        (d[1] * dano.knockback + dano.knockback_lift) * aceita," \
  "        (d[1] * dano.knockback) * aceita," "F5 o empurrao para cima perdido" "${FIS[@]}"
muta $F "            st.knockback[0] += dv[0];" "" "F6 o mover so' e' empurrado em y" "${FIS[@]}"
muta crates/ph2d-physics-ecs/src/bridge/topdown.rs \
  "            let mut passo = slide::first_step(movimento, dt, law.max_slides);" \
  "            let mut passo = slide::first_step(v, dt, law.max_slides);" \
  "F7 o canal do mover nunca chega ao passo" "${FIS[@]}"
muta $F "                if comecou && let Some(dv) = empurrao(dano, &h, toque.direccao, &fs) {" \
  "                if let Some(dv) = empurrao(dano, &h, toque.direccao, &fs) {" \
  "F8 empurra em todo tique do toque" "${FIS[@]}"
muta $F "                    em.insert(ph2d_ecs::BlinkOff);" "" "F9 o piscar nunca esconde" "${FIS[@]}"
muta $F "            w.entity_mut(e).remove::<ph2d_ecs::BlinkOff>();" "" \
  "F10 quem perde a vida fica escondido" "${FIS[@]}"
muta $F "                    st.vida.invencivel(&h.config())," "                    true," \
  "F11 pisca fora da invencibilidade" "${FIS[@]}"
muta crates/ph2d-topdown/src/intent.rs "    aproximar(empurrao, [0.0, 0.0], taxa, dt)" "    empurrao" \
  "F12 o empurrao do mover nunca se gasta" "${FIS[@]}"

echo "== a PORTA de desenho =="
corre CONTROLO "${VIS[@]}"
muta crates/ph2d-entity-visibility/src/off_canvas.rs \
  "        && sim.get::<ph2d_ecs::BlinkOff>(entity).is_none()" "" "V1 a marca nao esconde" "${VIS[@]}"

echo "== o INSTANTANEO e o DRENO do Inspector =="
corre CONTROLO "${INSP[@]}"
muta $I "        E::KnockbackTaken(v) => h.knockback_taken = positivo(*v)," \
  "        E::KnockbackTaken(v) => h.knockback_taken = *v," "I1 aceitar negativo" "${INSP[@]}"
muta $I "        E::Numbers(b) => h.numbers = *b," "        E::Numbers(_) => {}" "I2 a caixa dos numeros morta" "${INSP[@]}"
muta $I "        E::Knockback(v) => d.knockback = if v.is_finite() { *v } else { 0.0 }," \
  "        E::Knockback(v) => d.knockback = v.max(0.0)," "I3 o empurrao negativo preso a zero" "${INSP[@]}"
muta $I "        E::NumbersColor(c) => h.numbers_color = cor(*c)," "        E::NumbersColor(c) => h.numbers_color = *c," \
  "I4 a cor dos numeros sem cerca" "${INSP[@]}"

echo "== a SECCAO do painel e as SEMENTES =="
corre CONTROLO "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/sync_sections.rs \
  "    crate::sync_topdown::sync(host, inspector_state, entity_changed);" "" \
  "N1 o mover nao e' semeado" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/event_topdown.rs \
  "            crate::ids::INSP_TD_KNOCKBACK_RECOVERY => TopDownFieldEdit::KnockbackRecovery(f)," \
  "            crate::ids::INSP_TD_KNOCKBACK_RECOVERY => TopDownFieldEdit::Deceleration(f)," \
  "N2 a recuperacao escreve na travagem" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/populate_vida.rs "        .chain([ids::INSP_VIDA_NUMBERS_COLOR])" \
  "        .chain([ids::INSP_BARRA_CORES[0]])" "N3 a amostra dos numeros morta" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/sync_vida.rs "            ids::INSP_VIDA_NUMBERS_COLOR,
            h.numbers_color,
            E::NumbersColor," "            ids::INSP_VIDA_NUMBERS_COLOR,
            h.numbers_color,
            E::BarFill," "N4 a cor dos numeros escreve na barra" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/sections/vida_impacto.rs "    if h.invincible_s > 0.0 {" "    if true {" \
  "N5 o piscar oferecido sem invencibilidade" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/event_vida.rs \
  "        ids::INSP_DANO_KNOCKBACK => E::Knockback(f)," "        ids::INSP_DANO_KNOCKBACK => E::KnockbackLift(f)," \
  "N6 o empurrao escreve no para-cima" "${PAINEL[@]}"

echo "== o MOVER na shell =="
corre CONTROLO "${TDSHELL[@]}"
muta shells/desktop/src/render_loop/inspector_topdown.rs "        knockback_recovery: law.knockback_recovery," \
  "        knockback_recovery: 24.0," "T1 o instantaneo mostra o de fabrica" "${TDSHELL[@]}"
muta shells/desktop/src/render_loop/inspector_topdown.rs \
  "        TopDownFieldEdit::KnockbackRecovery(v) => law.knockback_recovery = v.max(0.0)," \
  "        TopDownFieldEdit::KnockbackRecovery(v) => law.knockback_recovery = *v," \
  "T2 a recuperacao negativa" "${TDSHELL[@]}"

echo "== a CENA =="
corre CONTROLO "${CENA[@]}"
muta $V "        d.knockback = EMPURRAO;" "" "C1 a bala da cena nao empurra" "${CENA[@]}"
muta $V "pub const ARRASTO: f32 = 3.0;" "pub const ARRASTO: f32 = 0.0;" "C2 sem arrasto" "${CENA[@]}"

echo "== a SHELL ouve e retem =="
corre CONTROLO "${SHELL[@]}"
muta shells/desktop/src/render_loop/fase_physics_step.rs \
  "        health_bars.impacto.ouve(sim, physics.health_events());" "" "S1 os golpes nao sao ouvidos" "${SHELL[@]}"
muta shells/desktop/src/render_loop/fase_fixed_step_clocks.rs \
  "        let wall_dt = health_bars.impacto.retem(wall_dt, a_correr);" "" "S2 a pausa nao retem" "${SHELL[@]}"

corre RESTAURADO "${LEI[@]}"
corre RESTAURADO "${APP[@]}"
corre RESTAURADO "${PONTE_BARRA[@]}"
corre RESTAURADO "${FIS[@]}"
corre RESTAURADO "${VIS[@]}"
corre RESTAURADO "${INSP[@]}"
corre RESTAURADO "${PAINEL[@]}"
corre RESTAURADO "${TDSHELL[@]}"
corre RESTAURADO "${CENA[@]}"
corre RESTAURADO "${SHELL[@]}"
