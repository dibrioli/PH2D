#!/usr/bin/env bash
# muta_o_balao_do_aviso.sh — prova de mutacao da cura do report «as mensagens
# estao cortadas com … nao consigo ler tudo» (dono, 22/09).
#
# O que ela defende: o aviso da exportacao CABE no balao, medido pela regua do
# produto (`toast::text_budget_px` + `text_elide::fit`), no PIOR caso.
#
# ⚠️ O arnes CONTROLA-SE A SI MESMO nos quatro pontos de sempre (ancora unica ·
# a mutacao compila · N > 0 testes correram · a corrida limpa VERDE) e tem o
# PRE-VOO (`MUTA_SO_ANCORAS=1`).
#
# ⚠️ A populacao sao DUAS crates: a lei partilhada da frase vive na `ph2d-mesh`,
# a regua e a porta na `ph2d-editor-core`, e o GATE na `ph2d-app-sculpt3d`.
# A populacao e' de quem OBSERVA — e quem observa e' o gate.
#
# ⛔ Chame-o sempre pela porta de recursos:
#   PH2D_PRAZO=2400 bash scripts/ph2d-run.sh bash docs/3D/ferramentas/muta_o_balao_do_aviso.sh
set -u
SO_ANCORAS="${MUTA_SO_ANCORAS:-}"
MESH=crates/ph2d-mesh/src
EC=crates/ph2d-editor-core/src
BK=$(mktemp -d)
cp -r "$MESH" "$BK/mesh"
cp -r "$EC" "$BK/ec"
restore() {
  rm -rf "$MESH"; cp -r "$BK/mesh" "$MESH"
  rm -rf "$EC"; cp -r "$BK/ec" "$EC"
  # ⚠️ `cp -r` devolve o mtime ANTIGO e o cargo guarda o build DA MUTACAO.
  find "$MESH" "$EC" -name '*.rs' -exec touch {} +
}
trap restore EXIT

corrida() { cargo nextest run -p ph2d-app-sculpt3d -p ph2d-editor-core -E 'test(/balao|toast/)' 2>&1; }
populacao() { grep -oP '\K[0-9]+(?= tests? run)' | awk '{s+=$1}END{print s+0}'; }

if [ -z "$SO_ANCORAS" ]; then
  limpa=$(corrida); rc_limpo=$?
  verde=$(printf '%s' "$limpa" | populacao)
  echo "VERDE antes: $verde testes correram"
  [ "${verde:-0}" -gt 0 ] || { echo "ABORTO: a corrida limpa nao correu teste nenhum"; exit 2; }
  if [ "$rc_limpo" -ne 0 ]; then
    echo "ABORTO: a corrida limpa esta' VERMELHA -- um placar tirado daqui e' fabricado."
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
  else
    corridos=$(printf '%s' "$out" | populacao)
    if [ "$corridos" -eq 0 ]; then
      echo "  ABORTO [$nome]: zero testes correram — ultimas linhas:"
      printf '%s' "$out" | tail -5 | sed 's/^/      | /'
    elif [ $rc -ne 0 ]; then echo "  SANGRA  [$nome]"; sangram=$((sangram+1))
    else echo "  SOBREVIVE [$nome]  <<<<"; fi
  fi
  restore
}

# ── A FRASE: a EXPLICACAO volta, e o pior caso passa de 45 para 68 ───────
muta "$MESH/read.rs" \
  '        lost.push("fine paint");' \
  '        lost.push("fine paint (mesh resolution only)");' \
  'B1 a explicacao volta a frase: o STL passa a 68 caracteres'

# ── O PREFIXO: `Lost:` volta a `not carried:`, e o pior caso vai a 50 ────
muta "$MESH/read.rs" \
  '    format!("Lost: {}", lost.join(", "))' \
  '    format!("not carried: {}", lost.join(", "))' \
  'B2 o prefixo longo volta: o pior caso vai a 50'

# ── A PORTA: o orcamento mente e devolve a coluna INTEIRA ────────────────
# ⚠️ Ela nao sangra pelo gate do aviso (com mais espaco tudo cabe): quem a
#    apanha e' o `debug_assert` do pintor, que compara as duas contas.
# ⚠️ A ancora foi RE-APONTADA em 22/09: o `cargo fmt` partiu a expressao em cinco
#    linhas e ela passou a casar ZERO — apanhada pelo PRE-VOO, em segundos e sem
#    correr um teste. *Uma ancora que casa zero le-se num placar como sobreviveu.*
muta "$EC/toast.rs" \
  '        - Spacing::Xl.px() * 2.0' \
  '        - Spacing::Xl.px() * 0.0' \
  'B3 a porta do orcamento diverge da conta do pintor'

# ── O CONTROLO do proprio gate: o orcamento vai a ZERO ───────────────────
muta "$EC/progress.rs" \
  'const COLUMN_W: f32 = 360.0;' \
  'const COLUMN_W: f32 = 60.0;' \
  'B4 a coluna encolhe: o CONTROLO do gate tem de disparar'

# ── O CONTROLO INERTE ────────────────────────────────────────────────────
muta "$MESH/read.rs" \
  '#[must_use]
pub fn lost_by(' \
  '#[must_use]

pub fn lost_by(' \
  'B5 CONTROLO: uma mutacao INERTE (uma linha em branco) nao pode sangrar'

echo
if [ -n "$SO_ANCORAS" ]; then
  echo "PRE-VOO: $sangram de $total ancoras casam exactamente uma vez (ZERO testes corridos)"
  [ "$sangram" -eq "$total" ]
  exit $?
fi
echo "PLACAR DO BALAO: $sangram de $total sangram (o B5 e' o CONTROLO e nao pode)"
[ "$sangram" -eq $((total - 1)) ]
