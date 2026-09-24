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
MSH=crates/ph2d-mesh/src
APP=crates/ph2d-app-sculpt3d/src
BK=$(mktemp -d)
cp -r "$COL" "$BK/col"; cp -r "$REN" "$BK/ren"; cp -r "$SCU" "$BK/scu"; cp -r "$MSH" "$BK/msh"; cp -r "$APP" "$BK/app"
restore() {
  rm -rf "$COL"; cp -r "$BK/col" "$COL"
  rm -rf "$REN"; cp -r "$BK/ren" "$REN"
  rm -rf "$SCU"; cp -r "$BK/scu" "$SCU"
  rm -rf "$MSH"; cp -r "$BK/msh" "$MSH"
  rm -rf "$APP"; cp -r "$BK/app" "$APP"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  # ⚠️ `-name '*.rs'` deixava o `.wgsl` com o mtime ANTIGO — e desde 23/09 ha'
  #    mutacoes no gemeo. O `include_str!` do censo depende do mtime dele.
  find "$COL" "$REN" "$SCU" "$MSH" "$APP" \( -name '*.rs' -o -name '*.wgsl' \) -exec touch {} +
}
trap restore EXIT

FILTRO='test(/p2_tests|assar_tests|esta_ligada_nos|nivel_base|canto_de_uma_face|dois_lados_de_uma_aresta|ponto_de_uma_face|recusa_nomeia|graduado|payload|igualac|igualada|igualar|abre_uniforme|desarma|uniformiz|v2_abre|area_por_face/)'
corrida() {
  cargo nextest run -p ph2d-mesh -p ph2d-mesh-colors -p ph2d-mesh-render -p ph2d-app-sculpt3d -E "$FILTRO" 2>&1
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


# ── A LEI DA AREA ───────────────────────────────────────────────────────

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

# ── O DEVICE DESARMA um plano graduado ──────────────────────────────────
# ⭐⭐ A `P11` VOLTOU a' ancora onde nasceu. Ate' 23/09 ela mutava a guarda
#    `lado_uniforme` que desarmava o device; de 23/09 a 24/09 o registo tinha
#    `19` palavras, a guarda nao existia e ela mediu a soma das amostras das
#    arestas; em 24/09 o registo voltou a `10` por ordem do dono e a guarda
#    VOLTOU. *Uma ancora pode mudar de especie duas vezes, e o pre-voo e' que
#    diz quando.* A mutacao desenha um plano graduado com o lado da face `0` —
#    a tinta de umas faces no sitio das outras, sem nada no ecra a acusar.
muta "$REN/tinta_gpu.rs" \
  '    let Some(lado) = t.lado_uniforme() else {' \
  '    let Some(lado) = Some(t.lado_da_face(0)) else {' \
  'P11 a placa desenha um plano GRADUADO com o lado da face 0 em vez de desarmar'

# ⛔⛔ AQUI VIVIAM A P16, A P17, A P18, A P19 E A P20 — as nove palavras do
#    registo por face (o lado da face, o inicio e o lado de cada aresta) e o
#    passo do subconjunto no gemeo. Sairam em 2026-09-24 COM a lei, por ordem do
#    dono: *liberar a memoria que o `Even Detail` deixou reservada*.

# ── A CONVERSAO de um plano graduado de um ficheiro antigo ──────────────
# ⭐⭐⭐ O que as substitui: nenhum plano graduado chega a' placa pelo produto,
#    porque o carregador o CONVERTE ao abrir. As cinco maneiras de a partir.
muta "$APP/doc.rs" \
  '    if t.lado_uniforme().is_none() {
        t = t
            .uniformizada(faces())
            .ok_or(SculptDocError::Tinta { peca, esperadas })?;
    }' \
  '' \
  'U1 o carregador deixa de converter: o plano abre graduado e a placa desarma-o'

muta "$APP/doc.rs" \
  '            .uniformizada(faces())' \
  '            .uniformizada(faces())
            .map(|_| ph2d_mesh_colors::Tinta::semeada(t.plano_por_vertice(), faces(), t.nivel()))' \
  'U2 o carregador RE-SEMEIA da cor por vertice em vez de LER o plano gravado'

muta "$COL/uniformiza.rs" \
  '        ordem.sort_by_key(|&f| (topo.nivel_de(f), f));' \
  '        ordem.sort_by_key(|&f| (std::cmp::Reverse(topo.nivel_de(f)), f));' \
  'U3 a face GROSSA escreve por ultimo e interpola por cima da amostra verdadeira da aresta'

muta "$COL/uniformiza.rs" \
  '        if lista.len() != topo.faces() {
            return None;
        }' \
  '' \
  'U4 uma lista com faces a menos passa a ser convertida'

muta "$COL/uniformiza.rs" \
  '            .any(|(f, c)| crate::cantos(c) != topo.cantos_de(f))' \
  '            .any(|(f, c)| false && crate::cantos(c) != topo.cantos_de(f))' \
  'U5 um quad lido como triangulo passa a ser convertido'

# ⛔⛔ AQUI VIVIAM AS TRES ANCORAS DO ESCOLHEDOR (a mediana · o piso · o
#    interruptor lido pelo `garante`), e elas sairam com ele por ordem do dono
#    (2026-09-23). O que fica e' a `P26`, que mede o que o LEITOR DE FICHEIROS
#    ainda alcanca: um plano graduado gravado antes da retirada tem de abrir.

# ⛔⛔ E AQUI VIVIAM AS ANCORAS DA LEI POR AREA (a cerca do salto, o nivel
#    derivado da area, e a propria `face_areas`): as tres sairam em 2026-09-23
#    com a `niveis_por_area`, que ficou sem consumidor quando o dono retirou o
#    `Even Detail`. O que fica mede o PLANO, que o leitor de ficheiros alcanca.

# ⛔⛔ O QUE O PLANO GUARDA COMO PEDIDO — e ele NAO e' o nivel mais fino.
# ⚠️ Esta ancora sobreviveu a' retirada do `Even Detail` de proposito: quem
#    alcanca a `graduada` hoje e' o LEITOR DE FICHEIROS, e um plano gravado
#    antes de 23/09 tem de abrir com o degrau que ele pediu. Confundir os dois
#    reconstroi o plano em TODO quadro e a tinta fina some-se a 60 Hz.
muta "$COL/lib.rs" \
  '            nivel: pedido,' \
  '            nivel: topo.nivel_mais_fino(),' \
  'P26 o plano guarda o nivel MAIS FINO em vez do que foi PEDIDO'

# ── E o PLANO GRADUADO atravessa o FICHEIRO ─────────────────────────────
muta "$APP/doc.rs" \
  '                    niveis: if t.lado_uniforme().is_some() {' \
  '                    niveis: if true {' \
  'P24 o documento grava um plano graduado como se fosse uniforme'

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
