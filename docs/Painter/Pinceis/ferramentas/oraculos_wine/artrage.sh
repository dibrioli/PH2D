#!/bin/bash
# ArtRage 6 Demo (Windows, sem prazo, não grava/exporta/grava scripts) sob Wine, prefixo próprio, tela Xvfb :71.
source "$HOME/Apps/pincel-oraculos/env.sh" artrage
cd "$WINEPREFIX/drive_c/Program Files/Ambient Design/ArtRage 6 Demo" || exit 1
exec timeout 14400 wine "ArtRage Demo.exe" "$@"
