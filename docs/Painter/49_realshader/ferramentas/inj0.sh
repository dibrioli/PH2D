#!/bin/bash
# Injector no ecrã REAL do dono (:0) — autorizado pelo dono em 26/10 para o estudo do RealShader.
export WINEPREFIX="$HOME/Apps/rebelle/wineprefix" WINEDEBUG=-all
cd "$WINEPREFIX/drive_c" && DISPLAY=:0 timeout 120 wine "$HOME/Apps/rebelle/py/python.exe" 'Z:\tmp\claude-1000\-home-enio-Documentos-Projetos-PH2D\75e97cd2-c9ef-40e0-9f8c-03f299803bbd\scratchpad\real\inj_real.py' "$*" 2>&1 | tail -5
