# shellcheck shell=bash
# Item I (comissão 29/10): o prazo de 45 min do subagente é IMPOSTO (`.claude/hooks/prazo-do-agente.sh`) e o
# pesado de verdade vai à fila da onda com a régua congelada (`scripts/fila-pesada.sh`).
echo "— I: prazo do agente e fila do pesado —"
_i() { # _i <evento> <agente> <agora> [comando] → a decisão (ou o contexto) que o hook devolve
  jq -n --arg e "$1" --arg a "$2" --arg c "${4:-}" \
    '{hook_event_name:$e, tool_name:"Bash", tool_input:{command:$c}} + (if $a == "" then {} else {agent_id:$a} end)' \
    | PH2D_AGENTES_DIR="$tmp/agentes" PH2D_AGORA="$3" bash .claude/hooks/prazo-do-agente.sh \
    | jq -r '.hookSpecificOutput | (.permissionDecision // .additionalContext // "") | .[0:40]' 2>/dev/null
}
_i SubagentStart ag1 1000 >/dev/null
igual "o início fica gravado" "$(cat "$tmp/agentes/ag1" 2>/dev/null)" "1000"
igual "aos 10 min um teste por nome passa" "$(_i PreToolUse ag1 1600 'bash scripts/cargo-test-narrow.sh x y')" ""
igual "aos 10 min um comando leve passa" "$(_i PreToolUse ag1 1600 'git status')" ""
igual "aos 39 min não há aviso" "$(_i PostToolUse ag1 3340)" ""
case "$(_i PostToolUse ag1 3460)" in "⏱ 41 min"*) ok "aos 41 min o agente é avisado" ;; *) ko "aos 41 min NÃO houve aviso" ;; esac
igual "aos 40 min um narrow (900 s) não cabe" "$(_i PreToolUse ag1 3400 'bash scripts/cargo-test-narrow.sh x y')" "deny"
igual "…com PH2D_PRAZO dentro do que falta, passa" "$(_i PreToolUse ag1 3400 'PH2D_PRAZO=200 bash scripts/cargo-test-narrow.sh x y')" ""
igual "aos 46 min o pesado é recusado" "$(_i PreToolUse ag1 3760 'PH2D_PRAZO=10 bash scripts/ph2d-run.sh cargo check')" "deny"
igual "aos 46 min o leve passa (para escrever o retorno)" "$(_i PreToolUse ag1 3760 'cat x')" ""
igual "regerar presets num subagente vai à fila" "$(_i PreToolUse ag1 1600 'PH2D_REGERA_PRESETS=1 bash scripts/cargo-test-narrow.sh p q')" "deny"
igual "a janela principal não é tocada" "$(_i PreToolUse "" 9999999 'bash scripts/ship.sh')" ""
# a fila
_f() { PH2D_FILA_DIR="$tmp/fila" bash scripts/fila-pesada.sh "$@"; }
echo r1 >"$tmp/regua.txt"
_f pôr "echo corri >>$tmp/corridas" >/dev/null; _f pôr "echo corri >>$tmp/corridas" >/dev/null
igual "o mesmo comando entra 1× na fila" "$(wc -l <"$tmp/fila/fila")" "1"
_f correr >/dev/null 2>&1; igual "correr sem régua congelada recusa" "$?" "1"
_f congelar "$tmp/regua.txt" >/dev/null; echo r2 >"$tmp/regua.txt"
_f correr >/dev/null 2>&1; igual "régua mudada depois de congelar recusa" "$?" "1"
_f congelar "$tmp/regua.txt" >/dev/null; _f correr >/dev/null 2>&1
igual "com a régua congelada corre 1× e esvazia" "$(cat "$tmp/corridas" 2>/dev/null)·$(wc -l <"$tmp/fila/fila")" "corri·0"
