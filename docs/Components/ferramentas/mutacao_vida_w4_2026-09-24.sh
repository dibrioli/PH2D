#!/bin/bash
# ⭐ PROVA DE MUTAÇÃO da W4 do plano 28 (§11): a BARRA DE VIDA — a lei do rasto (`ph2d-hud`), a
# ponte que a corre e a desenha, o instantâneo e o dreno do Inspector, a secção do painel, a cena e
# a ligação da shell. Corre-se por `bash scripts/ph2d-run.sh bash <este ficheiro>`.
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
  cp "$f" /tmp/mut_w4.bak
  if ! python3 - "$f" "$a" "$b" <<'P'
import sys; p,a,b=sys.argv[1:4]; s=open(p).read()
assert s.count(a)==1, ("agulha", a, s.count(a)); open(p,'w').write(s.replace(a,b))
P
  then echo "  $rot -> ABORTADO (agulha)"; cp /tmp/mut_w4.bak "$f"; touch "$f"; return; fi
  corre "$rot" "$@"
  cp /tmp/mut_w4.bak "$f"; touch "$f"
}

LEI=(-p ph2d-hud --lib barra)
PONTE=(-p ph2d-app-components --lib health_bar)
INSP=(-p ph2d-app-components --lib vida_inspector)
CENA=(-p ph2d-app-components --lib vida_smoke)
PAINEL=(-p ph2d-panel-inspector --test it a_seccao_barra_de_vida)
SHELL=(-p ph2d-host-desktop --test it as_barras_de_vida)
L=crates/ph2d-hud/src/barra.rs
B=crates/ph2d-app-components/src/health_bar_bridge.rs
I=crates/ph2d-app-components/src/vida_inspector.rs
V=crates/ph2d-app-components/src/vida_smoke.rs

echo "== a LEI do rasto =="
corre CONTROLO "${LEI[@]}"
muta $L "    if agora < r.ultimo {" "    if false {" "L1 um golpe nao recomeca a espera" "${LEI[@]}"
# ⚠️ A L2 original («a cura nao puxa o rasto para cima», o ramo de retorno cedo) SOBREVIVEU e o
# ramo foi APAGADO: o `max(agora)` do fim ja' o fazia. A L3 mede a unica metade viva.
muta $L "        valor: (r.valor - desce).max(agora)," "        valor: r.valor - desce," \
  "L3 o rasto desce abaixo da vida" "${LEI[@]}"
muta $L "    [(0.0, w), faixa(rasto.max(agora)), faixa(agora)]" \
  "    [(0.0, w), faixa(agora), faixa(rasto)]" "L4 as faixas trocadas" "${LEI[@]}"

echo "== a PONTE =="
corre CONTROLO "${PONTE[@]}"
muta $B "        return Some(dono);" "        return None;" "B1 alvo vazio nao e' o proprio" "${PONTE[@]}"
muta $B "                bevy_ecs::query::Without<MasterPiece>," "" "B2 o molde desenha" "${PONTE[@]}"
muta $B "                bevy_ecs::query::Without<ph2d_ecs::MasterRoot>," "" \
  "B3 a raiz do molde desenha antes do passe" "${PONTE[@]}"
muta $B "                && w.get::<MasterPiece>(*e).is_none()" "" \
  "B4 o nome acha um molde" "${PONTE[@]}"
muta $B "                .is_some_and(|v| v.hidden)" "                .is_some_and(|_| false)" \
  "B5 o escondido desenha" "${PONTE[@]}"
muta $B "            if cfg.hide_when_full && f >= 1.0 && r.valor >= 1.0 {" "            if false {" \
  "B6 hide_when_full ignorado" "${PONTE[@]}"
muta $B "    let s = det.abs().sqrt();" "    let s = 1.0_f32 + 0.0 * det;" "B7 a escala ignorada" "${PONTE[@]}"
muta $B "        self.rastos.clear();" "" "B8 rebobinar nao repoe" "${PONTE[@]}"
muta $B "        .map_or(h.start, |n| n.pontos as f32);" "        .map_or(h.start, |_| h.start);" \
  "B9 a vida de agora ignorada" "${PONTE[@]}"
