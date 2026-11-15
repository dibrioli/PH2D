#!/bin/bash
# corre DENTRO do kwin --virtual: X11 da Xwayland desta sessão, nunca o ecrã do dono
unset WAYLAND_DISPLAY
case "$DISPLAY" in :0|"") echo "RECUSA DISPLAY=$DISPLAY"; exit 9;; esac
BLENDER_USER_CONFIG=/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/f9e2f2a8-26d5-4ade-973a-9c6928b75506/scratchpad/probe/bcfg blender "$3" --python "$1" > "$2" 2>&1
