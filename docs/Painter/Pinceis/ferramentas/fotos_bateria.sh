#!/bin/bash
# Fotografa a BATERIA do EXAME (bateria.json) no Rebelle 8, por preset e por cena — a generalização do
# `NewFeaturesWetPaint/ferramentas/fotos_lote.sh`. Por cena: eventos (bateria_rebelle.py) → parte A (traços +
# SIMULATION) → zoom 100 % → foto A (molhada) → parte B (DRY) → foto B (seca); recorte 1000×640 da tela.
# Cada parte acaba num BOOKMARK e o envio espera a resposta dele (ws_bateria.mjs), não um prazo fixo.
# RETOMÁVEL: salta a cena que já tem as duas fotos. Pára de começar presets novos depois de `ATE` (epoch).
#
# lista: `nome|TOOL|Grupo/Nome[|base TOOL:Grupo/Nome[|cenas separadas por espaço]]`
# PRIMEIRA=1 fica com a 1.ª cena de cada preset (a 1.ª passagem: uma foto de CADA preset antes das repetições);
# SO_CENA=<cena> restringe a passagem a uma cena;
# `<saída>/salta.txt` (linhas `TOOL|nome|cena`) lista as cenas que derrubaram o Rebelle duas vezes;
# `<saída>/.em_curso` diz a cena que estava a meio quando o Rebelle deixou de responder.
# Precisa da tela virtual com a porta ligada (gotas_inicia.sh + rebelle_pronto.sh).
# uso: fotos_bateria.sh <lista> <pasta de saída> [log.csv]   (saída: <pasta>/<TOOL>/<nome>_<cena>_<A|B>.png)
# saída: 0 = lista acabou · 10 = ATE passou · 11 = o Rebelle não responde (reinicie e retome)
set -u
O="$HOME/Apps/rebelle/oraculo"; F="$(cd "$(dirname "$0")" && pwd)"; L="$1"; S="$2"
LOG="${3:-$S/log.csv}"; EV="${EV_DIR:-$O/bateria_ev}"; ATE="${ATE:-0}"
BAT="$F/bateria.json"; CENAS_OMISSAO="gestos tamanhos sobre"
mkdir -p "$S" "$EV"
[ -f "$LOG" ] || echo "quando,nome,TOOL,preset,cena,estado,motivo,segundos" > "$LOG"
D=$(cat "$O/display.txt"); [ "$D" = ":0" ] && { echo RECUSA; exit 3; }
janelas() { xprop -display "$D" -root _NET_CLIENT_LIST 2>/dev/null | grep -o '0x[0-9a-f]*'; }
principal() { for w in $(janelas); do xprop -display "$D" -id "$w" WM_NAME | grep -q 'Trial\|pixels\|RebelleArtwork' && { echo "$w"; return; }; done; }
dialogos() { # fecha toda janela que não é a principal (os erros de preset ficam EM FILA) e diz quantas eram
  local n=0 p; p=$(principal)
  for w in $(janelas); do [ "$w" = "$p" ] && continue
    n=$((n+1)); timeout 30 bash "$O/inj.sh" "key enter; wait 400" >/dev/null; done
  echo "$n"
}
regista() { echo "$(date +%FT%T),$1,$2,\"$3\",$4,$5,\"$6\",$7" >> "$LOG"; }
foto() { # foto <ficheiro final> — a janela principal, conferida (título 1000 × 640, tela alinhada), recortada
  local w t full="$O/_bat_full.png"; w=$(principal); [ -n "$w" ] || return 1
  t=$(xprop -display "$D" -id "$w" WM_NAME); [[ "$t" == *"1000 x 640 pixels"* ]] || return 2
  import -display "$D" -window "$w" "$full" || return 3
  python3 -I - "$full" <<'PY' || return 4
import sys
from PIL import Image
a = Image.open(sys.argv[1]).convert("RGB")
fora, dentro = a.getpixel((479, 219)), a.getpixel((480, 220))
# o fundo cinzento do Rebelle fora da tela, o papel branco dentro: a tela está a 100 % no sítio do recorte
assert max(fora) < 140 and min(dentro) > 200, f"recorte desalinhado: fora={fora} dentro={dentro}"
PY
  magick "$full" -crop 1000x640+480+220 +repage -strip -define png:compression-level=9 "$1" && rm -f "$full"
}
n=0
while IFS='|' read -r nm tool preset base cenas; do
  [ -z "$nm" ] || [ "${nm:0:1}" = "#" ] && continue
  if [ "$ATE" != 0 ] && [ "$(date +%s)" -gt "$ATE" ]; then echo "ATE passou"; exit 10; fi
  mkdir -p "$S/$tool"
  lc=(${cenas:-$CENAS_OMISSAO}); [ -n "${PRIMEIRA:-}" ] && lc=("${lc[0]}")
  for cena in "${lc[@]}"; do
    [ -n "${SO_CENA:-}" ] && [ "$cena" != "$SO_CENA" ] && continue
    grep -qx "${tool}|${nm}|${cena}" "$S/salta.txt" 2>/dev/null && continue
    a="$S/$tool/${nm}_${cena}_A.png"; b="$S/$tool/${nm}_${cena}_B.png"
    [ -f "$a" ] && [ -f "$b" ] && continue
    echo "${tool}|${nm}|${cena}" > "$S/.em_curso"
    t0=$(date +%s); motivo=""
    nd=$(dialogos); [ "$nd" != 0 ] && echo "  (fechei $nd diálogo(s) antes de $nm $cena)"
    ev="$EV/${nm}_${cena}"
    if ! python3 -I "$F/bateria_rebelle.py" "$BAT" "$cena" "$tool" "$preset" "$ev" ${base:+--base "$base"}; then
      regista "$nm" "$tool" "$preset" "$cena" falha "eventos" 0; echo "✗ $nm $cena eventos"; continue; fi
    timeout 200 node "$F/ws_bateria.mjs" "${ev}_A.json" 180000 2 > "$EV/_ws.txt"; rc=$?
    if [ $rc -ne 0 ]; then
      regista "$nm" "$tool" "$preset" "$cena" falha "ws A rc=$rc $(tail -1 "$EV/_ws.txt")" $(( $(date +%s) - t0 ))
      echo "✗ $nm $cena: o Rebelle não respondeu (rc=$rc)"; exit 11; fi
    nd=$(dialogos)
    if [ "$nd" != 0 ]; then
      regista "$nm" "$tool" "$preset" "$cena" falha "diálogo de erro ($nd) — preset inválido?" $(( $(date +%s) - t0 ))
      echo "✗ $nm $cena: diálogo de erro"; continue; fi
    timeout 30 bash "$O/inj.sh" "click 1853 273; wait 300; key ctrl+a; wait 200; type 100; wait 200; key enter; wait 1200" >/dev/null
    foto "$a" || motivo="foto A rc=$?"
    timeout 90 node "$F/ws_bateria.mjs" "${ev}_B.json" 60000 2 > "$EV/_ws.txt"; rc=$?
    if [ $rc -ne 0 ]; then rm -f "$a"
      regista "$nm" "$tool" "$preset" "$cena" falha "ws B rc=$rc" $(( $(date +%s) - t0 )); echo "✗ $nm $cena B"; exit 11; fi
    [ -z "$motivo" ] && { foto "$b" || motivo="foto B rc=$?"; }
    if [ -n "$motivo" ]; then rm -f "$a" "$b"
      regista "$nm" "$tool" "$preset" "$cena" falha "$motivo" $(( $(date +%s) - t0 )); echo "✗ $nm $cena $motivo"; continue; fi
    regista "$nm" "$tool" "$preset" "$cena" ok "" $(( $(date +%s) - t0 ))
    echo "✓ $nm $cena $(( $(date +%s) - t0 )) s"
  done
  n=$((n+1))
done < "$L"
echo "LOTE_FIM $n"
exit 0
