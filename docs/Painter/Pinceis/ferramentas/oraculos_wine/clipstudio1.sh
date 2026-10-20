#!/bin/bash
# Clip Studio Paint 1.13.2 (Windows, sem licença = modo de avaliação) sob Wine, prefixo próprio, tela Xvfb :75.
export WINEDLLOVERRIDES="mscoree="; source "$HOME/Apps/pincel-oraculos/env.sh" clipstudio1
cd "$WINEPREFIX/drive_c/Program Files/CELSYS/CLIP STUDIO 1.5/CLIP STUDIO PAINT" 2>/dev/null || cd "$WINEPREFIX/drive_c/Program Files (x86)/CELSYS/CLIP STUDIO 1.5/CLIP STUDIO PAINT" || exit 1
exec timeout 14400 wine CLIPStudioPaint.exe "$@"
