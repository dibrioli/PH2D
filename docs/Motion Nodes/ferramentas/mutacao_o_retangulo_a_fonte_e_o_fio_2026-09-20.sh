#!/usr/bin/env bash
# ⭐ PROVA DE MUTAÇÃO — a ordem do dono de 2026-09-20: *«Fonts maiores. linhas mais grossas. No
# lugar das capsulas os retângulos como nos headers dos nós, só que grandes»*.
#
# Cada mutação apaga UMA lei e tem de SANGRAR num gate nomeado. ⚠️ O arnês conta quantos testes
# de facto correram: um filtro que casa ZERO testes imprime `ok` e lê-se como «sobreviveu»
# (cicatriz medida nesta casa três vezes).
#
#   bash "docs/Motion Nodes/ferramentas/mutacao_o_retangulo_a_fonte_e_o_fio_2026-09-20.sh"
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"

PANEL=crates/ph2d-panel-motion-graph/src
APP=crates/ph2d-app-motion/src
VIVOS=0
MORTOS=0

restaura() {
  for f in "$@"; do
    cp "/tmp/mut_bak_$(basename "$f")" "$f"
    touch "$f" # ⚠️ um `cp`/`mv` devolve mtime antigo e o cargo guarda o build DA MUTAÇÃO
  done
}

guarda() {
  for f in "$@"; do cp "$f" "/tmp/mut_bak_$(basename "$f")"; done
}

# corre <pacote> <filtro> <rotulo>
corre() {
  local pkg="$1" filtro="$2" rotulo="$3"
  local saida
  saida=$(cargo test -p "$pkg" --profile ci-test -- "$filtro" 2>&1)
  local n
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

echo "=== 1/6 · o canto volta a ser o de uma PASTILHA ==="
guarda "$PANEL/paint_capsula.rs"
python3 - <<'PY'
p='crates/ph2d-panel-motion-graph/src/paint_capsula.rs'
s=open(p).read()
a="    CARD_RADIUS * view.zoom\n"
assert s.count(a)==1, s.count(a)
open(p,'w').write(s.replace(a,"    view.zoom * 27.5\n",1))
PY
touch "$PANEL/paint_capsula.rs"
corre ph2d-panel-motion-graph o_canto_da_capsula "a forma volta a ser cápsula"
restaura "$PANEL/paint_capsula.rs"

echo "=== 2/6 · a fonte volta à estimativa por contagem de caracteres (17,9) ==="
guarda "$PANEL/paint_capsula.rs"
sed -i 's/^pub(crate) const CAPSULA_FONTE: f32 = 21.5;/pub(crate) const CAPSULA_FONTE: f32 = 17.9;/' "$PANEL/paint_capsula.rs"
sed -i 's/^const _: () = assert!(CAPSULA_FONTE > 1.6 \* TITLE_SIZE);/const _: () = assert!(CAPSULA_FONTE > 1.3 * TITLE_SIZE);/' "$PANEL/paint_capsula.rs"
touch "$PANEL/paint_capsula.rs"
corre ph2d-panel-motion-graph o_nome_tem_um_tamanho_so "o corpo deixa de ser MÁXIMO"
restaura "$PANEL/paint_capsula.rs"

echo "=== 3/6 · a fonte passa do que cabe (corte com reticências) ==="
guarda "$PANEL/paint_capsula.rs"
sed -i 's/^pub(crate) const CAPSULA_FONTE: f32 = 21.5;/pub(crate) const CAPSULA_FONTE: f32 = 24.0;/' "$PANEL/paint_capsula.rs"
touch "$PANEL/paint_capsula.rs"
corre ph2d-panel-motion-graph o_nome_tem_um_tamanho_so "o pior nome do catálogo é CORTADO (painel)"
corre ph2d-app-motion nenhum_nome_do_catalogo "o pior nome do catálogo é CORTADO (censo dos 136)"
restaura "$PANEL/paint_capsula.rs"

echo "=== 4/6 · o fio volta a encolher na cápsula ==="
guarda "$PANEL/paint_wire.rs"
python3 - <<'PY'
p='crates/ph2d-panel-motion-graph/src/paint_wire.rs'
s=open(p).read()
a="        crate::geom::Detalhe::Capsula => base_w * view.zoom.max(ZOOM_DO_PISO_DO_FIO),\n"
assert s.count(a)==1, s.count(a)
open(p,'w').write(s.replace(a,"        crate::geom::Detalhe::Capsula => base_w * view.zoom,\n",1))
PY
touch "$PANEL/paint_wire.rs"
corre ph2d-panel-motion-graph na_capsula_um_fio "o fio afastado volta a 0,54 px"
restaura "$PANEL/paint_wire.rs"

echo "=== 5/6 · o piso morde TAMBÉM no regime do cartão ==="
guarda "$PANEL/paint_wire.rs"
python3 - <<'PY'
p='crates/ph2d-panel-motion-graph/src/paint_wire.rs'
s=open(p).read()
a="        crate::geom::Detalhe::Completo => base_w * view.zoom,\n"
assert s.count(a)==1, s.count(a)
open(p,'w').write(s.replace(a,"        crate::geom::Detalhe::Completo => base_w * view.zoom.max(ZOOM_DO_PISO_DO_FIO),\n",1))
PY
touch "$PANEL/paint_wire.rs"
corre ph2d-panel-motion-graph acima_do_limiar_a_largura "o cartão de perto muda de desenho"
restaura "$PANEL/paint_wire.rs"

echo "=== 6/6 · um traço de fio escapa à porta ==="
guarda "$PANEL/paint_wire.rs"
python3 - <<'PY'
p='crates/ph2d-panel-motion-graph/src/paint_wire.rs'
s=open(p).read()
a="largura_do_fio(GHOST_W, view)"
assert s.count(a)==2, s.count(a)
open(p,'w').write(s.replace(a,"GHOST_W * view.zoom",2))
PY
touch "$PANEL/paint_wire.rs"
corre ph2d-panel-motion-graph nenhum_traco_de_fio "o fantasma volta a encolher sem ninguém ver"
restaura "$PANEL/paint_wire.rs"

echo
echo "=== PLACAR: $MORTOS sangraram · $VIVOS sobreviveram ==="
[ "$VIVOS" -eq 0 ] || exit 1
