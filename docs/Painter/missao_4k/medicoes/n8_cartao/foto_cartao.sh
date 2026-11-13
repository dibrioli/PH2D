#!/bin/bash
# Foto do cartão Performance: Xvfb 1600x1000, Wet de fábrica, aba Brush, rola a lista até o fim, clica a caixa.
set -u
R=/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor; cd $R
D=:95; O=$R/target/n8/foto; [ -z "${KEEP:-}" ] && rm -rf $O; mkdir -p $O/home
PLACA_ENV=(VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json
  MESA_VK_WSI_DEBUG=sw __GLX_VENDOR_LIBRARY_NAME=mesa __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json)
bash scripts/vez-da-placa.sh || exit 75
Xvfb $D -screen 0 1600x1000x24 >/dev/null 2>&1 & XV=$!; sleep 1
env -u WAYLAND_DISPLAY DISPLAY=$D HOME=$O/home PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PAINEL=wet PH2D_COMPOSITE_TELA=1600x1000 \
  PH2D_EXIT_AFTER_FRAMES=1000000 "${PLACA_ENV[@]}" target/smoke/ph2d-host-desktop > $O/app.log 2>&1 & APP=$!
WIN=; for _ in $(seq 1 120); do grep -q "pilha armada" $O/app.log && WIN=$(DISPLAY=$D xdotool search --pid $APP 2>/dev/null | tail -1); [ -n "$WIN" ] && break; sleep 0.5; done
echo "win ${WIN:-NENHUMA}"; sleep 3
DISPLAY=$D import -window root $O/0_inicio.png
DISPLAY=$D xdotool mousemove 905 500; for _ in $(seq 1 200); do DISPLAY=$D xdotool click 5; sleep 0.02; done; sleep 1
DISPLAY=$D import -window root $O/2_fim_da_lista.png
Y=$(python3 - <<PY2 || echo
PY2
)
DISPLAY=$D xdotool mousemove 827 706 click 1; sleep 1; DISPLAY=$D import -window root $O/3_marcada.png
DISPLAY=$D xdotool mousemove 600 400 sleep 0.3 mousemove 610 410 sleep 0.3 mousemove 905 500; sleep 1
kill $APP; sleep 1; kill -9 $APP 2>/dev/null; kill $XV
