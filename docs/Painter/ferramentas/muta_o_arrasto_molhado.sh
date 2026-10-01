#!/usr/bin/env bash
# Prova de mutação do Smudge e do Rewet sobre a tinta MOLHADA (doc 44 §3, 2026-09-29). Corre-se pela porta da linha:
#   bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/muta_o_arrasto_molhado.sh
# MUTA_SO_ANCORAS=1 = só o pré-voo (cada âncora casa 1×); MUTA_FILTRO="M3 M5" = só essas.
# Controlos: âncora casa 1× (pré-voo), corrida limpa
# VERDE, e cada corrida conta os testes que CORRERAM (0 ⇒ aborta, nunca «sobreviveu»).
set -u
cd "$(git rev-parse --show-toplevel)"
SP=$(mktemp -d)
ACC=crates/ph2d-tool-painter/src/tool/paint/watercolor_accum.rs
MIS=crates/ph2d-tool-painter/src/tool/paint/watercolor_mistura.rs
SMU=crates/ph2d-tool-painter/src/tool/paint/watercolor_smudge.rs
SMEAR_RS=crates/ph2d-painter-brush/src/smear.rs
ARR=crates/ph2d-tool-painter/src/tool/paint/watercolor_mistura_arrasto.rs
COR=crates/ph2d-tool-painter/src/tool/paint/watercolor_accum_cor.rs
AGU=crates/ph2d-tool-painter/src/tool/paint/watercolor_mistura_agua.rs
corre() { # imprime "rc total"
  local tot=0 rc=0 out
  for f in watercolor_arrasto_molhado watercolor_mistura_molhada junction_is_soft watercolor_mistura_agua; do
    out=$(cargo test -q -p ph2d-tool-painter --profile ci-test --lib "$f" 2>&1); r=$?
    [ $r -ne 0 ] && rc=1
    n=$(printf '%s\n' "$out" | grep -oE '[0-9]+ passed; [0-9]+ failed' | awk '{s+=$1+$3} END{print s+0}')
    tot=$((tot+n))
    if printf '%s\n' "$out" | grep -q "error\[E"; then echo "NAO-COMPILA"; return; fi
  done
  out=$(cargo test -q -p ph2d-painter-brush --profile ci-test --lib smear 2>&1); r=$?
  [ $r -ne 0 ] && rc=1
  n=$(printf '%s\n' "$out" | grep -oE '[0-9]+ passed; [0-9]+ failed' | awk '{s+=$1+$3} END{print s+0}')
  tot=$((tot+n))
  echo "$rc $tot"
}
M=(
"M1|$ARR|            &mut self.antes,|            &mut self.antes.clone(),"
"M2|$ARR|ph2d_painter_brush::smear_dab_premultiplicado(|ph2d_painter_brush::smear_dab("
"M3|$MIS|    if !planos.capturado[idx / 4] {|    if planos.proprio[idx + 3] == 0 {"
"M5|$ARR|        self.captura(buf, fw, [x0, y0, x1, y1]);|"
"M6|crates/ph2d-tool-painter/src/tool/paint/stroke_lifecycle.rs|self.paint.wet_mistura.ha_tinta_da_sessao = wet_session;|self.paint.wet_mistura.ha_tinta_da_sessao = true;"
"M7|$COR|            pigment.max(arrasto.unwrap_or(0.0)),|            pigment,"
"M8|$SMEAR_RS|            dst[c] = ((pd + (ps - pd) * w) / na).round().clamp(0.0, 255.0) as u8;|            dst[c] = (pd + (ps - pd) * w).round().clamp(0.0, 255.0) as u8;"
"M9|$MIS|        self.capturado.iter_mut()|        self.capturado.iter_mut().take(0)"
"M10|$SMEAR_RS|        mistura_premultiplicada,|        mistura_recta,"
"M11|$COR|let prio_do_deposito = if pigment { 1.0 } else { prio };|let prio_do_deposito = 1.0;"
"M12|$ARR|        let (Some(keep), Some(v)) = (guarda, antes_do_arrasto) else {|        let (Some(keep), Some(v)) = (None::<fn(usize) -> f32>, antes_do_arrasto) else {"
"M13|$COR|if gated { Some(|if false { Some("
"M14|$ARR|            let pd = f32::from(dst[c]) * da;|            let pd = f32::from(dst[c]);"
"R1|$MIS|.then_some(agua)|.then_some(0.0)"
"R2|$MIS|    let antes = parceiro.unwrap_or([|    let antes = parceiro.filter(|_| false).unwrap_or(["
"R3|$COR|let raio = super::watercolor_mistura_agua::raio_da_agua(|let raio = 0 * super::watercolor_mistura_agua::raio_da_agua("
"R4|$MIS|t, pelo_botao, pela_agua);|t, pela_agua, pelo_botao);"
"R5|$AGU|            let pd = f32::from(antes[c]) * aa;|            let pd = f32::from(antes[c]);"
"R6|$AGU|                let px = if self.capturado[g] {|                let px = if false {"
"R7|$MIS|    let mistura = pelo_botao.max(pela_agua);|    let mistura = pelo_botao;"
)
aplica() { python3 - "$1" "$2" "$3" <<'PY'
import sys
p,a,b=sys.argv[1],sys.argv[2].replace('\\&','&').replace('\\|','|'),sys.argv[3].replace('\\&','&').replace('\\|','|')
t=open(p).read(); n=t.count(a)
if n!=1: print(f"ANCORA {n}"); sys.exit(3)
open(p,"w").write(t.replace(a,b))
PY
}
if [ "${MUTA_SO_ANCORAS:-0}" = 1 ]; then
  ok=0; for m in "${M[@]}"; do IFS='|' read -r nome f a b <<<"$m"; a=${a//\\&/&}; a=${a//\\|/|}; n=$(python3 -c "import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))" "$f" "$a"); echo "$nome $n"; [ "$n" = 1 ] && ok=$((ok+1)); done; echo "PRE-VOO $ok de ${#M[@]}"; exit 0; fi
read rc tot < <(corre); echo "LIMPA rc=$rc testes=$tot"; [ "$rc" = 0 ] && [ "$tot" -gt 0 ] || { echo "ABORTA: limpa não verde"; exit 2; }
S=0; V=0
for m in "${M[@]}"; do
  IFS='|' read -r nome f a b <<<"$m"
  [ -n "${MUTA_FILTRO:-}" ] && [[ ! " $MUTA_FILTRO " =~ " $nome " ]] && continue
  cp "$f" "$SP/bak.rs"
  if ! aplica "$f" "$a" "$b"; then echo "$nome ABORTA âncora"; cp "$SP/bak.rs" "$f"; touch "$f"; continue; fi
  res=$(corre)
  cp "$SP/bak.rs" "$f"; touch "$f"
  set -- $res
  if [ "$1" = NAO-COMPILA ]; then echo "$nome NAO-COMPILA"; elif [ "$2" = 0 ]; then echo "$nome ABORTA zero testes"; elif [ "$1" = 1 ]; then echo "$nome SANGRA ($2 testes)"; S=$((S+1)); else echo "$nome SOBREVIVEU ($2 testes)"; V=$((V+1)); fi
done
echo "SANGRAM $S · SOBREVIVEM $V"
[ "$V" = 0 ]
