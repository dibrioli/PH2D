#!/usr/bin/env bash
# Provas de mutacao das TRES ordens do dono de 2026-09-19 sobre o grafo:
#
#   (A) «Se arrastar um no' no grafo em cima de outro no', eles mudam de posicao na cadeia»
#   (B) «Se arrastar num no' em cima de uma conexao (linha) […] ele passa a ser conectado naquela
#        linha, contudo, sem quebrar a cadeia»
#   (C) «quando puxo um fio de grip e abre-se o modal de nos e escolho Duplicator, o grid em vez
#        de se conectar em points, esta' se conectando no slot de Shape»
#
# ⭐⭐⭐ (C) e' a MESMA lei que o splice ja' curara em 2026-09-01, escrita uma segunda e uma
#    terceira vez. Hoje as tres rotas passam pela porta `ph2d_node_registry::landing_port`.
#
# ⚠️ `touch` no fim de cada restauro · CONTROLO sobre o proprio FILTRO · `muta` aborta na ancora.
# ⛔⛔ NUNCA duas corridas deste script ao mesmo tempo (ele escreve na arvore de verdade).
#
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_a_largada_de_uma_carta_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1

REG="crates/ph2d-node-registry/src/port_landing.rs"
EDI="crates/ph2d-app-motion/src/motion_bridge_edit.rs"
DRP="crates/ph2d-panel-motion-graph/src/interact_drop.rs"
REW="crates/ph2d-app-motion/src/motion_bridge_rewire.rs"
NDR="crates/ph2d-panel-motion-graph/src/interact_node_drop.rs"

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

echo "== (C) A PORTA POR ONDE UM FIO ATERRA =="

bloco 'a principal deixa de ganhar' ph2d-node-registry landing \
  "$REG" 1 \
  "    if primary < n && serve(primary) {
        return Some(primary);
    }" \
  ""

bloco 'a principal passa a ser OBRIGATORIA (nao cede)' ph2d-node-registry landing \
  "$REG" 1 \
  "    (0..n).find(|&i| serve(i))" \
  "    None"

bloco 'o smart_connect volta a primeira que casa o tipo' ph2d-app-motion o_fio_da_paleta_aterra \
  "$EDI" 1 \
  "        let primaria = motion.registry.primary_input(man.id);" \
  "        let primaria = 0;"

bloco 'o fio largado no CORPO volta a primeira livre' ph2d-panel-motion-graph um_fio_largado_no_corpo \
  "$DRP" 1 \
  "    ph2d_node_registry::landing_port(node.primary_input, node.inputs.len(), |i| {" \
  "    ph2d_node_registry::landing_port(0, node.inputs.len(), |i| {"

bloco 'o heal de um no apagado volta a porta 0' ph2d-app-motion o_heal_de_um_duplicador \
  "$REW" 1 \
  "    let principal = entrada_de(&motion.doc.graph, &motion.registry, nid, None);" \
  "    let principal = 0;"

echo
echo "== (A) A TROCA DE LUGAR NA CADEIA =="

bloco 'meia troca: so as entradas' ph2d-app-motion dois_nos_trocam_de_lugar \
  "$REW" 1 \
  "                from: (troca(e.from.0), e.from.1)," \
  "                from: (e.from.0, e.from.1),"

bloco 'as cartas deixam de trocar de sitio' ph2d-app-motion as_cartas_trocam_de_sitio \
  "$REW" 1 \
  "    if let (Some(pa), Some(pb)) = (pa, pb) {" \
  "    if false && let (Some(pa), Some(pb)) = (pa, pb) {"

bloco 'a troca deixa de validar' ph2d-app-motion uma_troca_que_nao_cabe \
  "$REW" 1 \
  "    }) && trial.validate(&motion.registry).is_ok();" \
  "    });"

echo
echo "== (B) ENFIAR UM NO NUM FIO, SEM QUEBRAR A CADEIA =="

bloco 'a cadeia de onde ele saiu deixa de fechar' ph2d-app-motion um_no_enfiado_num_fio \
  "$REW" 1 \
  "    if let Some(f) = fonte {" \
  "    if false && let Some(f) = fonte {"

# ⛐⛐ **A ancora casava DUAS vezes, e isso era um achado:** as duas rotas de splice tinham o
#    corpo byte a byte igual. Hoje ha' uma PORTA (`liga_pelo_meio`) e a mutacao tem um so' sitio.
bloco 'a outra ponta do fio fica a pairar' ph2d-app-motion um_no_enfiado_num_fio \
  "$REW" 1 \
  "        && trial
            .connect(Edge {
                from: (node, 0),
                to: edge.to,
                delayed: false,
            })
            .is_ok()" \
  ""

# ⛔⛔ **A guarda do PROPRIO fio foi APAGADA do produto por esta corrida.** A mutacao que a
#    removia NAO SANGROU: sem ela o splice tenta ligar `no -> no`, que e' um ciclo, e o `connect`
#    recusa-o na mesma. *Uma linha que a mutacao nao consegue matar nao e' lei, e' comentario com
#    sintaxe de codigo* (CLAUDE.md §5.0). O gate `enfiar_um_no_no_proprio_fio_e_inerte` FICA — ele
#    passou a medir a recusa de quem de facto a faz.

echo
echo "== A LEITURA DA LARGADA, NO PAINEL =="

bloco 'o FIO passa a ganhar do NO' ph2d-panel-motion-graph o_no_ganha_do_fio \
  "$NDR" 1 \
  "    if let Some(outro) = snap.nodes.iter().find(|n| {" \
  "    if false && let Some(outro) = snap.nodes.iter().find(|n| {"

bloco 'os fios do proprio no voltam a contar' ph2d-panel-motion-graph os_fios_do_proprio_no \
  "$NDR" 1 \
  "    candidatos.retain(|&(tn, tp)| {
        tn != arrastado
            && snap
                .edges
                .iter()
                .any(|e| e.to_node == tn && e.to_port == tp && e.from_node != arrastado)
    });" \
  ""

bloco 'o pedido do FIO nunca sai do painel' ph2d-panel-motion-graph node_drop \
  "$NDR" 1 \
  "        Some(Largada::Fio(to_node, to_port)) => {
            super::push_intent(GraphIntent::SpliceExistingIntoWire {
                node: arrastado,
                to_node,
                to_port,
            });
        }" \
  "        Some(Largada::Fio(..)) => {}"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram."
else
  echo "⛔ $FALHAS de $TOTAL falharam."
fi
# ⛔⛔ A prova do restauro compara-se com o BACKUP, nunca com o `HEAD`.
for f in "$REG" "$EDI" "$DRP" "$REW" "$NDR"; do
  cmp -s "$f" "$TMP/$(basename "$f").orig" || {
    echo "⛔⛔ $f NAO foi restaurado — reponha de $TMP a mao."; exit 1; }
done
exit "$FALHAS"
