#!/usr/bin/env bash
# ⭐ PROVA DE MUTAÇÃO — as ordens do dono de 2026-09-20:
#   «Fonts maiores. linhas mais grossas. No lugar das capsulas os retângulos como nos headers dos
#    nós, só que grandes» · «fonts 30% maiores» · «aumenta a largura do retângulo conforme o
#    tamanho do nome».
#
# Cada mutação apaga UMA lei e tem de SANGRAR num gate nomeado.
#
# ⛔⛔ **O ARNÊS TEM DE PROVAR QUE A MUTAÇÃO ENTROU, e a 1.ª redacção não provava.** Ela escrevia
# os blocos de python a partir de outro python, e um `\n` dentro de uma string foi expandido cedo
# demais ⇒ o `python3` morria com `SyntaxError`, o ficheiro ficava INTACTO e a corrida imprimia
# `✗ SOBREVIVEU` sobre produto CERTO (cinco vezes). *Uma mutação que não aplica lê-se exactamente
# como uma que sobreviveu* — e é por isso que [`muta`] compara o ficheiro com o backup e recusa
# quando ele não mudou.
#
# ⚠️ E a segunda metade do mesmo achado: o arnês conta quantos testes de facto CORRERAM, porque um
# filtro que casa zero imprime `ok`.
#
#   bash "docs/Motion Nodes/ferramentas/mutacao_o_retangulo_a_fonte_e_o_fio_2026-09-20.sh"
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"

PANEL=crates/ph2d-panel-motion-graph/src
VIVOS=0
MORTOS=0

guarda() { cp "$1" "/tmp/mut_bak_$(basename "$1")"; }

restaura() {
  cp "/tmp/mut_bak_$(basename "$1")" "$1"
  touch "$1" # ⚠️ um `cp` devolve mtime antigo e o cargo guarda o build DA MUTAÇÃO
}

# muta <ficheiro> — o python vem do stdin. Recusa se o ficheiro não mudou.
muta() {
  local f="$1"
  guarda "$f"
  if ! python3 -; then
    echo "  ⛔ ARNÊS: o python da mutação falhou"
    return 1
  fi
  if cmp -s "$f" "/tmp/mut_bak_$(basename "$f")"; then
    echo "  ⛔ ARNÊS: a mutação NÃO ENTROU — o ficheiro está intacto"
    return 1
  fi
  touch "$f"
  return 0
}

