#!/bin/bash
# F38: placa cheia a ~400 MiB livres na NVIDIA, cena 41, F37 × F38; liberta a placa aos 15 s.
set -uo pipefail
BIN="$1"; OUT="$2"; RAIZ=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX
mkdir -p "$OUT/cfg" "$OUT/home"
cat > "$OUT/sessao.sh" <<EOS
#!/bin/bash
[ "\$DISPLAY" = "${DISPLAY:-:0}" ] && exit 3
cd "$RAIZ"
janela() { for w in \$(xprop -display "\$DISPLAY" -root _NET_CLIENT_LIST | grep -o '0x[0-9a-f]*'); do xprop -display "\$DISPLAY" -id "\$w" _NET_WM_PID | grep -qx "_NET_WM_PID(CARDINAL) = \$1" && echo "\$w"; done | tail -1; }
for v in f37 f38b f37 f38b; do
  "$RAIZ/target/smoke/examples/ocupa_a_placa" 400 60 > "$OUT/ocupa.log" 2>&1 & OC=\$!
  until grep -q seguro "$OUT/ocupa.log" || ! kill -0 \$OC 2>/dev/null; do sleep 1; done
  t=\$v.\$SECONDS
  env -u WAYLAND_DISPLAY HOME="$OUT/home" PH2D_OBJECT_MODE_SMOKE=41 PH2D_GPU_ERROS_TODOS=1 "$BIN/\$v/ph2d-host-desktop" > "$OUT/app.\$t.log" 2>&1 & APP=\$!
  sleep 14; W=\$(janela \$APP); [ -n "\$W" ] && import -display "\$DISPLAY" -window "\$W" "$OUT/cheia.\$t.png"
  e1=\$(grep -c '^\[gpu-erro\] #' "$OUT/app.\$t.log"); r1=\$(grep -ci 'outofmemory' "$OUT/app.\$t.log")
  kill \$OC; wait \$OC 2>/dev/null
  sleep 14; W=\$(janela \$APP); [ -n "\$W" ] && import -display "\$DISPLAY" -window "\$W" "$OUT/livre.\$t.png"
  e2=\$(grep -c '^\[gpu-erro\] #' "$OUT/app.\$t.log")
  echo "\$t | \$(head -c 200 "$OUT/ocupa.log" | tail -1) | cheia: gpu-erro=\$e1 oom=\$r1 | depois de libertar: gpu-erro=\$e2 | aviso-dono=\$(grep -c 'sem memoria\|sem memória\|ran out of memory' "$OUT/app.\$t.log")" >> "$OUT/log"
  kill -9 \$APP; wait \$APP 2>/dev/null; sleep 2
done
EOS
chmod +x "$OUT/sessao.sh"
XDG_CONFIG_HOME="$OUT/cfg" timeout 400 kwin_wayland --virtual --width 1930 --height 1040 --xwayland --exit-with-session "$OUT/sessao.sh" > "$OUT/kwin.log" 2>&1
cat "$OUT/log"
