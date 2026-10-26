#!/bin/bash
# Injector no ecrã REAL do dono (:0) — autorizado pelo dono em 26/10 para o estudo do RealShader.
export WINEPREFIX="$HOME/Apps/rebelle/wineprefix" WINEDEBUG=-all
P="Z:$(cd "$(dirname "$0")" && pwd | tr / '\\')\\inj_real.py"
cd "$WINEPREFIX/drive_c" && DISPLAY=:0 timeout 120 wine "$HOME/Apps/rebelle/py/python.exe" "$P" "$*" 2>&1 | tail -5
