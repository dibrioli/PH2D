#!/usr/bin/env bash
# Provas de mutação dos DOIS defeitos do report do dono de 2026-09-19:
#
#   D1  «Se coloco o tamanho, para de animar»   → o tamanho absoluto é a BASE que o grafo MODULA
#   D2  «Gap y quebrou e movimenta tudo         → a amostra do gizmo VARRE a nuvem em vez de ser
#        em vez de criar espaço»                   um PREFIXO na borda dela
#
# Cada bloco: MUTA o produto → corre O GATE QUE DEVE MORRER → restaura.
# ⚠️ `touch` no fim de cada restauro (o `cp` devolve mtime antigo e o cargo serve o build MUTADO).
# ⚠️⚠️ **CONTROLO sobre o próprio FILTRO** — um filtro que casa ZERO testes sai VERDE e lê-se
#    exactamente como «SOBREVIVEU».
# ⚠️ Toda troca é por `muta`, que ABORTA se a âncora não aparecer o número esperado de vezes.
#
# Corra-o pela porta de recursos:
#   bash scripts/ph2d-run.sh bash "docs/Motion Nodes/ferramentas/mutacao_gizmo_2026-09-19.sh"
set -u
cd "$(dirname "$0")/../../.." || exit 1

GIZ=crates/ph2d-app-motion/src/ponto_gizmo.rs
OVL=crates/ph2d-app-motion/src/ponto_gizmo_overlay.rs

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

prova() { # nome filtro
  TOTAL=$((TOTAL+1))
  echo "── $1"
  local out rc corridos
  out=$(cargo test -p ph2d-app-motion --lib -- "$2" 2>&1); rc=$?
  corridos=$(printf '%s' "$out" | grep -oE 'running [0-9]+ tests?' | grep -oE '[0-9]+' \
             | awk '{s+=$1} END {print s+0}')
  if [ "$corridos" -lt 1 ]; then
    echo "  ⛔ FILTRO VAZIO — '$2' nao casou teste nenhum (ou nao compilou):"
    printf '%s\n' "$out" | grep -E '^error' | head -3
    FALHAS=$((FALHAS+1)); return
  fi
  if [ "$rc" = 0 ]; then
    echo "  ⛔⛔ SOBREVIVEU — os $corridos teste(s) de '$2' passaram sobre o produto MUTADO"
    FALHAS=$((FALHAS+1))
  else
    echo "  ✅ sangrou (de $corridos teste(s) corridos)"
  fi
}

bloco() { # nome filtro ficheiro vezes antigo novo
  guarda "$3"
  if muta "$3" "$4" "$5" "$6"; then prova "$1" "$2"; else FALHAS=$((FALHAS+1)); fi
  restaura "$3"
}

echo "=== D1 — o tamanho absoluto e' a BASE que o grafo MODULA ==="

bloco "a pegada absoluta volta a IGNORAR a escala do grafo (a lei ANTIGA)" \
  "o_tamanho_absoluto_e_a_base" "$OVL" 1 \
  "    f64::from(tamanho_px) * s.abs()" \
  "    let _ = s; f64::from(tamanho_px)"

bloco "a mesma, medida pela ROTA (o grafo a animar a coluna size)" \
  "com_o_tamanho_posto_o_glifo_ainda_anima" "$OVL" 1 \
  "    f64::from(tamanho_px) * s.abs()" \
  "    let _ = s; f64::from(tamanho_px)"

# ⚠⚠ A 1.ª redação desta mutava para `s.abs() * 40.0` e **SOBREVIVEU** — porque `40` é
# exactamente o número que a fixtura daquele gate pede. *Uma mutação cuja constante coincide com a
# da fixtura é invisível por construção*; hoje ela usa um número que não está em fixtura nenhuma.
bloco "o numero do artista deixa de mandar na MAGNITUDE" \
  "o_tamanho_absoluto_e_a_base" "$OVL" 1 \
  "    f64::from(tamanho_px) * s.abs()" \
  "    let _ = tamanho_px; s.abs() * 99.0"

bloco "a mesma, medida pelo DOBRO no cartao (a metade (d) da rota)" \
  "com_o_tamanho_posto_o_glifo_ainda_anima" "$OVL" 1 \
  "    f64::from(tamanho_px) * s.abs()" \
  "    let _ = tamanho_px; s.abs() * 99.0"

bloco "o fecho peg volta a fazer o tamanho GANHAR da peca" \
  "com_o_tamanho_posto_o_glifo_ainda_anima" "$OVL" 1 \
  "            g.tamanho_em(i).map_or_else(
                || pegada_px(escala, altura_da_area),
                |t| pegada_absoluta(t, escala),
            )" \
  "            g.tamanho_em(i)
                .map_or_else(|| pegada_px(escala, altura_da_area), f64::from)"

echo
echo "=== D2 — a amostra VARRE a nuvem, e nao e' um prefixo ==="

bloco "a nuvem volta a cortar-se por PREFIXO (o defeito do report)" \
  "a_amostra_varre_a_nuvem" "$GIZ" 1 \
  "            Feicao::Ponto => Self { alcance: total, n }," \
  "            Feicao::Ponto => Self { alcance: n, n },"

bloco "o indice ignora o ALCANCE e volta a ser k" \
  "a_amostra_varre_a_nuvem" "$GIZ" 1 \
  "        k * self.alcance / self.n" \
  "        let _ = self.alcance; k"

bloco "a CADEIA passa a saltar (a topologia parte-se)" \
  "uma_cadeia_corta_se_por_prefixo" "$GIZ" 1 \
  "            Feicao::Osso | Feicao::Corda => Self { alcance: n, n }," \
  "            Feicao::Osso | Feicao::Corda => Self { alcance: total, n },"

bloco "UMA coluna colhe por outros indices (o vector paralelo desalinha)" \
  "toda_coluna_le_pelos_mesmos_indices" "$GIZ" 1 \
  "pub(crate) fn escalas(s: &Stream, am: &Amostra) -> Option<Vec<f32>> {" \
  "pub(crate) fn escalas(s: &Stream, _am: &Amostra) -> Option<Vec<f32>> {
    let am = &Amostra::inteira(s.count());"

# ⛔⛔ A mutação que esta linha tinha — `resolve` a pedir a amostra de uma NUVEM para uma cadeia —
# **SOBREVIVEU aos 30 testes do gizmo**, e a cura não foi um gate a mais: a `Amostra::de` passou a
# receber o `Stream` e a DERIVAR a feição, logo aquela mutação é hoje **inexprimível**. O que fica
# é a mesma lei mutada DENTRO da porta, onde ela ainda pode ser escrita.
bloco "a porta deixa de derivar a feicao da corrente" \
  "uma_cadeia_corta_se_por_prefixo" "$GIZ" 1 \
  "        let feicao = feicao_de(s);" \
  "        let feicao = Feicao::Ponto;"

echo
echo "── TOTAL: $TOTAL provas · $FALHAS falha(s)"
[ "$FALHAS" = 0 ]
