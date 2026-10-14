#!/usr/bin/env bash
# Aviso de conversa grande — corre a cada pedido do Enio (UserPromptSubmit).
#
# POR QUE (CLAUDE.md §2, «o custo é RELER»): cada passo relê a conversa inteira. Medido em 14/10
# (40 sessões): o 1.º passo de uma janela nova custa ~63 mil tokens e a média por passo foi 394 mil
# (alvo ≤ 250 mil); em 7 dias, 80 de ~900 sessões passaram de 500 mil. A regra «uma janela nova por
# onda de trabalho» existia e não chegava a quem decide abrir a janela — o Enio. Este aviso chega.
#
# ⚠️ AVISA, não resume: encurtar a conversa à força (o auto-compact mais cedo) perde detalhe a meio
# de uma tarefa — custo de qualidade que o dono recusou. Quem decide é ele, entre um assunto e outro.
#
# Avisa ao passar de PH2D_CONVERSA_AVISO (300 mil) e depois a cada PH2D_CONVERSA_PASSO (150 mil), não
# em todo pedido. Falha ABERTA: qualquer surpresa ⇒ silêncio e o pedido segue.
set -uo pipefail

entrada="$(cat 2>/dev/null)" || exit 0
command -v jq >/dev/null 2>&1 || exit 0
tp="$(printf '%s' "$entrada" | jq -r '.transcript_path // empty' 2>/dev/null)" || exit 0
sid="$(printf '%s' "$entrada" | jq -r '.session_id // "x"' 2>/dev/null | tr -c 'A-Za-z0-9-' '_')"
[ -n "$tp" ] && [ -f "$tp" ] || exit 0

# O contexto do ÚLTIMO passo: o que a próxima resposta vai reler.
ctx="$(tail -n 400 "$tp" 2>/dev/null | tac | jq -Rr 'fromjson? | .message.usage? // empty
  | ((.input_tokens // 0) + (.cache_read_input_tokens // 0) + (.cache_creation_input_tokens // 0))' \
  2>/dev/null | head -1)"
case "$ctx" in ''|*[!0-9]*) exit 0 ;; esac

limiar="${PH2D_CONVERSA_AVISO:-300000}"
passo="${PH2D_CONVERSA_PASSO:-150000}"
[ "$ctx" -ge "$limiar" ] || exit 0
degrau=$(( (ctx - limiar) / passo ))
marca="${XDG_RUNTIME_DIR:-/tmp}/ph2d-conversa-${sid}"
ultimo="$(cat "$marca" 2>/dev/null || echo -1)"
case "$ultimo" in ''|*[!0-9-]*) ultimo=-1 ;; esac
[ "$degrau" -gt "$ultimo" ] || exit 0
echo "$degrau" > "$marca" 2>/dev/null

mil=$(( ctx / 1000 ))
vezes=$(( (ctx + 32000) / 63000 ))
jq -n --arg u "📏 Esta conversa já está grande: cada passo relê ~${mil} mil tokens, ~${vezes}× o de uma conversa nova. Se o próximo pedido é um ASSUNTO NOVO, abra uma conversa nova — fica mais rápido e mais barato. Para continuar o mesmo assunto, siga aqui." \
      --arg m "Contexto desta janela: ~${mil} mil tokens por passo (CLAUDE.md §2: alvo ≤ 250 mil; janela nova ~63 mil). Delegue leitura e verificação aos agentes (explorador, verificador) para não crescer mais. Se este pedido abre outra onda de trabalho, diga ao Enio numa linha, sem jargão, que uma conversa nova sai mais barata, e deixe o estado escrito (handoff/commit) antes." \
  '{systemMessage:$u, hookSpecificOutput:{hookEventName:"UserPromptSubmit", additionalContext:$m}}' 2>/dev/null
exit 0
