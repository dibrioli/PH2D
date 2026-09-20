#!/usr/bin/env bash
# Provas de mutacao das TRES ordens do dono de 2026-09-19 sobre o desenho do no' afastado:
#
#   (1) «permita que ate' que o zoom reduza os nos em 20 % a mais que agora, as fonts e numeros
#        dos nos ainda permanecam visiveis»
#   (2) «se depois disso o zoom continuar a reduzir os nos, o desenho tradicional dos nos se
#        modifica para uma simples capsula da cor caracteristica do grupo a que o no pertence, com
#        o nome do no ocupando toda capsula, os parametros de ajustes sao escondidos e os slots de
#        conexao ficam maiores»
#   (3) «os nomes dos nos ficam bem maiores nas capsulas»
#
# ⚠️ `touch` no fim de cada restauro · CONTROLO sobre o proprio FILTRO · `muta` aborta na ancora.
# ⛔⛔ NUNCA duas corridas deste script ao mesmo tempo (ele escreve na arvore de verdade).
#
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_a_capsula_e_o_piso_do_texto_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1

GC="crates/ph2d-panel-motion-graph/src/geom_card.rs"
GE="crates/ph2d-panel-motion-graph/src/geom.rs"
PC="crates/ph2d-panel-motion-graph/src/paint_card.rs"
CA="crates/ph2d-panel-motion-graph/src/paint_capsula.rs"

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

echo "== (1) O PISO DO TEXTO DESCE 20 % =="

# ⭐ **A folga a DESAPARECER (`1.0`) nem chega a correr um teste: o `const _: () = assert!` ao
#    lado dela NAO COMPILA.** Isso e' a cerca a ser mais forte que um gate, e e' de proposito —
#    aqui a mutacao e' a folga a MUDAR (`0.9`), que compila e e' o que um dedo distraido faz.
bloco 'a folga do dono muda de valor' ph2d-panel-motion-graph only_the_text_of_a_param_row \
  "$GC" 1 \
  "const FOLGA_DO_DONO: f32 = 0.8;" \
  "const FOLGA_DO_DONO: f32 = 0.9;"

bloco 'a row volta a agarrar-se ate' ph2d-panel-motion-graph uma_row_deixa_de_se_agarrar \
  "$GC" 1 \
  "    PARAM_LABEL_SIZE * view.zoom >= PISO_ANTIGO_PX
}" \
  "    param_text_is_drawn(view)
}"

echo
echo "== (2) A CAPSULA =="

bloco 'o no nunca vira capsula' ph2d-panel-motion-graph a_capsula_substitui_o_cartao \
  "$GC" 1 \
  "    if param_text_is_drawn(view) {
        Detalhe::Completo
    } else {
        Detalhe::Capsula
    }" \
  "    let _ = view;
    Detalhe::Completo"

bloco 'a capsula tem altura fixa' ph2d-panel-motion-graph a_capsula_tem_a_altura \
  "$GC" 1 \
  "    HEADER_H.max(pinos * ROW_H)" \
  "    let _ = pinos;
    HEADER_H"

bloco 'os pinos nao se centram na capsula' ph2d-panel-motion-graph os_pinos_centram_se \
  "$GE" 1 \
  "            n.y + capsula_h(n) * 0.5 + (i as f32 - (k - 1.0) * 0.5) * ROW_H" \
  "            n.y + HEADER_H + i as f32 * ROW_H + ROW_H * 0.5"

bloco 'os pinos voltam a encolher' ph2d-panel-motion-graph os_pinos_param_de_encolher \
  "$GC" 1 \
  "    raio_base * view.zoom.max(ZOOM_DA_CAPSULA)" \
  "    raio_base * view.zoom"

bloco 'o toggle e o badge sobrevivem a capsula' ph2d-panel-motion-graph numa_capsula_nao_ha_toggle \
  "$GE" 3 \
  "    if detalhe(view) == Detalhe::Capsula {
        return None;
    }" \
  ""

# ⛔⛔ **O filtro e' o gate do PRODUTO e nao o censo textual**, e a 1.a redaccao provou porque: o
#    `o_pintor_do_cartao_desvia` varre o ficheiro a' procura da chamada, e um `if false &&`
#    SOBREVIVE-LHE (o texto continua la'). Quem mata esta mutacao e' o arnes que PINTA: com o
#    desvio desligado o cartao volta a ser completo a `zoom 0,5` e a faixa de params volta a
#    chegar a pixel. *O censo textual afirma que alguem escreveu a chamada; so' o arnes de
#    pintura afirma que ela acontece.*
bloco 'o pintor deixa de desviar para a capsula' ph2d-panel-motion-graph the_bar_of_a_param_row \
  "$PC" 1 \
  "    if geom::detalhe(view) == geom::Detalhe::Capsula" \
  "    if false && geom::detalhe(view) == geom::Detalhe::Capsula"

echo
echo "== (3) O NOME, BEM MAIOR =="

bloco 'o nome da capsula volta ao tamanho do titulo' ph2d-panel-motion-graph o_nome_da_capsula_e_maior \
  "$CA" 1 \
  "const NOME_DA_CAPSULA: f32 = 0.62;" \
  "const NOME_DA_CAPSULA: f32 = 0.3;"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram."
else
  echo "⛔ $FALHAS de $TOTAL falharam."
fi
for f in "$GC" "$GE" "$PC" "$CA"; do
  cmp -s "$f" "$TMP/$(basename "$f").orig" || {
    echo "⛔⛔ $f NAO foi restaurado — reponha de $TMP a mao."; exit 1; }
done
exit "$FALHAS"
