#!/usr/bin/env bash
# Prova do guarda de recursos. Corra-a depois de lhe tocar:
#     bash .claude/hooks/tecto-de-recursos.prova.sh
#
# ⚠️ As linhas de «NÃO pode ser tocado» valem tanto como as de «recusado»: um
# falso positivo aqui pára as seis linhas, e o laço interno (`cargo check`) é o
# caso que NUNCA pode cair — encarecê-lo empurra os agentes de volta ao
# `cargo test`, que é o defeito 4,3:1 que o repo já mediu (CLAUDE.md §2).
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 1
falhas=0
t() {
  out=$(jq -n --arg c "$2" --argjson b "$3" \
        '{tool_name:"Bash",tool_input:{command:$c,run_in_background:$b}}' \
        | bash .claude/hooks/tecto-de-recursos.sh)
  if [ -z "$out" ]; then r="passa"; else r=$(printf '%s' "$out" | jq -r '.hookSpecificOutput.permissionDecision'); fi
  if [ "$r" = "$4" ]; then printf "  ✓ %-8s %s\n" "[$r]" "$1"
  else printf "  ✗ %-8s %s   (esperava %s)\n" "[$r]" "$1" "$4"; falhas=$((falhas+1)); fi
}
echo "— recusado —"
t "cargo test cru"        'cargo test -p ph2d-timeline'                     false deny
t "build --release"       'cargo build -p ph2d-host-desktop --release'      false deny
t "nextest workspace"     'cargo nextest run --workspace'                   false deny
t "smoke cru"             'cargo run -p ph2d-host-desktop --profile smoke'  false deny
t "gate de GPU cru"       'cargo test -p ph2d-field-gpu -- --ignored'       false deny
t "vigia sem prazo"       'until grep -q ok /tmp/x; do sleep 45; done'      true  deny
t "while true de fundo"   'while true; do sleep 30; done'                   true  deny
echo "— tem de passar —"
t "LAÇO INTERNO: check"   'cargo check -p ph2d-field-eval --all-targets'    false passa
t "cargo fmt"             'cargo fmt --all -- --check'                      false passa
t "cargo tree"            'cargo tree -p ph2d-field-eval'                   false passa
t "já pela porta"         'bash scripts/ph2d-run.sh cargo test -p ph2d-ecs' false passa
t "porta com a placa"     'PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-field-gpu' false passa
t "teste dirigido"        'bash scripts/cargo-test-narrow.sh ph2d-timeline' false passa
t "o portão de envio"     'bash scripts/ship.sh'                            false passa
t "git"                   'git status --short'                              false passa
t "vigia COM prazo"       "timeout 900 bash -c 'until test -f /tmp/x; do sleep 30; done'" true passa
t "laço em 1.º plano"     'until test -f /tmp/x; do sleep 5; done'          false passa
t "grep que MENCIONA"     'grep -rn "cargo test" docs/'                     false passa
echo "— falha ABERTO (um guarda que se engana a fechar pára seis linhas) —"
for e in 'lixo nao-json' '{}' '{"tool_input":{}}'; do
  printf '%s' "$e" | bash .claude/hooks/tecto-de-recursos.sh >/dev/null 2>&1
  [ $? = 0 ] && printf "  ✓ passa      entrada degenerada: %s\n" "$e" || { printf "  ✗ FECHOU     %s\n" "$e"; falhas=$((falhas+1)); }
done
echo; [ "$falhas" = 0 ] && echo "✓ $((0)) falhas — o guarda está honesto" || { echo "✗ $falhas falha(s)"; exit 1; }
