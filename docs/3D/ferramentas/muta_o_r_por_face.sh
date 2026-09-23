#!/usr/bin/env bash
# muta_o_r_por_face.sh — prova de mutacao da P2: a reticula deixa de ter UM
# lado e passa a ter um nivel POR FACE.
#
# ⚠️ A POPULACAO sao TRES crates pela lei do §20.6 (a populacao de um arnes e'
# de quem OBSERVA a mutacao, nunca de quem a CONTEM): o enderecamento vive na
# `ph2d-mesh-colors`, a guarda do device na `ph2d-mesh-render`, e os dois elos
# do censo de fiacao na `ph2d-app-sculpt3d`.
#
# ⚠️ Auto-controlos (os quatro de sempre): ancora unica · a mutacao compila ·
# N > 0 testes correram · a corrida LIMPA verde. Mais o PRE-VOO
# (`MUTA_SO_ANCORAS=1`), que apanha uma ancora morta pelo `cargo fmt` em
# segundos e sem correr um teste.
#
# ⛔ Chame-o sempre pela porta de recursos:
#   PH2D_PRAZO=3600 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_o_r_por_face.sh
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
COL=crates/ph2d-mesh-colors/src
REN=crates/ph2d-mesh-render/src
SCU=crates/ph2d-sculpt3d/src
BK=$(mktemp -d)
cp -r "$COL" "$BK/col"; cp -r "$REN" "$BK/ren"; cp -r "$SCU" "$BK/scu"
restore() {
  rm -rf "$COL"; cp -r "$BK/col" "$COL"
  rm -rf "$REN"; cp -r "$BK/ren" "$REN"
  rm -rf "$SCU"; cp -r "$BK/scu" "$SCU"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  find "$COL" "$REN" "$SCU" -name '*.rs' -exec touch {} +
}
trap restore EXIT

