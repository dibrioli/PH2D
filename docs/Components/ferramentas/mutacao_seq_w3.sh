#!/usr/bin/env bash
# Provas de mutação do TOP-20 #19 (a CUTSCENE) — W3, a SECÇÃO do Inspector e a cena.
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ **CONTROLO sobre o próprio FILTRO** — um filtro que casa ZERO testes sai VERDE, e isso
#      lê-se exactamente como «sobreviveu» (lição paga pela wave do projéctil, #14).
# ⚠️ **Toda troca é por `muta`, que ABORTA se o texto não aparecer o número esperado de vezes.**
#    o sangue que elas procuram.
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

prova() { # nome crate filtro [timeout_s]
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(timeout "${4:-900}" cargo test -p "$2" --all-targets -- "$3" 2>&1); rc=$?
  if [ "$rc" = 124 ]; then
    echo "  ✅ sangrou (o teste nao acabou em ${4}s — o prazo e' o que o fazia acabar)"
    return
  fi
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

# um bloco = guarda, muta, prova, restaura. Se a âncora falhar, restaura e conta como falha.
bloco() { # nome crate filtro ficheiro vezes antigo novo [timeout]
  guarda "$4" b
  if muta "$4" "$5" "$6" "$7"; then
    prova "$1" "$2" "$3" "${8:-900}"
  else
    TOTAL=$((TOTAL+1)); FALHAS=$((FALHAS+1))
  fi
  restaura "$4" b
}


FASE=shells/desktop/src/render_loop/fase_sequences.rs

PAINEL=crates/ph2d-panel-inspector/src
CORE=crates/ph2d-editor-core/src
FAMILIA=crates/ph2d-app-components/src
SHELL=shells/desktop/src

echo "════ W3 — a secção do Inspector e a cena ════"

# (1) ⛔ O CLIQUE numa opção morre no `is_focusable` sem o registo. É a doença que esta crate já
#     pagou SETE vezes: pintado, hit-registado e morto sob o dedo.
bloco "as opcoes saem do populate" ph2d-panel-inspector escolher_uma_cutscene \
  "$PAINEL/populate_sequence.rs" 1 \
  "    register_button_ids(store, &crate::ids::INSP_SEQ_OPT);" ""

# (2) ⛔ Sem o CHIP registado como `Dropdown` o despachante genérico não sabe ABRIR nada.
bloco "o chip deixa de ser Dropdown" ph2d-panel-inspector o_chip_e_pintado_e_e_um_dropdown \
  "$PAINEL/populate_sequence.rs" 1 \
  "    store.register(
        crate::ids::INSP_SEQ_PICK,
        InteractiveState::Dropdown {" \
  "    store.register(
        crate::ids::INSP_SEQ_CLEAR,
        InteractiveState::Dropdown {"

# (3) ⛔ O rect do chip tem de ATRAVESSAR o slot até ao passe diferido, senão o popover abre e não
#     se escolhe nada.
bloco "o rect do chip nao chega ao slot" ph2d-panel-inspector aberto_o_selector \
  "$PAINEL/sections/sequence.rs" 1 \
  "        crate::state_popovers::set_pending_seq_dd(Some(row.control));" ""

# (4) ⛔⛔ O que viaja é o NOME e não o índice — e a ISCA é o que torna isto observável.
bloco "a edicao passa a levar a primeira" ph2d-panel-inspector escolher_uma_cutscene \
  "$PAINEL/event_sequence.rs" 1 \
  "        if let Some(nome) = info.nomes.get(i) {" \
  "        if let Some(nome) = info.nomes.first() {"

# (5) ⛔ O popover que não fecha tapa as secções por baixo dele.
bloco "o popover nao fecha" ph2d-panel-inspector escolher_uma_cutscene \
  "$PAINEL/event_sequence.rs" 1 \
  "        fecha(host);" ""

# (6) ⛔ O botão de largar é a única porta de volta.
bloco "largar deixa de escrever" ph2d-panel-inspector largar_a_cutscene \
  "$PAINEL/event_sequence.rs" 1 \
  "        push(host, bits, E::Container(String::new()));" ""

# (7) ⛔ O botão só existe com cutscene escolhida — senão é um controlo que nunca faz nada.
bloco "o botao de largar aparece sempre" ph2d-panel-inspector largar_a_cutscene \
  "$PAINEL/sections/sequence.rs" 1 \
  "    if info.nome().is_none() {
        return y;
    }" ""

# (8) ⛔⛔ A resolução é a da LEI, e um `==` cru aqui mostraria «escolhida» sobre um nome que não
#     corre (a lei apara os dois lados).
bloco "a resolucao vira um == cru" ph2d-app-components o_nome_resolve_para_o_indice \
  "$FAMILIA/sequence_inspector.rs" 1 \
  "    let escolhido = seq.resolve(cutscenes.iter().map(|c| c.nome));" \
  "    let escolhido = cutscenes.iter().position(|c| c.nome == seq.container);"

# (9) ⛔ Escrever o MESMO nome não é uma mudança — senão um clique na cutscene já escolhida entra
#     no `Ctrl+Z`.
bloco "reescrever o mesmo nome conta" ph2d-app-components escrever_o_mesmo_nome \
  "$FAMILIA/sequence_inspector.rs" 1 \
  "    if seq.container == *nome {
        return false;
    }" ""

# (10) ⛔⛔ O aviso do relógio curto: sem a folga, um empate EXACTO acusaria o que o artista
#      escreveu de propósito.
bloco "a folga do relogio curto sai" ph2d-app-components um_empate_a_menos_de_um_milissegundo \
  "$CORE/sequence_edits.rs" 1 \
  "            && self.duracao_do_relogio + 1e-3 < self.duracao_da_cutscene" \
  "            && self.duracao_do_relogio < self.duracao_da_cutscene"

# (11) ⛔⛔⛔ A VISTA: o `bridge` e o painel têm de ler a MESMA porta.
bloco "o bridge reescreve a condicao" ph2d-host-desktop o_bridge_pergunta_a_porta \
  "$SHELL/render_loop/timeline_bridge.rs" 1 \
  "    if super::fase_sequences::a_vista_deixa_correr(solo, container) {" \
  "    if container.is_none() && !solo {"

# (12) ⛔ E a porta tem de dizer NÃO às DUAS vistas de edição.
bloco "a porta deixa a vista Keys correr" ph2d-host-desktop so_a_vista_da_cena_deixa \
  "$SHELL/render_loop/fase_sequences.rs" 1 \
  "    container.is_none() && !keys_mode" "    container.is_none()"

echo
if [ "$FALHAS" = 0 ]; then
  echo "✅ $TOTAL de $TOTAL mutações sangraram"
else
  echo "⛔ $FALHAS de $TOTAL NÃO sangraram"
fi
exit "$FALHAS"
