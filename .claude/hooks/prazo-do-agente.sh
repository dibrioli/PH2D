#!/usr/bin/env bash
# Prazo IMPOSTO de cada subagente (item I da comissão de velocidade, 29/10). Medido desde 28/10: 29 dos 179
# subagentes (16 %) passaram dos 45 min escritos no briefing e somaram 61,5 h (67 % das horas de agente) —
# o pior 7,2 h, 6,5 h num laço de espera. A regra escrita não bastou; este hook fá-la valer:
#   SubagentStart — grava o início do agente (por `agent_id`);
#   PostToolUse   — passados PH2D_AGENTE_AVISO_MIN (40), lembra o agente a cada chamada: «devolva onde chegou»;
#   PreToolUse    — (Bash) um comando pesado leva `PH2D_PRAZO` ≤ o tempo que falta (o `ph2d-run.sh` corta-o
#                   no prazo); passados PH2D_AGENTE_PRAZO_MIN (45), recusa todo comando pesado;
#                   o pesado DE VERDADE (regerar presets, bancada) nunca corre num subagente: vai à fila da
#                   onda (`scripts/fila-pesada.sh`), que o coordenador corre 1× com a régua congelada.
# A janela principal (sem `agent_id`) não é tocada. Sem jq ou sem `agent_id`: deixa passar.
command -v jq >/dev/null 2>&1 || exit 0
inp="$(cat)"
agente="$(printf '%s' "$inp" | jq -r '.agent_id // empty' 2>/dev/null)"
[ -n "$agente" ] || exit 0
evento="$(printf '%s' "$inp" | jq -r '.hook_event_name // empty')"
dir="${PH2D_AGENTES_DIR:-$(git -C "${CLAUDE_PROJECT_DIR:-.}" rev-parse --git-common-dir 2>/dev/null || echo /tmp)/ph2d-agentes}"
mkdir -p "$dir" 2>/dev/null
f="$dir/$agente"
agora="${PH2D_AGORA:-$(date +%s)}"
prazo_min="${PH2D_AGENTE_PRAZO_MIN:-45}"
aviso_min="${PH2D_AGENTE_AVISO_MIN:-40}"

if [ "$evento" = "SubagentStart" ]; then
  echo "$agora" >"$f"
  jq -n --arg m "Prazo deste agente: ${prazo_min} min (aviso aos ${aviso_min}). Passado o prazo os comandos pesados são recusados: pare e devolva «onde cheguei» e o que falta." \
    '{hookSpecificOutput:{hookEventName:"SubagentStart",additionalContext:$m}}'
  exit 0
fi
[ -f "$f" ] || echo "$agora" >"$f"   # agente nascido antes do hook: conta a partir da 1.ª chamada
inicio="$(cat "$f" 2>/dev/null || echo "$agora")"
gasto=$(( (agora - inicio) / 60 ))
resta_s=$(( prazo_min * 60 - (agora - inicio) ))

if [ "$evento" = "PostToolUse" ]; then
  [ "$gasto" -ge "$aviso_min" ] || exit 0
  jq -n --arg m "⏱ ${gasto} min de ${prazo_min}: termine o passo em curso e devolva «onde cheguei» + o que falta. Não comece nada pesado." \
    '{hookSpecificOutput:{hookEventName:"PostToolUse",additionalContext:$m}}'
  exit 0
fi

[ "$evento" = "PreToolUse" ] || exit 0
[ "$(printf '%s' "$inp" | jq -r '.tool_name // empty')" = "Bash" ] || exit 0
cmd="$(printf '%s' "$inp" | jq -r '.tool_input.command // empty')"
nega() { jq -n --arg r "$1" '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:$r}}'; exit 0; }

if printf '%s' "$cmd" | grep -qE 'PH2D_REGERA_|bancada'; then
  nega "⛔ O pesado de verdade (regerar presets, bancada) não corre num subagente: ponha-o na fila da onda —
  bash scripts/fila-pesada.sh pôr '<o comando>'
— e diga-o no retorno; o coordenador corre a fila 1× com a régua congelada (REGRAS_DE_CUSTO §METODOLOGIA 4)."
fi
pesado='(^|[;&|(`[:space:]])cargo[[:space:]]+(\+[^[:space:]]+[[:space:]]+)?(build|test|nextest|clippy|run|bench)([[:space:]]|$)|scripts/(ph2d-run|cargo-test-narrow|nextest-impacted|suite-completa|ship|integrar-rodada)\.sh'
printf '%s' "$cmd" | grep -qE "$pesado" || exit 0
if [ "$resta_s" -le 0 ]; then
  nega "⛔ Prazo de ${prazo_min} min deste agente esgotado (${gasto} min): comandos pesados recusados. Devolva AGORA «onde cheguei», os números que tem e o que falta — o coordenador decide."
fi
# O prazo que o comando levaria: o explícito, ou o default de cada porta (os números vivem nos scripts).
pedido="$(printf '%s' "$cmd" | grep -oE 'PH2D_PRAZO=[0-9]+' | head -1 | cut -d= -f2)"
if [ -z "$pedido" ]; then
  case "$cmd" in
    *cargo-test-narrow.sh*) pedido=900 ;;
    *ship.sh*) pedido=7200 ;;
    *nextest-impacted.sh* | *suite-completa.sh* | *integrar-rodada.sh*) pedido=5400 ;;
    *) pedido=1800 ;;
  esac
fi
if [ "$pedido" -gt "$resta_s" ]; then
  nega "⏱ Faltam ${resta_s} s do prazo deste agente (${prazo_min} min). Corra o comando com o corte no prazo — ponha à frente:
  PH2D_PRAZO=${resta_s} <o comando>
(o ph2d-run.sh mata-o aí). Se não cabe, não o corra: devolva «onde cheguei»."
fi
exit 0