muta $B "    let max = if h.max > 0.0 { h.max } else { h.start };" "    let max = h.max;" \
  "B10 sem maximo nao mede contra o inicio" "${PONTE[@]}"

echo "== o INSTANTANEO e o DRENO do Inspector =="
corre CONTROLO "${INSP[@]}"
muta $I "    if health.is_none() && damage.is_none() && bar.is_none() {" \
  "    if health.is_none() && damage.is_none() {" "I1 um placar so' com barra sem seccao" "${INSP[@]}"
muta $I "        E::BarOffsetY(v) => b.offset_y = livre(*v)," "        E::BarOffsetY(v) => b.offset_y = positivo(*v)," \
  "I2 o deslocamento negativo preso a zero" "${INSP[@]}"
muta $I "        E::BarHideWhenFull(on) => b.hide_when_full = *on," "        E::BarHideWhenFull(_) => {}" \
  "I3 a caixa nao chega a barra" "${INSP[@]}"
muta $I "        E::BarFill(c) => b.fill = cor(*c)," "        E::BarFill(c) => b.fill = *c," \
  "I4 a cor sem cerca" "${INSP[@]}"
muta crates/ph2d-editor-core/src/vida_edits.rs \
  "        if !self.has_body && (self.health.is_some() || self.damage.is_some()) {" \
  "        if !self.has_body {" "I5 a barra sozinha queixa-se de corpo" "${INSP[@]}"

echo "== a SECCAO do painel =="
corre CONTROLO "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/populate_vida.rs "        store.register_picker_swatch(id);" "" \
  "P1 a amostra nao abre o selector" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/sync_vida.rs "        (trail, b.trail, E::BarTrail)," \
  "        (trail, b.trail, E::BarBack)," "P2 duas amostras trocadas" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/sync_vida.rs "    cores(host, &info);" "" \
  "P3 a cor escolhida morre no painel" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/event_vida.rs "        ids::INSP_BARRA_OFFSET_Y => E::BarOffsetY(f)," "" \
  "P4 um numero da barra sem braco" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/event_vida.rs \
  "        ids::INSP_BARRA_HIDE_FULL => E::BarHideWhenFull(!b?.hide_when_full)," \
  "        ids::INSP_BARRA_HIDE_FULL => E::BarHideWhenFull(b?.hide_when_full)," \
  "P5 a caixa nao inverte" "${PAINEL[@]}"
muta crates/ph2d-panel-inspector/src/sync_vida.rs "            (ids::INSP_BARRA_WIDTH, f64::from(b.width))," "" \
  "P6 a largura nao e' semeada" "${PAINEL[@]}"

echo "== a CENA =="
corre CONTROLO "${CENA[@]}"
muta $V "pub const BARRA_Y: f32 = 0.57;" "pub const BARRA_Y: f32 = 0.75;" "C1 a barra de fabrica toca a fila" "${CENA[@]}"
muta $V 'pub const ALVO_DO_PLACAR: &str = "Alvo de 3 tiros (1)";' \
  'pub const ALVO_DO_PLACAR: &str = "Alvo de 3 tiros";' "C2 o placar nomeia o molde" "${CENA[@]}"
muta $V "                height: BARRA_H,
" "" "C3 a receita com a altura de fabrica" "${CENA[@]}"

echo "== a SHELL liga, desenha e rebobina =="
corre CONTROLO "${SHELL[@]}"
muta shells/desktop/src/render_loop/motores_do_quadro.rs "    barras(sim, health_bars, relogio);" "" \
  "S1 o quadro nao corre as barras" "${SHELL[@]}"
muta shells/desktop/src/render_loop/present.rs "            && barras.is_empty();" ";" \
  "S2 as barras fora do atalho" "${SHELL[@]}"
muta shells/desktop/src/render_loop/fase_fabrica_e_morte.rs "    repostos += health_bars.rewind();" "" \
  "S3 o renascimento nao repoe os rastos" "${SHELL[@]}"

corre RESTAURADO "${LEI[@]}"
corre RESTAURADO "${PONTE[@]}"
corre RESTAURADO "${INSP[@]}"
corre RESTAURADO "${CENA[@]}"
corre RESTAURADO "${PAINEL[@]}"
corre RESTAURADO "${SHELL[@]}"
