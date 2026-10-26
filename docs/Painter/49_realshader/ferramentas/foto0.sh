#!/bin/bash
# Fotografa SÓ a janela principal do Rebelle («Rebelle 8 Pro | …») no ecrã real (:0).
W=""; for w in $(xprop -display :0 -root _NET_CLIENT_LIST | grep -o '0x[0-9a-f]*'); do xprop -display :0 -id $w WM_NAME 2>/dev/null | grep -q '"Rebelle 8 Pro' && W=$w; done
[ -z "$W" ] && { echo "sem janela do Rebelle"; exit 1; }
import -display :0 -window "$W" "$1" && echo "$1 $W"
