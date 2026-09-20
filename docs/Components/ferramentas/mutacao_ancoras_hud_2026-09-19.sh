#!/usr/bin/env bash
# Provas de mutação das ÂNCORAS DO HUD (TOP-20 #20) — a caixa EFECTIVA do canvas.
#
# Arnês IDÊNTICO ao das waves anteriores desta linha — controlo sobre o próprio FILTRO (um filtro
# que casa ZERO testes imprime `ok` e lê-se como «sobreviveu») e `muta` a ABORTAR quando a âncora
# não aparece o número esperado de vezes.
#
# uso:  bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_ancoras_hud_2026-09-19.sh
set -u
cd "$(dirname "$0")/../../.." || exit 1

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1").$2"; }
restaura() { cp "$TMP/$(basename "$1").$2" "$1"; touch "$1"; }

muta() { # ficheiro vezes antigo novo
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import sys
p, n, old, new = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
s = open(p).read()
c = s.count(old)
if c != n:
    sys.exit(f"  ⛔ ANCORA: {old!r} aparece {c} vezes em {p} (esperado {n})")
open(p, "w").write(s.replace(old, new))
PY
}

prova() { # nome crate filtro [alvos]
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  # shellcheck disable=SC2086
  out=$(timeout 900 cargo test -p "$2" ${4:---all-targets} -- "$3" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto mutado"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

bloco() { # nome crate filtro ficheiro vezes antigo novo [alvos]
  guarda "$4" b
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3" "${8:-}"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}

LEI=crates/ph2d-hud/src/lib.rs
PONTE=crates/ph2d-app-components/src/hud_bridge.rs
PASSE=shells/desktop/src/layout_live_anchors.rs
SELECAO=crates/ph2d-app-components/src/hud_anchors.rs
FASE=shells/desktop/src/render_loop/fase_hud.rs
RECOOK=shells/desktop/src/render_loop/fase_vector_layout_recook.rs

echo "=== A LEI: a caixa efectiva (ph2d-hud) ==="

bloco "a banda NAO atravessa a escala" ph2d-hud a_banda_atravessa_a_escala \
  "$LEI" 1 \
  'let local = |banda: f32, escala: f32| if escala > 0.0 { banda / escala } else { 0.0 };' \
  'let local = |banda: f32, _escala: f32| banda;'

bloco "a efectiva vira a de REFERENCIA" ph2d-hud o_canto_da_efectiva_pousa_na_borda \
  "$LEI" 1 \
  'let hx = canvas.ref_w / 2.0 + local(b[0], p.scale[0]);' \
  'let hx = canvas.ref_w / 2.0;'

bloco "a guarda da escala ZERO sai" ph2d-hud uma_vista_degenerada \
  "$LEI" 1 \
  'if escala > 0.0 { banda / escala } else { 0.0 }' \
  'banda / escala'

echo "=== A PONTE: a moldura que o canvas oferece ==="

bloco "a escala da moldura vira 1" ph2d-host-desktop com_a_raiz_a_escalar \
  "$PONTE" 1 \
  '[f64::from(p.scale[0]), f64::from(p.scale[1])],' \
  '[1.0, 1.0],' --bins

echo "=== O PASSE: quem ancora quem ==="

bloco "o canvas deixa de ancorar" ph2d-host-desktop um_filho_preso_a_direita \
  "$PASSE" 1 \
  '            self.anchor_kid(scene, sim, xforms, live, a.kid, m);' \
  '            let _ = (a, m);' --bins

# ⚠️ A 1.ª redacção desta mutação apagava a linha da cerca e **sobreviveu**: o chamador voltava a
# perguntar pelo `UiCanvas` e NEUTRALIZAVA-a. A cura foi tirar a 2.ª resposta (a porta devolve o
# PAR), e a mutação passa a ser a que de facto expressa o defeito — ancorar contra um canvas que
# não existe.
bloco "a cerca do PAI sai" ph2d-app-components um_filho_de_moldura_nao_entra \
  "$SELECAO" 1 \
  '    let cfg = *w.get::<ph2d_ecs::UiCanvas>(parent)?;' \
  '    let cfg = w.get::<ph2d_ecs::UiCanvas>(parent).copied().unwrap_or_default();'

echo "=== A ORDEM no quadro ==="

bloco "a fase do HUD nao guarda a vista" ph2d-host-desktop o_hud_conduz_a_raiz \
  "$FASE" 1 \
  '        self.components.hud.vista = vista;' \
  '' '--test it'

bloco "o recook nao recebe a vista" ph2d-host-desktop o_hud_conduz_a_raiz \
  "$RECOOK" 1 \
  '        self.layout_live.vista = self.components.hud.vista;' \
  '' '--test it'

echo
echo "════════════════════════════════════════"
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL sangraram"
else
  echo "⛔ $FALHAS de $TOTAL NAO sangraram"
fi
exit "$FALHAS"
