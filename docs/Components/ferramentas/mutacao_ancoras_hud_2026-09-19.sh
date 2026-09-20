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
INSPECTOR=crates/ph2d-app-components/src/hud_inspector.rs
CENA=shells/desktop/src/hud_smoke.rs
IDS=crates/ph2d-panel-inspector/src/ids/inspector_hud.rs

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

# ⛔⛔ A guarda que separa o `Keep` do `Expand` — sem ela o `Keep` vira o `expand` do alvo, que e'
# o defeito que shipou por uma hora em 19/09.
bloco "o KEEP deixa de confinar" ph2d-hud o_keep_confina \
  "$LEI" 1 \
  '    if canvas.fit != Fit::Expand {' \
  '    if false {'

# O modo novo sai do selector ⇒ ele existe, tem lei, tem gates, e o artista nao lhe chega.
#
# ⚠️ A 1.ª redacção apontava ao gate de IDA-E-VOLTA da `ph2d-app-components` e **sobreviveu**: com
# `ALL = [Keep, Stretch, Stretch]` o `index()` devolve a posição do PRIMEIRO igual, logo a volta
# fecha para as três entradas. *Um `ALL` com duplicado passa toda régua de ida-e-volta* — quem o
# apanha é o `dedup` dos rótulos, e é esse o gate que esta mutação tem de correr.
bloco "o EXPAND sai do selector" ph2d-hud todo_modo_e_alcancavel \
  "$LEI" 1 \
  '    pub const ALL: [Self; 3] = [Self::Keep, Self::Stretch, Self::Expand];' \
  '    pub const ALL: [Self; 3] = [Self::Keep, Self::Stretch, Self::Stretch];'

# A pose do Expand deixa de ser a do Keep ⇒ a imagem dentro da caixa muda, que o alvo nao faz.
# ⚠️ As quebras de linha têm de ser REAIS (`$'…'`): num `'…'` do bash o `\n` chega ao python como
# dois caracteres, a âncora casa ZERO vezes, e o arnês aborta — que foi o que fez na 1.ª corrida.
bloco "a pose do EXPAND diverge do KEEP" ph2d-hud a_pose_do_expand_e_a_do_keep \
  "$LEI" 1 \
  $'        Fit::Expand => {\n            let s = fx.min(fy);\n            [s, s]\n        }' \
  '        Fit::Expand => [fx, fy],'

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

echo "=== O QUE A SECCAO DIZ (report do dono, 20/09) ==="

# A nota que diz ONDE estao as linhas do canvas desaparece ⇒ o artista ve' metade e conclui que a
# outra nao existe, que foi exactamente o report.
# ⚠️ A mutação é de UMA linha de propósito: a 1.ª redacção metia um `\n` num `'…'` do bash e o
# ficheiro saiu com um `\` literal ⇒ nao compilou, e o arnês leu «filtro vazio». Aqui ela faz a
# nota aparecer TAMBEM na raiz, que e' a metade NEGATIVA do gate.
bloco "a nota aparece tambem na RAIZ" ph2d-app-components a_seccao_diz_onde_estao \
  "$INSPECTOR" 1 \
  '    let canvas_parent = if canvas.is_some() {' \
  '    let canvas_parent = if false {'

# O roteiro deixa de nomear a HIERARQUIA ⇒ o passo volta a ser impossivel a quem tem o rotulo
# escolhido.
bloco "o roteiro nao diz onde escolher a raiz" ph2d-host-desktop o_roteiro_manda_escolher \
  "$CENA" 1 \
  'na HIERARQUIA (painel da esquerda) clique na linha' \
  'no painel clique na linha' '--test it'

echo "=== O QUE O ARTISTA VE (report do dono, 20/09) ==="

# ⛔⛔ O defeito EXACTO que ele reportou: o pintor passa tres rotulos e o array de ids tem DOIS ⇒
# o terceiro segmento nunca e' pintado. A regua antiga (contar rotulos no fonte) era cega a isto.
# ⚠️ A mutação apaga a ENTRADA **e** a aridade: apagar só a entrada deixa um `[NodeId; 3]` com
# duas, que é um erro de TIPO — e *uma mutação que não compila lê-se como sangrar*.
bloco "o segmentado perde um id" ph2d-panel-inspector o_segmentado_do_fit_tem_um_id \
  "$IDS" 1 \
  $'[NodeId; 3] = [\n    hash_node_id("insp_hud_fit_0"),\n    hash_node_id("insp_hud_fit_1"),\n    hash_node_id("insp_hud_fit_2"),\n];' \
  $'[NodeId; 2] = [\n    hash_node_id("insp_hud_fit_0"),\n    hash_node_id("insp_hud_fit_1"),\n];' '--test it'

# E dois segmentos com o MESMO id: a contagem fica certa e o clique de um acende o outro.
bloco "dois segmentos partilham id" ph2d-panel-inspector os_ids_do_segmentado_sao_distintos \
  "$IDS" 1 \
  '    hash_node_id("insp_hud_fit_2"),' \
  '    hash_node_id("insp_hud_fit_1"),' '--test it'

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
