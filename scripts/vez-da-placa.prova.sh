#!/usr/bin/env bash
# Prova do porteiro da placa (scripts/ph2d-run.sh + scripts/vez-da-placa.sh). Corra-a depois de lhes tocar:
#     bash scripts/vez-da-placa.prova.sh
# Usa um cadeado PRÓPRIO (`PH2D_GPU_FECHADURA`): não toca na placa nem na vez das outras linhas.
# O cliente Rust tem os seus testes (`ph2d-gpu`, `a_vez_da_placa`); a prova com a placa real é a
# rodada da Luz 2D (docs/DevOps/A_VEZ_DA_PLACA.md §3).
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 1
T="$(mktemp -d)"
trap 'rm -rf "$T"' EXIT
export PH2D_GPU_FECHADURA="$T/placa.lock" PH2D_GPU_FECHADURA_TESTES="$T/testes.lock" PH2D_PRAZO=60
unset PH2D_GPU_VEZ PH2D_NA_PORTA PH2D_GPU PH2D_GPU_ESPERA PH2D_PLACA PH2D_VEZ_ARVORE
run() { bash scripts/ph2d-run.sh "$@"; }
falhas=0
ok() { if eval "$2"; then printf '  ✓ %s\n' "$1"; else printf '  ✗ %s   (%s)\n' "$1" "$2"; falhas=$((falhas + 1)); fi; }
agora() { date +%s.%N; }
dif() { awk -v a="$1" -v b="$2" 'BEGIN { printf "%.1f", b - a }'; }

echo "— fora da porta ninguém espera —"
t0=$(agora); bash scripts/vez-da-placa.sh; rc=$?
ok "o cliente sem porteiro sai já, com 0" "[ $rc = 0 ] && [ \$(dif $t0 \$(agora) | cut -d. -f1) -lt 1 ]"

echo "— a vez pedida a meio espera o detentor —"
PH2D_GPU=1 run bash -c "agora() { date +%s.%N; }; echo A \$(agora) > $T/a; sleep 4; echo A_FIM \$(agora) >> $T/a" 2>/dev/null &
sleep 1
run bash -c "bash scripts/vez-da-placa.sh 2>/dev/null; echo B \$(date +%s.%N) > $T/b" 2>"$T/b.err"
wait
a_fim=$(awk '/A_FIM/ {print $2}' "$T/a"); b=$(awk '{print $2}' "$T/b")
ok "B só entrou depois de A acabar" "awk -v a=$a_fim -v b=$b 'BEGIN { exit !(b >= a) }'"
ok "B ouviu quem segura (linha, comando)" "grep -q 'echo A' $T/b.err"

echo "— a vez larga-se quando o pedinte acaba, não no fim da árvore —"
run bash -c "bash -c 'bash scripts/vez-da-placa.sh; sleep 1'; sleep 4" 2>/dev/null &
sleep 0.5; t0=$(agora)
PH2D_GPU=1 run true 2>/dev/null
ok "a outra árvore entra quando o pedinte acaba (< 2,5 s), não no fim da árvore (~4,5 s)" \
  "awk -v d=\$(dif $t0 \$(agora)) 'BEGIN { exit !(d < 2.5) }'"
wait

echo "— quem volta a pedir espera a vez de novo —"
run bash -c "bash -c 'bash scripts/vez-da-placa.sh; sleep 0.5'; sleep 1.5; bash -c 'bash scripts/vez-da-placa.sh; echo A2 \$(date +%s.%N) > $T/a2'" 2>/dev/null &
sleep 1.2
PH2D_GPU=1 run bash -c "sleep 2; echo B_FIM \$(date +%s.%N) > $T/bfim" 2>/dev/null
wait
ok "o 2.º pedido da mesma árvore entrou depois de a outra acabar" \
  "awk -v a=\$(awk '{print \$2}' $T/a2) -v b=\$(awk '{print \$2}' $T/bfim) 'BEGIN { exit !(a >= b) }'"

echo "— dois pedintes da mesma árvore entram juntos —"
run bash -c "bash -c 'bash scripts/vez-da-placa.sh; sleep 2' & sleep 0.5; t=\$(date +%s.%N); bash -c 'bash scripts/vez-da-placa.sh'; echo \$t \$(date +%s.%N) > $T/dois; wait" 2>/dev/null
ok "o 2.º pedinte não espera o 1.º (< 1 s)" "awk '{ exit !(\$2 - \$1 < 1) }' $T/dois"

echo "— um pedido escrito aos pedaços não se perde —"
# O pid, uma pausa maior que o prazo do `read` do porteiro, e o resto da linha (o `writeln!` de um
# `File` escrevia assim; 23/10, 2 de 548 testes pendurados).
aos_pedacos='v=$PH2D_GPU_VEZ; exec 4<>"$v/porta"; printf %s $$ >&4; sleep 0.7; printf " pedacos\n" >&4
  for i in $(seq 1 30); do [ -e "$v/vez.$$" ] && exit 0; sleep 0.1; done; exit 1'
run bash -c "$aos_pedacos" 2>/dev/null; rc=$?
ok "com a placa livre (o prazo de 0,5 s)" "[ $rc = 0 ]"
run bash -c "bash -c 'bash scripts/vez-da-placa.sh; sleep 4' & sleep 0.5; bash -c '$aos_pedacos'; r=\$?; wait; exit \$r" 2>/dev/null; rc=$?
ok "com a árvore já na placa (o prazo de 0,2 s)" "[ $rc = 0 ]"

