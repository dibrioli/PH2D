#!/usr/bin/env bash
# Hook PreToolUse(Bash): um `git commit` passa primeiro pelo `scripts/commit-barato.sh` (item F da comissão
# de velocidade, 29/10) — tecto de ficheiro/função/painel, `rustfmt --check`, HR-15 e filtros do nextest,
# só sobre os ficheiros do commit, sem compilar. Antes disto só a suíte do fecho os via, e o fecho da
# PainterWatercolor gastou ~10 commits a partir ficheiros > 700 linhas.
#
# ⚠️ FALHA ABERTO como o `tecto-de-recursos.sh`: jq ausente, JSON ilegível, script ausente (worktree
# antiga), prazo (9 s, o hook tem 10) ou erro do próprio espelho ⇒ o commit passa. Só um ✗ (exit 1) nega.
set -uo pipefail

entrada="$(cat 2>/dev/null)" || exit 0
command -v jq >/dev/null 2>&1 || exit 0
cmd="$(printf '%s' "$entrada" | jq -r '.tool_input.command // empty' 2>/dev/null)" || exit 0
[ -n "$cmd" ] || exit 0
printf '%s' "$cmd" | grep -qE '(^|[^[:alnum:]_-])git([[:space:]]+-[^[:space:]]+([[:space:]]+[^-[:space:]][^[:space:]]*)?)*[[:space:]]+commit([[:space:]]|$)' || exit 0
printf '%s' "$cmd" | grep -qE '(^|[[:space:]])PH2D_COMMIT_BARATO=0([[:space:]]|$)' && exit 0

dir="$(printf '%s' "$entrada" | jq -r '.cwd // empty' 2>/dev/null)"
[ -n "$dir" ] && [ -d "$dir" ] || dir="${CLAUDE_PROJECT_DIR:-$PWD}"
top="$(git -C "$dir" rev-parse --show-toplevel 2>/dev/null)" || exit 0
[ -f "$top/scripts/commit-barato.sh" ] || exit 0

saida="$(cd "$dir" && PH2D_COMMIT_BARATO_CWD="$dir" timeout 9 bash "$top/scripts/commit-barato.sh" --comando "$cmd" 2>&1)"
[ $? -eq 1 ] || exit 0
# Um ✗ do espelho é uma linha `✗ ficheiro:linha`; um traceback (o comando trazia `git … commit` num texto
# que o parser não entende — heredoc, `-C "$var"`) também sai 1 e é erro do espelho ⇒ passa.
printf '%s\n' "$saida" | grep -q '^Traceback' && exit 0
printf '%s\n' "$saida" | grep -qE '(^|[[:space:]])✗ ' || exit 0

jq -n --arg r "⛔ O commit leva o que a suíte do fecho vai reprovar (scripts/commit-barato.sh, sem compilar):

$saida

Cure e comite de novo. Se o commit SÓ move código entre ficheiros (o tecto volta no corte seguinte),
ou o espelho se enganou, ponha PH2D_COMMIT_BARATO=0 à frente do git commit — a lei continua a ser
o gate Rust na suíte." '{hookSpecificOutput:{hookEventName:"PreToolUse",
   permissionDecision:"deny", permissionDecisionReason:$r}}' 2>/dev/null
exit 0
