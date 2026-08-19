#!/usr/bin/env bash
# adr-index.sh — regenera docs/architecture/decisions/README.md a partir dos próprios ADRs.
#
# WHY this exists: medido em 2026-08-18, `docs/architecture/decisions/` é o diretório MAIS
# citado do repo — 32 links apontam para a PASTA, e o CLAUDE.md §5/§6 citam ADRs por NÚMERO
# (`[ADR-0143](docs/architecture/decisions/)`). Um link para a pasta entrega um `ls` ao agente,
# não um caminho: ele acaba a fazer `grep`/`ls` para descobrir o nome do arquivo. Com 160 ADRs
# e 45% da documentação inalcançável a partir do roteador, esse era o pior caso do repo.
#
# ⚠️ O índice é DERIVADO, nunca escrito à mão — uma lista de 160 itens mantida à mão envelhece
# na primeira semana (precedente: o placar da conferência do Motion, que é derivado de propósito).
#
# ⚠️ O `Status:` sai do PRÓPRIO ADR. Um ADR sem essa linha aparece como `—`, e isso é um ACHADO,
# não um defeito do script: é um ADR cujo próprio texto não diz se ainda vale.
#
# ⚠️ A REVOGAÇÃO é derivada por MAPA REVERSO, e a direção é o que importa. Este repo escreve
# `- **Supersede:** ...` no ADR **NOVO** (o que revoga), nunca no revogado. Um grep por
# um grep pela palavra marca portanto o lado ERRADO: a primeira versão deste script procurou
# "superseded by" e devolveu **0 de 30** — o padrão estava invertido, e a contagem zero foi o
# que denunciou. Aqui lemos as linhas `Supersede:` e marcamos os NÚMEROS CITADOS nelas.
#
# ⚠️ E a marca diz «segundo o ADR que o cita», de propósito: o ADR-0085 supersede uma REGRA
# dentro do ADR-0049, não o ADR inteiro. O índice reporta a alegação e sua origem; julgar o
# alcance é leitura humana.
#
# USAGE
#   bash scripts/adr-index.sh            # regenera o README.md
#   bash scripts/adr-index.sh --check    # não escreve; sai !=0 se o README está desatualizado (p/ gate)
#
# Só lê os ADRs e escreve o README do diretório deles.

set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1
DIR="docs/architecture/decisions"
OUT="$DIR/README.md"
[ -d "$DIR" ] || { echo "✗ $DIR não existe"; exit 1; }

TMP=$(mktemp)
SUP=$(mktemp)
trap 'rm -f "$TMP" "$SUP"' EXIT

n_total=0; n_sem_status=0; n_super=0

# --- mapa reverso da revogação: "<alvo> <quem-o-revoga>" -----------------------
# lê as linhas `Supersede(s):` de CADA ADR e credita os números CITADOS nelas.
# ⚠️ `Supersedes: none` existe no repo (ADR-0103) e não pode virar alvo.
python3 - "$DIR" <<'PYEOF' | sort -u > "$SUP"
import sys, os, re, glob
d = sys.argv[1]
# fronteira do BLOCO: começa no bullet `- **Supersede...`, termina no PRÓXIMO bullet de topo.
# ⚠️ isto é load-bearing: o ADR-0096 tem `- **NÃO afeta:** ... ADR-0043..0053` no bullet
# SEGUINTE. Ler uma linha a mais marca como revogado exatamente o que o ADR diz preservar.
INICIO = re.compile(r'^\s*[-*]\s*\**Supersede', re.I)
FIM    = re.compile(r'^\s*[-*]\s*\*\*')
NENHUM = re.compile(r'Supersedes?:?\**\s*(none|nenhum)', re.I)
NUM    = re.compile(r'(\d{4})')
for f in sorted(glob.glob(os.path.join(d, '[0-9][0-9][0-9][0-9]-*.md'))):
    quem = os.path.basename(f)[:4]
    try: linhas = open(f, errors='ignore').read().split('\n')[:60]
    except Exception: continue
    bloco, dentro = [], False
    for ln in linhas:
        if dentro:
            if FIM.match(ln) or (ln.strip() and not ln.startswith((' ', '\t'))): break
            bloco.append(ln); continue
        if INICIO.match(ln):
            if NENHUM.search(ln): break
            dentro = True; bloco.append(ln)
    if not bloco: continue
    txt = '\n'.join(bloco)
    alvos = set()
    # intervalo `NNNN..NNNN` (o repo escreve `[ADR-0043](0043-x.md)..[ADR-0053](0053-y.md)`)
    for a, b in re.findall(r'(\d{4})[^\d]{0,60}?\.\.[^\d]{0,60}?(\d{4})', txt):
        lo, hi = int(a), int(b)
        if 0 < hi - lo <= 40: alvos.update(f'{n:04d}' for n in range(lo, hi + 1))
    alvos.update(NUM.findall(txt))
    for a in sorted(alvos):
        if a != quem: print(a, quem)
