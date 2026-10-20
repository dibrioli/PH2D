#!/bin/bash
# Fotografa a tela Xvfb inteira da app (nunca o ecrã do dono). Uso: bash foto.sh <app> <saida.png> [geometria WxH+X+Y]
source "$HOME/Apps/pincel-oraculos/env.sh" "$1"
if [ -n "$3" ]; then import -display "$DISPLAY" -window root -crop "$3" +repage "$2"; else import -display "$DISPLAY" -window root "$2"; fi && echo "$2"
