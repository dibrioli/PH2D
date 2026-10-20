#!/bin/bash
# Ambiente comum dos oráculos de pincel sob Wine (Missão Pincéis J1, entrega C). Uso: source env.sh <app>
# A placa NÃO é tocada: Xvfb (X puro em software) + Mesa llvmpipe forçado + nenhum ICD Vulkan. Por isso estes
# oráculos não pedem a vez da placa (A_VEZ_DA_PLACA.md) e não disputam a tela do Rebelle (kwin --virtual).
APP="$1"
case "$APP" in artrage) N=71;; clipstudio) N=72;; corelpainter) N=73;; photoshop) N=74;; clipstudio1) N=75;; *) N=79;; esac
export PO="$HOME/Apps/pincel-oraculos" A="$HOME/Apps/$APP"
export DISPLAY=":$N" WAYLAND_DISPLAY= WINEPREFIX="$HOME/Apps/$APP/wineprefix" WINEDEBUG="${PO_WINEDEBUG:--all}"
export LIBGL_ALWAYS_SOFTWARE=1 __GLX_VENDOR_LIBRARY_NAME=mesa
export __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json VK_ICD_FILENAMES=/dev/null
export WINEDLLOVERRIDES="${WINEDLLOVERRIDES:-mscoree,mshtml=}"
[ "$DISPLAY" = ":0" ] && { echo RECUSA; exit 3; }
