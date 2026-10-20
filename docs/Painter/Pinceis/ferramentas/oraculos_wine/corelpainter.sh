#!/bin/bash
# Corel Painter 2023 (teste de 15 dias) sob Wine, prefixo próprio, tela Xvfb :73 (só software).
export WINEDLLOVERRIDES="winemenubuilder.exe=d"; source "$HOME/Apps/pincel-oraculos/env.sh" corelpainter
cd "$WINEPREFIX/drive_c/Program Files/Corel/Painter 2023" || exit 1
exec timeout 14400 wine "Painter 2023 x64.exe" "$@"
