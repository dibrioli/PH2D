#!/bin/bash
# C31 — UMA sessão com o dono: o libinput (a mesa, crua) e o PH2D (o que o KDE entrega pelo tablet-v2) gravam a
# MESMA pincelada, e o c31_compara.py diz se o KDE aplica uma curva de pressão antes do tablet-v2.
# Uso (na worktree, com o binário do smoke já compilado): bash docs/Painter/ferramentas/caneta/c31_sessao.sh
# Pede a senha UMA vez (o libinput lê /dev/input). Pinte 6–10 traços com força variada no PH2D e feche a janela.
set -u
cd "$(git rev-parse --show-toplevel)"
D=target/c31; mkdir -p "$D"
EV=$(for e in /sys/class/input/event*; do grep -q 'Pen$' "$e/device/name" 2>/dev/null && basename "$e"; done | head -1)
[ -n "$EV" ] || { echo "a caneta não está ligada (nenhum /dev/input/event* chamado «… Pen»)"; exit 1; }
echo "caneta: /dev/input/$EV ($(cat /sys/class/input/$EV/device/name))"
sudo -v || exit 1
sudo timeout 900 libinput debug-events --device "/dev/input/$EV" > "$D/libinput.txt" 2>&1 &
LI=$!
sleep 1
PH2D_PEN_LOG="$PWD/$D/ph2d_pen.log" timeout 900 ./target/smoke/ph2d-host-desktop
# O $LI é o próprio `sudo`, que repassa o sinal ao libinput (nunca um `pkill`: apanharia outros).
sudo kill -TERM "$LI" 2>/dev/null
wait "$LI" 2>/dev/null
python3 -I docs/Painter/ferramentas/caneta/c31_compara.py "$D/libinput.txt" "$D/ph2d_pen.log" | tee "$D/veredicto.txt"
