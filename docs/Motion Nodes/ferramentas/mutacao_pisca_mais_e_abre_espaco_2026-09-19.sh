#!/usr/bin/env bash
# Provas de mutacao das duas ordens do dono (2026-09-19, depois do 2.o smoke OK):
#
#   (F) «faca ambos piscarem mais vezes depois da troca»
#   (G) «se o espaco onde o no entrou for muito apertado, crie espaco»
#
# ⚠️ `touch` no fim de cada restauro · CONTROLO sobre o proprio FILTRO · `muta` aborta na ancora.
# ⛔⛔ NUNCA duas corridas deste script ao mesmo tempo (ele escreve na arvore de verdade).
#
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_pisca_mais_e_abre_espaco_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1

BRI="crates/ph2d-app-motion/src/motion_bridge.rs"
ESP="crates/ph2d-app-motion/src/motion_bridge_espaco.rs"
REW="crates/ph2d-app-motion/src/motion_bridge_rewire.rs"

TMP="$(mktemp -d)"
FALHAS=0
TOTAL=0

guarda()   { cp "$1" "$TMP/$(basename "$1").orig"; }
restaura() { cp "$TMP/$(basename "$1").orig" "$1"; touch "$1"; }

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

prova() { # nome pacote filtro
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(cargo test -p "$2" --lib -- "$3" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$3' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$3' passaram sobre o produto MUTADO"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

bloco() { # nome pacote filtro ficheiro vezes antigo novo
  guarda "$4"
  if muta "$4" "$5" "$6" "$7"; then prova "$1" "$2" "$3"; else FALHAS=$((FALHAS+1)); fi
  restaura "$4"
}

echo "== (F) O ECO PISCA TRES VEZES E ASSENTA =="

bloco 'o eco volta a uma passagem so (sem PULSO)' ph2d-app-motion o_eco_pisca \
  "$BRI" 1 \
  "    pulso * envelope" \
  "    envelope"

bloco 'o eco pisca e NAO assenta (sem envelope)' ph2d-app-motion o_eco \
  "$BRI" 1 \
  "    let envelope = (1.0 - p) * (1.0 - p);" \
  "    let envelope = 1.0 - 0.0 * p;"

bloco 'o eco pisca UMA vez' ph2d-app-motion o_eco_pisca \
  "$BRI" 1 \
  "pub(super) const PISCADAS: u32 = 3;" \
  "pub(super) const PISCADAS: u32 = 1;"

echo
echo "== (G) O VAO APERTADO ABRE-SE =="

bloco 'so o `v` anda, a jusante fica' ph2d-app-motion um_vao_apertado \
  "$ESP" 1 \
  "    for n in jusante(&motion.doc.graph, v) {" \
  "    for n in [v] {"

bloco 'o no fica onde caiu (em cima do vizinho)' ph2d-app-motion um_vao_apertado \
  "$ESP" 1 \
  "    if let Some(pn) = motion.doc.graph.pos(node) {" \
  "    if false && let Some(pn) = motion.doc.graph.pos(node) {"

bloco 'a arrumacao passa a correr SEMPRE' ph2d-app-motion um_vao_apertado \
  "$ESP" 1 \
  "    if vao >= preciso {
        return;
    }" \
  ""

bloco 'a paleta deixa de abrir espaco' ph2d-app-motion o_splice_da_paleta_abre \
  "$REW" 1 \
  "    // ⭐ **A MESMA lei do splice de um nó que já existe** — ver [\`abre_espaco\`]. Esta rota cria o
    // nó no ponto do gesto, e um fio entre dois cartões colados aperta-a exactamente igual.
    abre_espaco(motion, edge.from.0, node, edge.to.0);" \
  ""

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram."
else
  echo "⛔ $FALHAS de $TOTAL falharam."
fi
for f in "$BRI" "$ESP" "$REW"; do
  cmp -s "$f" "$TMP/$(basename "$f").orig" || {
    echo "⛔⛔ $f NAO foi restaurado — reponha de $TMP a mao."; exit 1; }
done
exit "$FALHAS"
