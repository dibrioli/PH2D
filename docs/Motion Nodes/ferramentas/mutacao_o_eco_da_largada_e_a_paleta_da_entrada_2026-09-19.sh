#!/usr/bin/env bash
# Provas de mutacao das DUAS ordens do dono de 2026-09-19 (a seguir ao smoke OK):
#
#   (D) «faltou uma indicacao visual da troca: nos e linhas podem ganhar um destaque de cor […]
#        de que estao sobrepostos prestes a trocar ou encaixar. Apos a troca o conjunto linha e
#        no piscam e se acentam. Undo/redo implementado para essas acoes.»
#   (E) «puxar um fio de um slot de entrada (a esquerda do no) ainda nao chama o modal de nos
#        compativeis. Faca isso possivel.»
#
# ⚠️ `touch` no fim de cada restauro · CONTROLO sobre o proprio FILTRO · `muta` aborta na ancora.
# ⛔⛔ NUNCA duas corridas deste script ao mesmo tempo (ele escreve na arvore de verdade).
#
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_o_eco_da_largada_e_a_paleta_da_entrada_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1

SOC="crates/ph2d-panel-motion-graph/src/interact_socket.rs"
MEN="crates/ph2d-panel-motion-graph/src/snapshot_menu.rs"
INT="crates/ph2d-panel-motion-graph/src/snapshot_intent.rs"
EDI="crates/ph2d-app-motion/src/motion_bridge_edit.rs"
ITR="crates/ph2d-panel-motion-graph/src/interact.rs"
CAR="crates/ph2d-panel-motion-graph/src/paint_card.rs"
WIR="crates/ph2d-panel-motion-graph/src/paint_wires.rs"
REA="crates/ph2d-panel-motion-graph/src/realce.rs"
REW="crates/ph2d-app-motion/src/motion_bridge_rewire.rs"
BRI="crates/ph2d-app-motion/src/motion_bridge.rs"

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

echo "== (E) A PALETA DE QUEM PODE ALIMENTAR UMA ENTRADA =="

bloco 'a largada para tras volta a ser um no-op' ph2d-panel-motion-graph uma_entrada_solta \
  "$SOC" 1 \
  "                    } else {
                        // ⭐⭐⭐ **Largado no VAZIO: a paleta dos que podem ALIMENTAR esta entrada**" \
  "                    } else if false {
                        // ⭐⭐⭐ **Largado no VAZIO: a paleta dos que podem ALIMENTAR esta entrada**"

bloco 'a paleta deixa de FILTRAR' ph2d-panel-motion-graph uma_entrada_solta \
  "$MEN" 1 \
  "    all.into_iter()
        .filter(|c| c.outputs.iter().any(|o| feeds(&o.ty, inp)))
        .collect()" \
  "    let _ = inp;
    all"

bloco 'o router deixa de reconhecer o quarto contexto' ph2d-panel-motion-graph the_wire_context \
  "$INT" 1 \
  "    } else if let Some((to_node, to_port)) = connect_to {" \
  "    } else if false && let Some((to_node, to_port)) = connect_to {"

bloco 'o no escolhido nasce SOLTO' ph2d-app-motion o_no_escolhido_nasce \
  "$EDI" 1 \
  "    if let Some(port) = port {
        #[expect(
            clippy::cast_possible_truncation,
            reason = \"um indice de porta e' u16 em todo o contrato\"
        )]
        let edge = Edge {
            from: (source, port as u16),
            to: (NodeId(to_node), to_port),
            delayed: false,
        };" \
  "    if let Some(port) = port {
        #[expect(
            clippy::cast_possible_truncation,
            reason = \"um indice de porta e' u16 em todo o contrato\"
        )]
        let edge = Edge {
            from: (source, port as u16),
            to: (source, to_port),
            delayed: false,
        };"

echo
echo "== (D1) O ALVO ACENDE ANTES DE LARGAR =="

bloco 'o alvo nunca acende' ph2d-panel-motion-graph o_alvo_acende_no_arrasto \
  "$ITR" 1 \
  "            state.largada_viva = sozinho.and_then(|um| {" \
  "            state.largada_viva = None.and_then(|um: u32| {"

