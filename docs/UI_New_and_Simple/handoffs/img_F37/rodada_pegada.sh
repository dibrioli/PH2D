#!/bin/bash
# F37: pegada ANTES × DEPOIS, cenas 1 e 41, intercalada, numa só sessão kwin --virtual.
# uso: PH2D_GPU=1 bash scripts/ph2d-run.sh bash <este> <saida_dir>
set -euo pipefail
S="$(cd "$(dirname "$0")" && pwd)"
OUT="${1:?saida}"
RAIZ=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX
mkdir -p "$OUT/cfg" "$OUT/home"
CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/ph2d-fotografa"
mkdir -p "$CACHE"
cat > "$OUT/cfg/kwinrulesrc" <<'EOF'
[General]
count=1
rules=r1

[r1]
Description=maximiza tudo
wmclassmatch=0
types=1
maximizehoriz=true
maximizehorizrule=3
maximizevert=true
maximizevertrule=3
EOF
cat > "$OUT/sessao.sh" <<EOF
#!/bin/bash
if [ -z "\$DISPLAY" ] || [ "\$DISPLAY" = ":0" ] || [ "\$DISPLAY" = "${DISPLAY:-:0}" ]; then
  echo "RECUSA: nao e' a tela virtual" > "$OUT/log"; exit 3
fi
cd "$RAIZ"
janela() {
  for w in \$(xprop -display "\$DISPLAY" -root _NET_CLIENT_LIST | grep -o '0x[0-9a-f]*'); do
    xprop -display "\$DISPLAY" -id "\$w" _NET_WM_PID | grep -qx "_NET_WM_PID(CARDINAL) = \$1" && echo "\$w"
  done | tail -1
}
mede() {
  nvidia-smi --query-compute-apps=pid,used_memory --format=csv,noheader,nounits \
    | awk -F', *' -v p="\$1" '\$1 == p { print \$2 }'
}
for passo in antes:1:x depois:1:desempenho depois:1:uso antes:41:x depois:41:desempenho depois:41:uso depois:41:uso depois:41:desempenho antes:41:x depois:1:uso depois:1:desempenho antes:1:x; do
  v=\${passo%%:*}; r=\${passo#*:}; c=\${r%%:*}; b=\${r##*:}
  tag=\$v.\$b.c\$c.\$SECONDS
  env -u WAYLAND_DISPLAY HOME="$OUT/home" __GL_SHADER_DISK_CACHE_SIZE=17179869184 \
    __GL_SHADER_DISK_CACHE_PATH="$CACHE" MESA_SHADER_CACHE_DIR="$CACHE/mesa" \
    PH2D_OBJECT_MODE_SMOKE=\$c PH2D_GPU_BLOCOS=\$b PH2D_FLUID_PROFILE=1 PH2D_GPU_PEGADA=12,20 PH2D_EXIT_AFTER_FRAMES=100000 \
    "$S/\$v/ph2d-host-desktop" > "$OUT/app.\$tag.log" 2>&1 &
  APP=\$!
  sleep 12; m12=\$(mede \$APP)
  sleep 8; m20=\$(mede \$APP)
  W=\$(janela \$APP)
  [ -n "\$W" ] && import -display "\$DISPLAY" -window "\$W" "$OUT/foto.\$tag.png" 2>/dev/null
  echo "\$v \$b cena\$c nvidia12=\$m12 nvidia20=\$m20 (pid \$APP)" >> "$OUT/log"
  kill -9 \$APP 2>/dev/null; wait \$APP 2>/dev/null
  sleep 2
done
EOF
chmod +x "$OUT/sessao.sh"
bash "$RAIZ/scripts/vez-da-placa.sh" || true
XDG_CONFIG_HOME="$OUT/cfg" timeout 600 kwin_wayland --virtual --width 1930 --height 1040 \
  --xwayland --exit-with-session "$OUT/sessao.sh" > "$OUT/kwin.log" 2>&1 || true
cat "$OUT/log"