FILTRO='test(/p2_tests|assar_tests|vinte_e_cinco|nivel_base|canto_de_uma_face|dois_lados_de_uma_aresta|ponto_de_uma_face|recusa_nomeia|graduado/)'
corrida() {
  cargo nextest run -p ph2d-mesh-colors -p ph2d-mesh-render -p ph2d-app-sculpt3d -E "$FILTRO" 2>&1
}
populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  limpa=$(corrida); rc_limpo=$?
  verde=$(printf '%s' "$limpa" | populacao)
  echo "VERDE antes: $verde testes correram"
  [ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa nao correu teste nenhum"; exit 2; }
  if [ "$rc_limpo" -ne 0 ]; then
    echo "ABORTO: a corrida limpa esta' VERMELHA -- um placar daqui e' fabricado."
    printf '%s' "$limpa" | grep -E '^ *(FAIL|Summary)' | tail -6 | sed 's/^/      | /'
    exit 2
  fi
fi

sangram=0; total=0
muta() { # ficheiro  ancora  substituto  nome
  local f="$1" agulha="$2" subst="$3" nome="$4"
  total=$((total+1))
  local n; n=$(python3 -c 'import sys;print(open(sys.argv[1]).read().count(sys.argv[2]))' "$f" "$agulha")
  if [ -n "$SO_ANCORAS" ]; then
    if [ "$n" -ne 1 ]; then echo "  ✗ ANCORA [$nome]: casou $n vezes (esperado 1)"; else sangram=$((sangram+1)); fi
    return
  fi
  if [ "$n" -ne 1 ]; then echo "  ABORTO [$nome]: a ancora casou $n vezes (esperado 1)"; return; fi
  python3 -c '
import sys
p,a,b = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
assert s.count(a) == 1, (p, s.count(a))
open(p,"w").write(s.replace(a, b, 1))
' "$f" "$agulha" "$subst"
  touch "$f"
  local out rc corridos
  out=$(corrida); rc=$?
  if echo "$out" | grep -q '^error\[\|^error: could not compile'; then
    echo "  ABORTO [$nome]: a mutacao nao compila"
    printf '%s' "$out" | grep -E '^error' | head -3 | sed 's/^/      | /'
  else
    corridos=$(printf '%s' "$out" | populacao)
    if [ "$corridos" -eq 0 ]; then
      echo "  ABORTO [$nome]: zero testes correram — ultimas linhas:"
      printf '%s' "$out" | tail -6 | sed 's/^/      | /'
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── A LEI DO SUBCONJUNTO, que e' a P2 inteira num produto ───────────────
# ⚠️ Sem o passo, a amostra `t = 1` de uma face de lado `2` cai na celula `1` de
#    uma aresta de lado `8` — um OITAVO do caminho em vez de metade.
muta "$COL/enderecos.rs" \
  '            let t = t * (le / lf);' \
  '            let t = t;' \
  'P1 a face grossa deixa de multiplicar pelo passo: a fronteira parte-se'

# ── A ARESTA leva o MAXIMO dos vizinhos ─────────────────────────────────
muta "$COL/topo.rs" \
  '                *e = (*e).max(k);' \
  '                *e = k;' \
  'P2 a aresta passa a levar a ULTIMA face em vez do maximo'

# ── O PREFIXO do bloco das arestas e' uma SOMA ──────────────────────────
muta "$COL/topo.rs" \
  '            a += (1u32 << k) - 1;' \
  '            a += 1;' \
  'P3 o bloco das arestas volta a supor um comprimento fixo'

# ── A RECUSA de uma lista de niveis do tamanho errado ───────────────────
muta "$COL/topo.rs" \
  '        if niveis.len() != self.faces() {
            return None;
        }' \
  '        if false {
            return None;
        }' \
  'P4 a regraduada aceita uma lista que nao descreve a malha'

# ── O `nivel_uniforme`, que e' quem RECUSA em dois consumidores ─────────
muta "$COL/topo.rs" \
  '        self.nivel_da_face.iter().all(|x| *x == k).then_some(k)' \
  '        Some(k)' \
  'P5 tudo se le como uniforme: o assado e o device deixam de recusar'

# ── A CERCA do salto entre vizinhas ─────────────────────────────────────
muta "$COL/lib.rs" \
  '                if hi - k[g as usize] > tecto_de_salto {' \
  '                if false {' \
  'P6 a cerca do salto deixa de armar'

muta "$COL/lib.rs" \
  '                    k[g as usize] = hi - tecto_de_salto;' \
  '                    k[g as usize] = hi.min(k[g as usize]);' \
  'P7 a cerca DESCE a vizinha fina em vez de subir a grossa'

# ── A LEI DA AREA ───────────────────────────────────────────────────────
muta "$COL/lib.rs" \
  '            let ideal = (alvo * a.max(0.0).sqrt()).log2();' \
  '            let ideal = (alvo * 1.0f32).log2();' \
  'P8 o nivel deixa de olhar a AREA: a dispersao nao cai'

# ── O ASSADO: o ladrilho e' da FACE ─────────────────────────────────────
muta "$COL/assar.rs" \
  '        .map(|f| tinta.lado_da_face(f) + 1 + 2 * FOLGA_EM_TEXELS)' \
  '        .map(|_| tinta.lado_da_face(0) + 1 + 2 * FOLGA_EM_TEXELS)' \
  'P9 o assado assa tudo ao nivel da face 0 e perde amostras em silencio'

# ── O EMPACOTADOR: a prateleira quebra de linha ─────────────────────────
muta "$COL/assar.rs" \
  '            if x + s > w {' \
  '            if false {' \
  'P13 a prateleira nunca quebra: nada cabe e o assado recusa por tamanho'

# ── O EMPACOTADOR: a altura da prateleira e' a da MAIOR ─────────────────
# ⚠️ Com `altura = s` a prateleira encolhe para a ULTIMA peca e as de baixo
#    sobem por cima das de cima — dois ladrilhos no mesmo texel, com UV
#    perfeitamente validas e os gates de COR todos verdes.
muta "$COL/assar.rs" \
  '            altura = altura.max(s);' \
  '            altura = s;' \
  'P14 as prateleiras pisam-se: dois ladrilhos no mesmo texel'

# ⛔⛔ **O P15 SAIU, e o que ele achou foi codigo a mais.** Ele mutava um
#    `if y + s > w { return None }` por peca e SOBREVIVEU — porque a ultima
#    prateleira e' a mais funda por construcao e o teste final ja' respondia
#    por todas. A cura nao foi um gate novo: foi apagar a linha.
#    *Uma linha que a mutacao nao consegue matar nao e' lei.*

# ── O PINCEL le o lado da FACE ──────────────────────────────────────────
muta "$SCU/tinta_fina.rs" \
  'self.tinta.lado_da_face(fi as usize)' \
  'self.tinta.lado_da_face(0)' \
  'P10 o pincel volta a ler UM lado para a peca inteira'

# ── O DEVICE desarma um plano graduado ──────────────────────────────────
muta "$REN/tinta_gpu.rs" \
  '    let Some(lado) = t.lado_uniforme() else {' \
  '    let Some(lado) = Some(t.lado_da_face(0)) else {' \
  'P11 o device assume o lado da face 0 e desenha tinta no sitio errado'

# ── O CONTROLO INERTE ───────────────────────────────────────────────────
muta "$COL/enderecos.rs" \
  'pub fn total(' \
  '
pub fn total(' \
  'P12 CONTROLO: uma mutacao INERTE (uma linha em branco) nao pode sangrar'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
  exit $?
fi
echo "PLACAR DO R POR FACE: $sangram de $total sangram (o P12 e' o CONTROLO e nao pode)"
[ "$sangram" -eq $((total - 1)) ]
