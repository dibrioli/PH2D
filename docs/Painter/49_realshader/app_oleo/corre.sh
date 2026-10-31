#!/bin/bash
# uso: corre.sh <saida_dir> <prazo_s> [VAR=val ...]  — corre o app na Radeon numa kwin --virtual, espera FIM,
# fotografa a janela e mata o app. Os PNG do documento são gravados pelo driver em <saida_dir>.
set -u
OUT="$1"; PRAZO="$2"; shift 2
RAIZ=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
BIN="$RAIZ/target/smoke/ph2d-host-desktop"
ICD=/usr/share/vulkan/icd.d/radeon_icd.x86_64.json
[ -f "$ICD" ] || ICD=/usr/share/vulkan/icd.d/radeon_icd.json
[ -f "$ICD" ] || { echo "sem ICD radeon"; exit 9; }
mkdir -p "$OUT/home" "$OUT/cfg"
CACHE="$HOME/.cache/ph2d-fotografa"; mkdir -p "$CACHE"
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
ENVS="$*"
cat > "$OUT/sessao.sh" <<EOF
#!/bin/bash
if [ -z "\$DISPLAY" ] || [ "\$DISPLAY" = ":0" ] || [ "\$DISPLAY" = "${DISPLAY:-:0}" ]; then
  echo "RECUSA display" > "$OUT/log"; exit 3
fi
cd "$RAIZ"
env -u WAYLAND_DISPLAY HOME="$OUT/home" VK_ICD_FILENAMES="$ICD" WGPU_BACKEND=vulkan \
  MESA_SHADER_CACHE_DIR="$CACHE/mesa" PH2D_OLEO_APP=1 PH2D_OLEO_APP_OUT="$OUT" $ENVS \
  PH2D_EXIT_AFTER_FRAMES=1000000 "$BIN" > "$OUT/app.log" 2>&1 &
APP=\$!
echo \$APP > "$OUT/app.pid"
sleep 4
grep -E "vulkan|nvidia|radeon|amdgpu" /proc/\$APP/maps | awk '{print \$6}' | sort -u > "$OUT/libs.txt"
T=0
while kill -0 \$APP 2>/dev/null && ! grep -q "oleo_app\] FIM" "$OUT/app.log"; do
  sleep 1; T=\$((T+1)); [ \$T -ge $PRAZO ] && { echo "PRAZO" >> "$OUT/log"; break; }
done
for w in \$(xprop -display "\$DISPLAY" -root _NET_CLIENT_LIST | grep -o '0x[0-9a-f]*'); do
  xprop -display "\$DISPLAY" -id "\$w" _NET_WM_PID | grep -qx "_NET_WM_PID(CARDINAL) = \$APP" && WIN=\$w
done
sleep 2
[ -n "\${WIN:-}" ] && import -display "\$DISPLAY" -window "\$WIN" "$OUT/janela.png" >> "$OUT/log" 2>&1
kill -9 \$APP 2>/dev/null
echo "fim T=\$T" >> "$OUT/log"
EOF
chmod +x "$OUT/sessao.sh"
KWIN_DRM_DEVICES=${KDEV:-/dev/dri/card0} XDG_CONFIG_HOME="$OUT/cfg" timeout $((PRAZO + 60)) kwin_wayland --virtual --width 1600 --height 1000 \
  --xwayland --exit-with-session "$OUT/sessao.sh" > "$OUT/kwin.log" 2>&1
cat "$OUT/log" 2>/dev/null
grep "oleo_app" "$OUT/app.log"
cat "$OUT/libs.txt"