# corre <pacote> <filtro> <rotulo>
corre() {
  local pkg="$1" filtro="$2" rotulo="$3"
  local saida n
  saida=$(cargo test -p "$pkg" --profile ci-test -- "$filtro" 2>&1)
  n=$(printf '%s' "$saida" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' | sort -rn | head -1)
  n=${n:-0}
  if [ "$n" -eq 0 ]; then
    echo "  ⛔ ARNÊS: o filtro «$filtro» casou ZERO testes — a corrida não afirma nada"
    VIVOS=$((VIVOS + 1))
    return
  fi
  if printf '%s' "$saida" | grep -q "test result: FAILED"; then
    echo "  ✓ SANGROU ($n testes) — $rotulo"
    MORTOS=$((MORTOS + 1))
  else
    echo "  ✗ SOBREVIVEU ($n testes) — $rotulo"
    VIVOS=$((VIVOS + 1))
  fi
}

# corre_compilador <pacote> <agulha> <rotulo> — para as leis cuja cerca é um `const assert!`,
# que o compilador impõe ANTES de qualquer teste. ⭐ Uma cerca do compilador é mais forte que um
# gate, e o arnês tem de saber lê-la: senão ela conta como «zero testes».
corre_compilador() {
  local pkg="$1" agulha="$2" rotulo="$3"
  local saida
  saida=$(cargo check -p "$pkg" --all-targets 2>&1)
  if printf '%s' "$saida" | grep -q "$agulha"; then
    echo "  ✓ SANGROU (compilador) — $rotulo"
    MORTOS=$((MORTOS + 1))
  else
    echo "  ✗ SOBREVIVEU (compilador) — $rotulo"
    VIVOS=$((VIVOS + 1))
  fi
}

falha_de_arnes() {
  echo "  ✗ contado como SOBREVIVENTE (o arnês não conseguiu medir)"
  VIVOS=$((VIVOS + 1))
}

echo "=== 1/10 · o canto volta a ser o de uma PASTILHA ==="
if muta "$PANEL/paint_capsula.rs" <<'PY'; then
p = "crates/ph2d-panel-motion-graph/src/paint_capsula.rs"
s = open(p).read()
a = "    CARD_RADIUS * view.zoom\n"
assert s.count(a) == 1, s.count(a)
open(p, "w").write(s.replace(a, "    view.zoom * 27.5\n", 1))
PY
  corre ph2d-panel-motion-graph o_canto_da_capsula "a forma volta a ser cápsula"
else falha_de_arnes; fi
restaura "$PANEL/paint_capsula.rs"

echo "=== 2/10 · a fonte volta ao corpo de ontem (21,5) ==="
if muta "$PANEL/paint_capsula.rs" <<'PY'; then
p = "crates/ph2d-panel-motion-graph/src/paint_capsula.rs"
s = open(p).read()
a = "pub(crate) const CAPSULA_FONTE: f32 = 27.95;"
assert s.count(a) == 1, s.count(a)
open(p, "w").write(s.replace(a, "pub(crate) const CAPSULA_FONTE: f32 = 21.5;", 1))
PY
  # ⚠️ A agulha é a ASSERÇÃO e não o código do erro: o rustc já mudou a frase à volta dela
  # («evaluation of constant value failed» → «evaluation panicked»), e a 1.ª redacção desta linha
  # procurava a frase antiga ⇒ lia `SOBREVIVEU` sobre uma cerca que reprovava. *Uma agulha que
  # procura a prosa de uma ferramenta envelhece com a ferramenta; o texto da nossa asserção não.*
  corre_compilador ph2d-panel-motion-graph "assertion failed: CAPSULA_FONTE" \
    "a catraca dos +30 % (cerca do compilador)"
else falha_de_arnes; fi
restaura "$PANEL/paint_capsula.rs"

echo "=== 3/10 · a estimativa de recurso volta a ser calibrada no CATÁLOGO ==="
if muta "$PANEL/geom_card.rs" <<'PY'; then
p = "crates/ph2d-panel-motion-graph/src/geom_card.rs"
s = open(p).read()
a = "const AVANCO_POR_CHAR: f32 = 1.06;"
assert s.count(a) == 1, s.count(a)
open(p, "w").write(s.replace(a, "const AVANCO_POR_CHAR: f32 = 0.706;", 1))
PY
  corre ph2d-panel-motion-graph o_nome_tem_um_tamanho "um nó renomeado com letras largas sai cortado"
else falha_de_arnes; fi
restaura "$PANEL/geom_card.rs"

echo "=== 4/10 · o fio volta a encolher na cápsula ==="
if muta "$PANEL/paint_wire.rs" <<'PY'; then
p = "crates/ph2d-panel-motion-graph/src/paint_wire.rs"
s = open(p).read()
a = "Detalhe::Capsula => base_w * view.zoom.max(ZOOM_DO_PISO_DO_FIO),"
assert s.count(a) == 1, s.count(a)
open(p, "w").write(s.replace(a, "Detalhe::Capsula => base_w * view.zoom,", 1))
PY
  corre ph2d-panel-motion-graph na_capsula_um_fio "o fio afastado volta a 0,54 px"
else falha_de_arnes; fi
restaura "$PANEL/paint_wire.rs"

echo "=== 5/10 · o piso morde TAMBÉM no regime do cartão ==="
if muta "$PANEL/paint_wire.rs" <<'PY'; then
p = "crates/ph2d-panel-motion-graph/src/paint_wire.rs"
s = open(p).read()
a = "Detalhe::Completo => base_w * view.zoom,"
assert s.count(a) == 1, s.count(a)
open(p, "w").write(
    s.replace(a, "Detalhe::Completo => base_w * view.zoom.max(ZOOM_DO_PISO_DO_FIO),", 1)
)
PY
  corre ph2d-panel-motion-graph acima_do_limiar_a_largura "o cartão de perto muda de desenho"
else falha_de_arnes; fi
restaura "$PANEL/paint_wire.rs"

echo "=== 6/10 · um traço de fio escapa à porta ==="
if muta "$PANEL/paint_wire.rs" <<'PY'; then
p = "crates/ph2d-panel-motion-graph/src/paint_wire.rs"
s = open(p).read()
a = "largura_do_fio(GHOST_W, view)"
assert s.count(a) == 2, s.count(a)
open(p, "w").write(s.replace(a, "GHOST_W * view.zoom", 2))
PY
  corre ph2d-panel-motion-graph nenhum_traco_de_fio "o fantasma volta a encolher sem ninguém ver"
else falha_de_arnes; fi
restaura "$PANEL/paint_wire.rs"

echo "=== 7/10 · a largura deixa de seguir o nome ==="
if muta "$PANEL/geom_card.rs" <<'PY'; then
p = "crates/ph2d-panel-motion-graph/src/geom_card.rs"
s = open(p).read()
a = "            let w = largura_da_capsula(n);"
assert s.count(a) == 1, s.count(a)
open(p, "w").write(s.replace(a, "            let w = CARD_W;", 1))
PY
  corre ph2d-panel-motion-graph a_largura_da_capsula "um nome comprido volta a ser cortado"
else falha_de_arnes; fi
restaura "$PANEL/geom_card.rs"

echo "=== 8/10 · a pastilha cresce só para a direita ==="
if muta "$PANEL/geom_card.rs" <<'PY'; then
p = "crates/ph2d-panel-motion-graph/src/geom_card.rs"
s = open(p).read()
a = "            (n.x + (CARD_W - w) * 0.5, w)"
assert s.count(a) == 1, s.count(a)
open(p, "w").write(s.replace(a, "            (n.x, w)", 1))
PY
  corre ph2d-panel-motion-graph a_capsula_larga_cresce "as 42 unidades caem todas num vão só"
else falha_de_arnes; fi
restaura "$PANEL/geom_card.rs"

echo "=== 9/10 · o pino volta à borda do CARTÃO ==="
if muta "$PANEL/geom.rs" <<'PY'; then
p = "crates/ph2d-panel-motion-graph/src/geom.rs"
s = open(p).read()
a = "    let edge_x = if output { cx + cw } else { cx };"
assert s.count(a) == 1, s.count(a)
open(p, "w").write(
    s.replace(a, "    let edge_x = if output { n.x + CARD_W } else { n.x };", 1)
)
PY
  corre ph2d-panel-motion-graph o_pino_aterra_na_borda "o fio aterra no meio do nome"
else falha_de_arnes; fi
restaura "$PANEL/geom.rs"

echo "=== 10/10 · a largura perde o PISO do cartão ==="
if muta "$PANEL/geom_card.rs" <<'PY'; then
p = "crates/ph2d-panel-motion-graph/src/geom_card.rs"
s = open(p).read()
a = "    CARD_W.max(texto + 2.0 * MARGEM_X_DA_CAPSULA)"
assert s.count(a) == 1, s.count(a)
open(p, "w").write(s.replace(a, "    texto + 2.0 * MARGEM_X_DA_CAPSULA", 1))
PY
  corre ph2d-panel-motion-graph a_largura_da_capsula "um nome curto ENCOLHE a pastilha"
else falha_de_arnes; fi
restaura "$PANEL/geom_card.rs"

echo
echo "=== PLACAR: $MORTOS sangraram · $VIVOS sobreviveram ==="
[ "$VIVOS" -eq 0 ] || exit 1