PYEOF

{
  echo "# ADRs — índice"
  echo
  echo "> ⚠️ **Gerado por \`bash scripts/adr-index.sh\` — não edite à mão.** Uma lista de 160 itens"
  echo "> mantida à mão envelhece na primeira semana; esta é derivada do cabeçalho de cada ADR."
  echo ">"
  echo "> O **estado por-módulo** vive no [\`CLAUDE.md §5\`](../../../CLAUDE.md); os **contratos"
  echo "> congelados**, no §6. Um ADR descreve a decisão **no dia em que foi tomada** — use-o para"
  echo "> responder *\"por que isto ficou assim?\"*, nunca para decidir a próxima ação."
  echo
  echo "| # | Status | Decisão |"
  echo "|---|---|---|"
} > "$TMP"

for f in "$DIR"/[0-9][0-9][0-9][0-9]-*.md; do
  [ -f "$f" ] || continue
  base=$(basename "$f")
  num=${base%%-*}
  n_total=$((n_total + 1))

  # título: primeira linha `# ...`, sem o prefixo `# ADR-NNNN — ` / `: `
  titulo=$(grep -m1 '^# ' "$f" 2>/dev/null \
           | sed -E 's/^# +//; s/^ADR-[0-9]+ *[—:-] *//' )
  [ -z "$titulo" ] && titulo="(sem título)"

  # status: a linha `- **Status:** ...` do próprio ADR, nas primeiras 15 linhas
  status=$(head -15 "$f" | grep -m1 -oE '\*\*Status:?\*\*:? *[^·|]*' \
           | sed -E 's/\*\*Status:?\*\*:? *//; s/ *$//')
  if [ -z "$status" ]; then status="—"; n_sem_status=$((n_sem_status + 1)); fi

  # ⚠️ a célula tem de caber numa tabela: vários `Status:` deste repo continuam em
  # narrativa longa (o maior tinha 495 caracteres e virava um parágrafo dentro da célula).
  # Corta na primeira sentença e sinaliza o corte — o texto inteiro está no ADR.
  status=$(printf '%s' "$status" | sed -E 's/([.;] ).*/\1…/' )
  [ "${#status}" -gt 90 ] && status="$(printf '%.87s' "$status")…"

  # revogação: pelo MAPA REVERSO, e dizendo QUEM alega
  por=$(awk -v a="$num" '$1==a {printf "%s%s", (c++?", ":""), $2} END{print ""}' "$SUP")
  if [ -n "$por" ]; then
    status="⛔ superseded por $por · $status"
    n_super=$((n_super + 1))
  fi

  # escapa a barra vertical, que partiria a célula da tabela
  titulo=${titulo//|/\\|}
  printf '| [%s](%s) | %s | %s |\n' "$num" "$base" "$status" "$titulo" >> "$TMP"
done

{
  echo
  echo "---"
  echo
  printf '**%d ADRs** · **%d** marcados ⛔ · **%d** sem linha `Status:` no próprio texto.\n' \
    "$n_total" "$n_super" "$n_sem_status"
  echo
  echo "⚠️ **⛔ diz «o ADR NNNN alega supersedê-lo»**, e a alegação pode ser PARCIAL: o ADR-0085"
  echo "supersede uma *regra* dentro do ADR-0049, não o ADR inteiro. O índice reporta a alegação"
  echo "e a sua origem — julgar o alcance é leitura humana."
  echo
  echo "⚠️ Um ADR sem \`Status:\` é um **achado**, não um defeito deste índice: é um ADR cujo"
  echo "próprio texto não diz se ainda vale. O índice mostra o que existe, não o que devia existir."
} >> "$TMP"

if [ "${1:-}" = "--check" ]; then
  if diff -q "$TMP" "$OUT" >/dev/null 2>&1; then
    echo "✓ $OUT está em dia ($n_total ADRs)"; exit 0
  fi
  echo "✗ $OUT desatualizado — rode: bash scripts/adr-index.sh"; exit 1
fi

cp "$TMP" "$OUT"
echo "✓ $OUT regenerado — $n_total ADRs · $n_super superseded · $n_sem_status sem Status"