echo "— PH2D_VEZ_ARVORE=1 volta à lei antiga (a medição A × B) —"
PH2D_VEZ_ARVORE=1 run bash -c "bash -c 'bash scripts/vez-da-placa.sh; sleep 1'; sleep 3" 2>/dev/null &
sleep 0.5; t0=$(agora)
PH2D_GPU=1 run true 2>/dev/null
ok "a árvore segura até ao fim (> 2,5 s)" "awk -v d=\$(dif $t0 \$(agora)) 'BEGIN { exit !(d > 2.5) }'"
wait

if [ "$(compgen -G '/dev/dri/renderD*' | wc -l)" -ge 2 ]; then
  echo "— a fila de testes não espera a principal —"
  PH2D_GPU=1 run sleep 3 2>/dev/null &
  sleep 0.5; t0=$(agora)
  run bash -c 'bash scripts/vez-da-placa.sh testes' 2>/dev/null
  ok "a vez de testes entra já, com a principal ocupada (< 1,5 s)" "awk -v d=\$(dif $t0 \$(agora)) 'BEGIN { exit !(d < 1.5) }'"
  wait

  echo "— a fila de testes é exclusiva entre árvores —"
  run bash -c "bash -c 'bash scripts/vez-da-placa.sh testes; sleep 2'" 2>/dev/null &
  sleep 0.5; t0=$(agora)
  run bash -c 'bash scripts/vez-da-placa.sh testes' 2>/dev/null
  ok "a 2.ª árvore espera a 1.ª (> 1 s)" "awk -v d=\$(dif $t0 \$(agora)) 'BEGIN { exit !(d > 1) }'"
  wait

  echo "— PH2D_PLACA=principal fecha a fila de testes —"
  PH2D_PLACA=principal run bash -c 'test ! -e "$PH2D_GPU_VEZ/porta-testes"' 2>/dev/null; rc=$?
  ok "sem porta-testes na árvore" "[ $rc = 0 ]"
  run bash -c 'test -p "$PH2D_GPU_VEZ/porta-testes"' 2>/dev/null; rc=$?
  ok "e com ela, sem o interruptor" "[ $rc = 0 ]"
else
  echo "— (uma só placa: os casos da fila de testes não se aplicam) —"
fi

echo "— quem nunca pede não segura nada —"
run bash -c 'sleep 3' 2>/dev/null &
sleep 0.5; t0=$(agora)
PH2D_GPU=1 run true 2>/dev/null
ok "a vez à partida entra já, ao lado de uma árvore que não a pediu" "[ \$(dif $t0 \$(agora) | cut -d. -f1) -lt 2 ]"
wait

echo "— por ordem de chegada —"
PH2D_GPU=1 run sleep 2 2>/dev/null &
sleep 0.5
for i in 1 2 3; do PH2D_GPU=1 run bash -c "echo $i >> $T/ordem" 2>/dev/null & sleep 0.3; done
wait
ok "1, 2, 3 entram pela ordem em que chegaram" "[ \"\$(tr -d '\n' < $T/ordem)\" = 123 ]"

echo "— uma árvore dentro de outra não espera pelo pai —"
t0=$(agora)
PH2D_GPU=1 run bash -c 'PH2D_GPU=1 bash scripts/ph2d-run.sh bash scripts/vez-da-placa.sh' 2>/dev/null; rc=$?
ok "a vez aninhada sai já, sem se trancar" "[ $rc = 0 ] && [ \$(dif $t0 \$(agora) | cut -d. -f1) -lt 3 ]"

echo "— desistir diz quem segura —"
PH2D_GPU=1 run sleep 4 2>/dev/null &
sleep 1
PH2D_GPU=1 PH2D_GPU_ESPERA=1 run true 2>"$T/rec.err"; rc=$?
ok "PH2D_GPU_ESPERA esgotado sai com 75" "[ $rc = 75 ]"
ok "e a mensagem nomeia o detentor" "grep -q 'sleep 4' $T/rec.err"
wait

echo "— o prazo liberta a placa —"
PH2D_PRAZO=2 PH2D_GPU=1 run sleep 100 2>/dev/null &
sleep 0.5; t0=$(agora)
PH2D_GPU=1 run true 2>/dev/null
ok "o pendurado morre no prazo e o seguinte entra (< 6 s)" "[ \$(dif $t0 \$(agora) | cut -d. -f1) -lt 6 ]"
wait

echo "— um kill -9 da porta não deixa a placa presa —"
# Sem a função `run`: o `$!` de uma função em fundo é o subshell dela, não a porta.
PH2D_GPU=1 bash scripts/ph2d-run.sh sleep 30 2>/dev/null &
porta=$!
sleep 1; kill -9 "$porta"; t0=$(agora)
PH2D_GPU=1 run true 2>/dev/null
ok "a placa liberta-se em ≤ 7 s (o porteiro vigia o dono a cada 0,5 s)" "[ \$(dif $t0 \$(agora) | cut -d. -f1) -lt 7 ]"
wait 2>/dev/null

echo "— o comando não herda o cadeado —"
PH2D_GPU=1 run bash -c "ls -l /proc/\$\$/fd | grep -c placa.lock" > "$T/fd" 2>/dev/null
ok "nenhum descritor do cadeado no comando" "[ \"\$(cat $T/fd)\" = 0 ]"

echo "— limpo no fim —"
ok "sem .dono depois de tudo acabar" "[ ! -e $T/placa.lock.dono ] && [ ! -e $T/testes.lock.dono ]"
ok "sem pastas de porteiro destas corridas" "! grep -rqs placa.lock \"\${XDG_RUNTIME_DIR:-/tmp}\"/ph2d-vez.*/porteiro 2>/dev/null"

echo; [ "$falhas" = 0 ] && echo "✓ 0 falhas — o porteiro está honesto" || { echo "✗ $falhas falha(s)"; exit 1; }
