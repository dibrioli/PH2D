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
t "fonte da Unreal"       'cat /home/enio/Documentos/Projetos/UnrealEngine/Engine/Source/Runtime/X.cpp' false deny
t "shader da Unreal"      'grep -rn Burley /home/enio/Documentos/Projetos/UnrealEngine/Engine/Shaders/' false deny
t "python do Blender"     'sed -n 1,40p /usr/share/blender/5.2/scripts/startup/x.py' false deny
# ⭐ R4 — o comando EXACTO de 05/10 18:30:03, e as outras portas para o mesmo logout.
t "O INCIDENTE: kill do pai" "pp=\$(ps -o ppid= -p 4175880 | tr -d ' '); ps -o pid,cmd -p \$pp; kill \$pp 2>/dev/null" false deny
t "kill \$PPID"           'kill $PPID'                                      false deny
_sd=$(pgrep -u "$(id -u)" -x systemd | head -1)
[ -n "$_sd" ] && t "kill do systemd --user ($_sd)" "kill $_sd"             false deny
t "kill -9 do PID 1"      'kill -9 1'                                       false deny
# ⭐ R5 — a crate INTEIRA no meio do trabalho (dono 14/10).
t "crate inteira pela porta" 'bash scripts/ph2d-run.sh cargo test -p ph2d-ecs'   false deny
t "crate inteira --release"  'bash scripts/ph2d-run.sh cargo test -p ph2d-ecs --release 2>&1 | tail -20' false deny
t "só o binário it"       'bash scripts/ph2d-run.sh cargo test -p ph2d-ecs --test it -- --nocapture' false deny
t "narrow sem nome"       'bash scripts/cargo-test-narrow.sh ph2d-timeline'  false deny
t "narrow caminho absoluto" 'cd /x && bash /home/enio/Documentos/Projetos/PH2D/scripts/cargo-test-narrow.sh ph2d-ecs 2>/dev/null' false deny
t "nextest de uma crate"  'bash scripts/ph2d-run.sh cargo nextest run -p ph2d-ecs --cargo-profile ci-test' false deny
t "seguido de ;"          'bash scripts/ph2d-run.sh cargo test -p ph2d-ecs; echo fim' false deny
# ⭐ R7 — clippy no meio do trabalho; R6 — edição que não prova que casou (dono 14/10).
t "clippy de uma crate"   'bash scripts/ph2d-run.sh cargo clippy -p ph2d-ecs --all-targets' false deny
t "clippy do workspace"   'bash scripts/ph2d-run.sh cargo clippy --workspace --all-targets -- -D warnings' false deny
t "sed -i"                "sed -i 's/velho/novo/' crates/x/src/lib.rs"     false deny
t "sed -E -i.bak"         "cd crates && sed -E -i.bak 's/a/b/' x.rs"         false deny
t "perl -pi"              "perl -pi -e 's/a/b/' x.rs"                        false deny
t "python replace SEM assert" "python3 - <<'PY'
from pathlib import Path
p = Path('x.rs'); s = p.read_text()
p.write_text(s.replace('a', 'b'))
PY"                                                                           false deny
t "heredoc a BASH corre"  "bash <<'EOF'
cargo test -p ph2d-ecs
EOF"                                                                          false deny
t "kill -9 -1"            'kill -9 -1'                                      false deny
t "pkill -u"              'pkill -u enio'                                   false deny
t "pkill do kwin"         'pkill -f kwin_wayland'                           false deny
t "killall plasmashell"   'killall plasmashell'                             false deny
t "pkill do VS Code"      'sleep 1; pkill code'                             false deny
t "loginctl"              'loginctl terminate-user enio'                    false deny
t "systemctl --user exit" 'systemctl --user exit'                           false deny
t "logout por qdbus"      'qdbus org.kde.Shutdown /Shutdown logout'         false deny
echo "— tem de passar —"
t "LAÇO INTERNO: check"   'cargo check -p ph2d-field-eval --all-targets'    false passa
t "cargo fmt"             'cargo fmt --all -- --check'                      false passa
t "cargo tree"            'cargo tree -p ph2d-field-eval'                   false passa
t "já pela porta"         'bash scripts/ph2d-run.sh cargo test -p ph2d-ecs weapon' false passa
t "porta com a placa"     'PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-field-gpu -- --ignored lente' false passa
t "teste dirigido"        'bash scripts/cargo-test-narrow.sh ph2d-timeline nesting' false passa
t "nome depois do --"     'bash scripts/ph2d-run.sh cargo test -p ph2d-ecs --test it -- --exact it::weapon::g2' false passa
t "nextest com test()"    "bash scripts/ph2d-run.sh cargo nextest run -p ph2d-ecs -E 'test(weapon)'" false passa
t "crate inteira COM motivo" 'PH2D_SUITE="motivo: crate nova" bash scripts/cargo-test-narrow.sh ph2d-node-x' false passa
t "o fecho"               'PH2D_SUITE=fecho bash scripts/ph2d-run.sh cargo test -p ph2d-ecs' false passa
t "só compilar os testes" 'bash scripts/ph2d-run.sh cargo test -p ph2d-ecs --no-run' false passa
t "a suíte do diff"       'bash scripts/nextest-impacted.sh'                false passa
t "grep que MENCIONA narrow" 'grep -rn cargo-test-narrow.sh docs/'         false passa
t "clippy no FECHO"       'PH2D_SUITE=fecho bash scripts/ph2d-run.sh cargo clippy --workspace --all-targets -- -D warnings' false passa
t "sed -n (ler)"          'sed -n 1,40p crates/x/src/lib.rs'                false passa
t "sed para OUTRO ficheiro" "sed -e 's/a/b/' x.rs > y.rs"                   false passa
t "grep que MENCIONA sed -i" "grep -rn 'sed -i' docs/"                     false passa
t "sed -i CONFERIDO"      "sed -i 's/a/b/' x.rs && grep -c 'b' x.rs"         false passa
t "python com .index()"   "python3 - <<'PY'
p = 'x.rs'; s = open(p).read(); i = s.index('fn a')
open(p, 'w').write(s.replace('fn a', 'fn b'))
PY"                                                                           false passa
t "--test de um ficheiro" 'bash scripts/ph2d-run.sh cargo test -p ph2d-eval-motion --test anel_memoria -- --nocapture' false passa
t "python replace COM assert" "python3 - <<'PY'
from pathlib import Path
p = Path('x.rs'); s = p.read_text(); assert s.count('a') == 1
p.write_text(s.replace('a', 'b'))
PY"                                                                           false passa
t "python re.subn + raise" "python3 - <<'PY'
import re, pathlib
p = pathlib.Path('x'); s, n = re.subn('a', 'b', p.read_text())
if n != 2: raise SystemExit(n)
p.write_text(s)
PY"                                                                           false passa
t "heredoc a python MENCIONA cargo test" "python3 - <<'PY'
print('cargo test -p ph2d-ecs e cargo clippy -p x')
PY"                                                                           false passa
t "o portão de envio"     'bash scripts/ship.sh'                            false passa
t "git"                   'git status --short'                              false passa
t "vigia COM prazo"       "timeout 900 bash -c 'until test -f /tmp/x; do sleep 30; done'" true passa
t "laço em 1.º plano"     'until test -f /tmp/x; do sleep 5; done'          false passa
t "grep que MENCIONA"     'grep -rn "cargo test" docs/'                     false passa
# ⭐ A metade que torna a R3 honesta: CORRER o alvo é o método, e tem de passar.
t "CORRER a Unreal"       '/home/enio/Documentos/Projetos/UnrealEngine/Engine/Binaries/Linux/UnrealEditor-Cmd P.uproject -run=pythonscript' false passa
t "CORRER o Blender"      'blender -b --factory-startup -P /var/tmp/arnes/render_ref.py' false passa
t "config do Blender"     'cat /usr/share/blender/5.2/datafiles/colormanagement/config.ocio' false passa
t "o nosso oráculo"       'python3 docs/Painter/ferramentas/oraculo_de_cor.py' false passa
# R4 — parar o que é SEU continua a passar.
t "kill de um PID seu"    'kill 4175986'                                    false passa
t "kill do grupo"         'kill -- -4175986'                                false passa
t "kill de job"           'kill %1'                                         false passa
t "um 1 fora do kill"     'sleep 1; kill 4175986; head -1 x.log'            false passa
t "sinal 9 e um PID seu"  'kill -9 4175986'                                 false passa
t "pkill da sonda"        'pkill -f sonda_aa'                               false passa
t "parar o scope"         'systemctl --user stop ph2d-run-line_x.scope'     false passa
t "listar as fatias"      "systemctl --user list-units 'ph2d*'"             false passa
t "ppid sem kill"         'ps -o ppid= -p 4175880'                          false passa
t "grep que MENCIONA kill" 'grep -rn "pkill systemd" docs/'                 false passa
echo "— falha ABERTO (um guarda que se engana a fechar pára seis linhas) —"
for e in 'lixo nao-json' '{}' '{"tool_input":{}}'; do
  printf '%s' "$e" | bash .claude/hooks/tecto-de-recursos.sh >/dev/null 2>&1
  [ $? = 0 ] && printf "  ✓ passa      entrada degenerada: %s\n" "$e" || { printf "  ✗ FECHOU     %s\n" "$e"; falhas=$((falhas+1)); }
done
echo; [ "$falhas" = 0 ] && echo "✓ $((0)) falhas — o guarda está honesto" || { echo "✗ $falhas falha(s)"; exit 1; }