bloco 'o alvo fica aceso depois de largar' ph2d-panel-motion-graph o_alvo_acende_no_arrasto \
  "$ITR" 1 \
  "            state.largada_viva = None;
        }
        // **Double-click a collapsed card" \
  "        }
        // **Double-click a collapsed card"

# ⛔⛔ **A 1.a redaccao destas duas SOBREVIVEU, e mudou o desenho do produto.** Elas desligavam a
#    pintura com um `if false &&` e o censo TEXTUAL (que varria os pintores pelo nome do campo)
#    ficava verde — o nome continua la'. ⇒ a DECISAO mudou-se para uma funcao pura
#    (`realce`), medida por VALOR, e o censo textual passou a perguntar a unica coisa que
#    so' texto responde: *o pintor ainda a CHAMA?*. As duas mutacoes abaixo sao uma de cada.
bloco 'a lei do realce deixa de acender o alvo' ph2d-panel-motion-graph realce:: \
  "$REA" 1 \
  "    if state.largada_viva == Some(Largada::Troca(id)) {
        return Some(1.0);
    }" \
  ""

bloco 'a lei do realce deixa de desvanecer o eco' ph2d-panel-motion-graph realce:: \
  "$REA" 1 \
  "    p.fios.contains(&to).then(|| p.t.clamp(0.0, 1.0))" \
  "    p.fios.contains(&to).then_some(1.0)"

bloco 'o cartao deixa de CHAMAR a lei' ph2d-panel-motion-graph os_pintores_chamam \
  "$CAR" 1 \
  "crate::realce::realce_do_cartao(state, n.id)" \
  "None::<f32>"

bloco 'o fio deixa de CHAMAR a lei' ph2d-panel-motion-graph os_pintores_chamam \
  "$WIR" 1 \
  "crate::realce::realce_do_fio(p.state, (e.to_node, e.to_port))" \
  "None::<f32>"

echo
echo "== (D2) O ECO DEPOIS DA ACCAO =="

bloco 'a troca deixa de acender o eco' ph2d-app-motion o_eco_da_largada \
  "$REW" 1 \
  "    motion.piscada = Some(crate::motion_state::PiscadaPendente {
        nos: vec![a.0, b.0],
        fios,
        inicio: motion.ui_now,
    });" \
  "    let _ = fios;"

bloco 'so os NOS piscam, os fios nao' ph2d-app-motion o_eco_da_largada \
  "$REW" 1 \
  "        .map(|e| (e.to.0.0, e.to.1))
        .collect()" \
  "        .map(|e| (e.to.0.0, e.to.1))
        .take(0)
        .collect()"

bloco 'o eco nunca desvanece' ph2d-app-motion o_eco_da_largada \
  "$BRI" 1 \
  "        t: resta * resta," \
  "        t: 1.0,"

bloco 'o eco fica aceso para sempre' ph2d-app-motion o_eco_da_largada \
  "$BRI" 1 \
  "    if decorrido >= PISCADA_S {" \
  "    if false && decorrido >= PISCADA_S {"

echo
echo "== (D3) UM GESTO, UM CTRL+Z =="

bloco 'o estrutural sai para FORA do parenteses' ph2d-panel-motion-graph o_arrasto_real_pede_a_troca \
  "$ITR" 1 \
  "                if let [um] = nodes.as_slice() {
                    let view = View::new(rect, state.view);
                    node_drop::pedir(snap, &view, *um, (g.x, g.y), moved);
                }
                push_intent(GraphIntent::EndDrag);" \
  "                push_intent(GraphIntent::EndDrag);
                if let [um] = nodes.as_slice() {
                    let view = View::new(rect, state.view);
                    node_drop::pedir(snap, &view, *um, (g.x, g.y), moved);
                }"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutacoes sangraram."
else
  echo "⛔ $FALHAS de $TOTAL falharam."
fi
for f in "$SOC" "$MEN" "$INT" "$EDI" "$ITR" "$CAR" "$WIR" "$REA" "$REW" "$BRI"; do
  cmp -s "$f" "$TMP/$(basename "$f").orig" || {
    echo "⛔⛔ $f NAO foi restaurado — reponha de $TMP a mao."; exit 1; }
done
exit "$FALHAS"
