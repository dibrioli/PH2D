#!/usr/bin/env bash
# cleanroom-sweep-controlo.sh — **o CONTROLO POSITIVO do `cleanroom-sweep.sh`.**
#
# Uso:  bash scripts/cleanroom-sweep-controlo.sh
# Exit: 0 = o instrumento discrimina · 1 = um canal ficou CEGO (ou há falso positivo)
#
# ⛔⛔⛔ **Um sweep verde cujo controlo ninguém correu não é prova de filtragem —
# é prova de que o instrumento não foi apontado a nada.** Esta casa mediu isso
# três vezes, e as três vezes o canal cego foi descoberto por acidente:
#
#   | data  | canal que estava cego                        | quem o achou |
#   |-------|----------------------------------------------|--------------|
#   | 14/09 | conteúdo de um `.gz` (o `strings` vê lixo)   | R-pré        |
#   | 16/09 | dentro do `.gz`, com a frase DOBRADA          | R-pré, 2.ª   |
#   | 16/09 | frase dobrada entre duas linhas de COMENTÁRIO | R-pré, 3.ª   |
#
# ⚠️ As três são a MESMA doença: *uma cura que abre um caminho novo herda as
# cegueiras que o caminho antigo já tinha pago*. Este ficheiro existe para que a
# quarta seja descoberta por um teste e não por um auditor.
#
# ⛔ **O canário é uma frase INVENTADA aqui.** Ele não é, e nunca pode ser,
# expressão de alvo nenhum: este script corre em qualquer árvore, sem vassoura
# real e sem tocar em nada do alvo.
#
# ⚠️ **A armadilha do canal DOBRADO, medida 2026-09-16:** um canário cuja 2.ª
# linha REPETE o marcador de comentário lê **limpo** — isso é a isenção
# simétrica do normalizador, não o canal. *Um canário assim testa a isenção e
# devolve verde.* O canal 5 abaixo é escrito sem o marcador na 2.ª linha de
# propósito, e o canal 6 é a forma que mordeu na 3.ª passagem.
set -u
cd "$(dirname "$0")/.." || exit 2
# ⭐ **O CONTROLO DO CONTROLO.** `PH2D_SWEEP=<path>` aponta-o a outra versão do
# instrumento — é assim que se prova que ele DISCRIMINA em vez de dizer sempre
# que sim. MEDIDO em 2026-09-16 contra as versões anteriores desta mesma árvore
# (`git show <commit>:scripts/cleanroom-sweep.sh`):
#
#   | versão do sweep                                   | canais que reprovam    |
#   |---------------------------------------------------|------------------------|
#   | antes de 14/09 — o `.gz` era opaco                | 4 · 5 · 6 · 7 · 7b · 8 |
#   | 14/09 — o `.gz` abriu                             | 5 · 6 · 7 · 7b · 8     |
#   | 15/09 — a passagem plana deixa de ser cega ao `_` | 6 · 7 · 7b · 8         |
#   | 16/09 — a desdobrada ganha dois chamadores        | 6 · 7 · 8              |
#   | esta                                              | **nenhum**             |
#
# ⚠️ **A 1.ª redacção desta tabela foi escrita DE MEMÓRIA e a medição
# desmentiu-a nas três linhas** — e ao medi-la apareceu um buraco no próprio
# controlo: **o canal `7b` não existia**, e sem ele as duas últimas linhas eram
# IDÊNTICAS, ou seja *o controlo não vigiava a cura mais recente de todas*.
#
# ⇒ *cada linha desta tabela é uma passagem de R-pré que gastou uma volta a
# descobrir à mão o que uma corrida deste script diria em dois segundos.*
SWEEP="${PH2D_SWEEP:-scripts/cleanroom-sweep.sh}"
[ -f "$SWEEP" ] || { echo "✗ nao encontrei $SWEEP" >&2; exit 2; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
CANARIO='CANARIO DE PAREDE FRASE INVENTADA PARA CONTROLO'
printf '%s\n' "$(printf '%s' "$CANARIO" | base64 -w0)" > "$TMP/vassoura.txt"

falhas=0
canal() { # $1 = rótulo · $2 = path · $3 = exit esperado
  local rc
  bash "$SWEEP" "$TMP/vassoura.txt" "$2" >/dev/null 2>&1
  rc=$?
  if [ "$rc" = "$3" ]; then
    printf '  ✓ %-52s exit=%s\n' "$1" "$rc"
  else
    printf '  ✗ %-52s exit=%s (esperado %s)\n' "$1" "$rc" "$3"
    falhas=$((falhas + 1))
  fi
}

# 1 — NOME de ficheiro
mkdir -p "$TMP/c1/$CANARIO" && : > "$TMP/c1/$CANARIO/x.txt"
# 2 — conteúdo de texto, numa linha
mkdir -p "$TMP/c2" && printf 'prosa %s e segue\n' "$CANARIO" > "$TMP/c2/a.md"
# 3 — conteúdo de BINÁRIO (a rota do `strings`)
mkdir -p "$TMP/c3" && { printf '\001\002\003'; printf '%s' "$CANARIO"; printf '\004'; } > "$TMP/c3/a.bin"
# 4 — dentro de um `.gz`, numa linha. ⚠️ O nome NÃO diz `.gz`: a detecção é
#     pelos dois bytes mágicos, e um corpus renomeado escaparia a um sufixo.
mkdir -p "$TMP/c4" && printf '%s\n' "$CANARIO" | gzip -c > "$TMP/c4/fixtura.dat"
# 5 — DOBRADO em duas linhas, com ênfase pelo meio (o canal de 16/09)
mkdir -p "$TMP/c5" && printf 'CANARIO DE **PAREDE** FRASE\nINVENTADA PARA CONTROLO\n' > "$TMP/c5/a.md"
# 6 — DOBRADO entre duas linhas de COMENTÁRIO (a forma de um cabeçalho de
#     proveniência de fixtura — que é onde uma frase do alvo entraria)
mkdir -p "$TMP/c6" && printf '# fixtura\n# CANARIO DE PAREDE FRASE\n# INVENTADA PARA CONTROLO\n' > "$TMP/c6/a.txt"
# 7 — o canal 6 COMPRIMIDO: os dois ramos têm de concordar
mkdir -p "$TMP/c7" && gzip -c < "$TMP/c6/a.txt" > "$TMP/c7/fixtura.dat"
# 7b — DOBRADO dentro de um `.gz` **sem marcador nenhum**. ⚠️ Sem este canal o
#      controlo não distingue a versão que deu ao ramo do `.gz` a passagem
#      desdobrada da que não deu — medido: as duas reprovavam exactamente os
#      mesmos canais. *Um controlo que não separa duas curas não vigia a
#      primeira delas.*
mkdir -p "$TMP/c7b" && printf 'CANARIO DE **PAREDE** FRASE\nINVENTADA PARA CONTROLO\n' \
  | gzip -c > "$TMP/c7b/fixtura.dat"
# 8 — marcador de outra família (`//`)
mkdir -p "$TMP/c8" && printf '// CANARIO DE PAREDE FRASE\n// INVENTADA PARA CONTROLO\n' > "$TMP/c8/a.rs"
# 9 — o CONTROLO NEGATIVO: sem canário nenhum, tem de ficar limpo
mkdir -p "$TMP/c9" && printf '# fixtura\n0.5 0.25 0.125\n' > "$TMP/c9/a.txt" \
  && gzip -c < "$TMP/c9/a.txt" > "$TMP/c9/b.dat"

echo "controlo do cleanroom-sweep.sh — cada canal tem de ACUSAR (exit 1):"
canal "1 · NOME de ficheiro"                          "$TMP/c1" 1
canal "2 · conteúdo de texto, numa linha"             "$TMP/c2" 1
canal "3 · conteúdo de BINÁRIO"                       "$TMP/c3" 1
canal "4 · dentro de um .gz (nome sem sufixo)"        "$TMP/c4" 1
canal "5 · DOBRADO, com ênfase pelo meio"             "$TMP/c5" 1
canal "6 · DOBRADO entre linhas de COMENTÁRIO"        "$TMP/c6" 1
canal "7 · o canal 6, COMPRIMIDO"                     "$TMP/c7" 1
canal "7b · DOBRADO no .gz, sem marcador"             "$TMP/c7b" 1
canal "8 · marcador de outra família (//)"            "$TMP/c8" 1
echo "e o controlo NEGATIVO tem de ficar limpo (exit 0):"
canal "9 · corpus sem canário"                        "$TMP/c9" 0

# ⛔ O uso errado tem código PRÓPRIO, e um laço que o colapse em «achado» faz
# perder uma passagem inteira a caçar um falso vermelho (medido, R-pré 2.ª).
echo "e o uso errado tem de se distinguir do achado (exit 2):"
canal "10 · vassoura inexistente"                     "$TMP/nao-existe" 2

if [ "$falhas" -eq 0 ]; then
  echo "✓ o instrumento discrimina em todos os canais"
  exit 0
fi
echo "✗ $falhas canal(is) do sweep nao discriminam — NAO confie num sweep verde ate' curar" >&2
exit 1
