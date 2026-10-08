#!/bin/bash
# Fecha SÓ a tela virtual da bancada das gotas e o wineserver deste prefixo. ⚠️ Lê os PIDs pelo NOME
# exacto e confere a linha de comando — um `pgrep -f <padrão>` apanha o próprio shell que o corre.
for p in $(pgrep -x kwin_wayland); do
  tr '\0' ' ' < "/proc/$p/cmdline" | grep -q 'gotas_sessao[.]sh' && kill "$p" && echo "tela virtual $p fechada"
done
sleep 3
WINEPREFIX="$HOME/Apps/rebelle/wineprefix" wineserver -k 2>/dev/null
rm -f "$HOME/Apps/rebelle/oraculo/display.txt"
echo "wineserver restantes: $(pgrep -c -x wineserver)"
