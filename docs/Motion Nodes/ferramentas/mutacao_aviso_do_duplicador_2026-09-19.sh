#!/usr/bin/env bash
# Provas de mutação do AVISO DE VISIBILIDADE (ordem do dono, 2026-09-19:
# «coloque um alerta de que se não forem usados com duplicator e um objeto a ser copiado,
# são invisíveis»).
#
# A corrente que o aviso atravessa tem QUATRO elos, e cada bloco mata um:
#
#   manifesto  →  a REGRA (marca_as_fontes_de_posicoes)      · os gates do app-motion
#              →  a BANDEIRA no registo                       · idem
#              →  a VISTA do cartão (snapshot_from)           · a_bandeira_chega_ao_cartao
#              →  a GEOMETRIA da fileira + a TINTA            · os gates do painel
#
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o MUTADO).
# ⚠️⚠️ CONTROLO sobre o próprio FILTRO — um filtro que casa ZERO testes sai VERDE e lê-se
#    exactamente como «SOBREVIVEU».
# ⚠️ Toda troca é por `muta`, que ABORTA se a âncora não aparecer o número esperado de vezes.
#
# Corra-o pela porta de recursos:
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_aviso_do_duplicador_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1

REG=crates/ph2d-node-registry/src/fontes_de_posicoes.rs
SNAP=crates/ph2d-panel-motion-graph/src/snapshot_build.rs
GEOM=crates/ph2d-panel-motion-graph/src/geom_card.rs
PAINT=crates/ph2d-panel-motion-graph/src/paint_card.rs
INIT=crates/ph2d-node-registry-init/src/lib.rs

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

echo "=== ELO 1 — a REGRA das tres clausulas ==="

bloco "a clausula do Vec2 cai: os nos de VALOR ganham o aviso" \
  ph2d-app-motion "lei_da_aparencia" "$REG" 1 \
  "            |p: &PortSpec| p.ty.domain == Domain::Instances && p.ty.dim == Dim::Vec2;" \
  "            |p: &PortSpec| p.ty.domain == Domain::Instances;"

bloco "a clausula da ORIGEM DE APARENCIA cai: o aviso mente no source.object" \
  ph2d-app-motion "lei_da_aparencia" "$REG" 1 \
  "            .filter(|id| !self.is_object_source(*id) && !self.is_live_vector_source(*id))" \
  "            .filter(|id| { let _ = id; true })"

bloco "o `!` do input inverte: toda PASSAGEM ganha o aviso" \
  ph2d-app-motion "lei_da_aparencia" "$REG" 1 \
  "            .filter(|m| m.outputs.iter().any(emite_posicoes) && !m.inputs.iter().any(recebe))" \
  "            .filter(|m| m.outputs.iter().any(emite_posicoes) && m.inputs.iter().any(recebe))"

bloco "a passagem deixa de marcar ninguem (o piso de populacao)" \
  ph2d-app-motion "lei_da_aparencia" "$REG" 1 \
  "        for id in fontes {
            self.so_posicoes.insert(id);
        }" \
  "        let _ = fontes;"

echo
echo "=== ELO 2 — a ORDEM em que ela corre ==="

bloco "a classificacao passa a ser ADITIVA sem cerca (a ordem deixa de decidir)" \
  ph2d-node-registry "a_classificacao_das_fontes" "$REG" 1 \
  "            .filter(|id| !self.is_object_source(*id) && !self.is_live_vector_source(*id))" \
  "            .filter(|id| { let _ = id; true })"

echo
echo "=== ELO 3 — a bandeira chega a' VISTA do cartao ==="

bloco "o snapshot crava `false` (a rotura abaixo do corte)" \
  ph2d-app-motion "a_bandeira_chega_ao_cartao" "$SNAP" 1 \
  "                so_posicoes: registry.so_posicoes(type_id)," \
  "                so_posicoes: false,"

echo
echo "=== ELO 4 — a fileira e a TINTA ==="

bloco "a nota deixa de ter fileira (a frase pinta por cima do readout)" \
  ph2d-panel-motion-graph "a_nota_custa_uma_fileira" "$GEOM" 1 \
  "    if n.so_posicoes { 1.0 } else { 0.0 }" \
  "    let _ = n;
    0.0"

bloco "o topo da nota some (nada e' pintado)" \
  ph2d-panel-motion-graph "a_nota_custa_uma_fileira" "$GEOM" 1 \
  "    n.so_posicoes
        .then(|| param_band_top(n) + card_param_rows(n) * ROW_H)" \
  "    let _ = n;
    None"

bloco "o bloco da TINTA e' apagado (o ecra fica em branco, a geometria verde)" \
  ph2d-panel-motion-graph "o_aviso_de_visibilidade_chega_a_pixel" "$PAINT" 1 \
  "    if let Some(top) = geom::nota_top(n) {" \
  "    if let Some(top) = geom::nota_top(n).filter(|_| false) {"

echo
echo "── TOTAL: $TOTAL provas · $FALHAS falha(s)"
[ "$FALHAS" = 0 ]
