#!/bin/bash
# ⭐ PROVA DA FAIXA ÚNICA NO KWIN (WAYLAND) — onda 4 da line/BarraDeMenus.
#
# Abre um KWin INVISÍVEL (`kwin_wayland --virtual`) com o seu próprio D-Bus (`dbus-run-session`: o
# KWin de dentro nunca vê a sessão do dono), corre o app como cliente WAYLAND nativo (sem Xwayland) e
# conduz um RATO FALSO (`rato_falso/`, protocolo org_kde_kwin_fake_input) que só existe nessa tela.
# A geometria da janela lê-se por um script do próprio KWin. Passos:
#   1 a janela abre sem a barra do sistema      5 duplo clique de novo restaura
#   2 arrastar pelo fundo da faixa move-a       6 o botão □ maximiza e restaura
#   3 o canto de baixo à direita redimensiona   7 o botão − minimiza
#   4 duplo clique no fundo maximiza            8 o botão × fecha o app (= o × do sistema)
# Fotos da tela virtual em docs/BarraDeMenus/fotos/onda4_*.png (foto_kwin.py, ScreenShot2 do D-Bus dela).
# uso: bash docs/BarraDeMenus/ferramentas/kwin_faixa_unica.sh   (saída: docs/BarraDeMenus/fotos/onda4_kwin.txt)
set -euo pipefail
RAIZ="$(cd "$(dirname "$0")/../../.." && pwd)"
if [ "${PH2D_NA_PORTA:-0}" != "1" ]; then
  exec bash "$RAIZ/scripts/ph2d-run.sh" bash "$0" "$@"
fi
( cd "$RAIZ" && cargo build -q -p ph2d-host-desktop --profile smoke )
BIN="$RAIZ/target/smoke/ph2d-host-desktop"
TMP="$(mktemp -d)"
export TMP
if [ "${GUARDA_TMP:-0}" = 1 ]; then echo "[faixa] sessão em $TMP" >&2; else trap 'rm -rf "$TMP"' EXIT; fi
SAIDA="$RAIZ/docs/BarraDeMenus/fotos/onda4_kwin.txt"
FONTE="$RAIZ/docs/BarraDeMenus/ferramentas/rato_falso"
wayland-scanner client-header "$FONTE/fake-input.xml" "$TMP/fake-input-client.h"
wayland-scanner private-code "$FONTE/fake-input.xml" "$TMP/fake-input-protocol.c"
gcc -O2 -I"$TMP" -o "$TMP/rato_falso" "$FONTE/rato_falso.c" "$TMP/fake-input-protocol.c" -lwayland-client -lm
CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/ph2d-fotografa"
mkdir -p "$TMP/cfg" "$TMP/home" "$CACHE"
SOCK="ph2d-faixa-$$"
DONO_WL="${WAYLAND_DISPLAY:-wayland-0}"
DONO_BUS="${DBUS_SESSION_BUS_ADDRESS:-}"

cat > "$TMP/geom.js" <<'JS'
for (const w of workspace.windowList()) {
  if (!w.normalWindow) continue;
  const g = w.frameGeometry;
  print("GEOM " + w.pid + " " + Math.round(g.x) + " " + Math.round(g.y) + " " + Math.round(g.width) + " " +
    Math.round(g.height) + " noBorder=" + w.noBorder + " minimized=" + w.minimized);
}
JS
cat > "$TMP/fecha.js" <<'JS'
for (const w of workspace.windowList()) { if (w.normalWindow) w.closeWindow(); }
JS
cat > "$TMP/ativa.js" <<'JS'
for (const w of workspace.windowList()) {
  if (w.normalWindow) { w.minimized = false; workspace.activeWindow = w; }
}
JS

cat > "$TMP/sessao.sh" <<'EOF'
#!/bin/bash
set -uo pipefail
source "$TMP/vars"
[ "$WAYLAND_DISPLAY" = "$DONO_WL" ] && { echo "RECUSA: socket do dono" >> "$TMP/log"; exit 3; }
[ -n "$DONO_BUS" ] && [ "$DBUS_SESSION_BUS_ADDRESS" = "$DONO_BUS" ] && { echo "RECUSA: D-Bus do dono" >> "$TMP/log"; exit 3; }
N=0
kwin_js() {  # corre um script do KWin e devolve as linhas GEOM novas
  N=$((N + 1))
  local id antes
  antes=$(grep -c 'GEOM' "$TMP/kwin.log" || true)
  id=$(qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.loadScript "$1" "p$N")
  qdbus6 org.kde.KWin "/Scripting/Script$id" org.kde.kwin.Script.run >/dev/null
  sleep 0.5
  qdbus6 org.kde.KWin "/Scripting/Script$id" org.kde.kwin.Script.stop >/dev/null 2>&1 || true
  grep 'GEOM' "$TMP/kwin.log" | tail -n +$((antes + 1)) | grep " $APP " | tail -1 | sed 's/.*GEOM //'
}
geom() { kwin_js "$TMP/geom.js"; }
# UM rato para a sessão inteira, alimentado por um fifo (ver rato_falso.c).
mkfifo "$TMP/rato.fifo"
"$TMP/rato_falso" < "$TMP/rato.fifo" 2>>"$TMP/log" &
exec 3> "$TMP/rato.fifo"
rato() {  # os passos e o tempo que levam (o rato corre ao lado)
  local p ms=0
  for p in "$@"; do echo "$p" >&3; case "$p" in "sleep "*) ms=$((ms + ${p#sleep })) ;; esac; done
  sleep "$(awk -v m=$ms 'BEGIN { print m / 1000 + 0.8 }')"
}
anota() { echo "$*" >> "$TMP/log"; }
# Onde a faixa tem fundo livre (o app di-lo com PH2D_FAIXA_LOG, varrido no alvo do último quadro).
FOTOS="$RAIZ_FOTOS"
foto() {  # a tela virtual inteira, pelo D-Bus DELA (foto_kwin.py recusa o do dono)
  DONO_BUS="$DONO_BUS" python3 -I "$FOTO_PY" "$FOTOS/onda4_$1.png" >> "$TMP/log" 2>&1
}
fundo() { grep -a 'fundo livre' "$TMP/app.log" | tail -1 | sed 's/.*x=\([-0-9]*\).*/\1/'; }
env -u DISPLAY HOME="$TMP/home" __GL_SHADER_DISK_CACHE_PATH="$CACHE" MESA_SHADER_CACHE_DIR="$CACHE/mesa" \
  __GL_SHADER_DISK_CACHE_SIZE=17179869184 PH2D_FAIXA_LOG=1 PH2D_GAME_SMOKE=2 \
  "$BIN" > "$TMP/app.log" 2>&1 &
APP=$!
for _ in $(seq 120); do
  sleep 0.5
  G=$(geom); [ -n "$G" ] && break
done
[ -n "${G:-}" ] || { anota "RECUSA: a janela do app nunca apareceu"; kill -9 $APP; exit 5; }
sleep 6
read -r P X Y W H B M <<< "$(geom)"
anota "1 abre:            $X $Y $W $H $B $M"
foto 1_abre_sem_a_barra_do_sistema
# O fundo da faixa: entre as abas e o campo (o campo centra-se e tem no máximo 400 px).
FX=$((X + $(fundo))); FY=$((Y + 14))
rato "move $FX $FY" "down" "sleep 400" "move $((FX + 60)) $((FY + 40))" "sleep 150" \
     "move $((FX + 150)) $((FY + 100))" "sleep 400" "up"
read -r P X2 Y2 W2 H2 B2 M2 <<< "$(geom)"
anota "2 arrasta +150+100: $X2 $Y2 $W2 $H2 (dx=$((X2 - X)) dy=$((Y2 - Y)))"
CX=$((X2 + W2 - 2)); CY=$((Y2 + H2 - 2))
rato "move $CX $CY" "down" "sleep 900" "move $((CX + 60)) $((CY + 40))" "sleep 150" \
     "move $((CX + 120)) $((CY + 80))" "sleep 400" "up"
read -r P X3 Y3 W3 H3 B3 M3 <<< "$(geom)"
anota "3 canto +120+80:    $X3 $Y3 $W3 $H3 (dw=$((W3 - W2)) dh=$((H3 - H2)))"
FX=$((X3 + $(fundo))); FY=$((Y3 + 14))
rato "move $FX $FY" "down" "sleep 60" "up" "sleep 120" "down" "sleep 60" "up"
read -r P X4 Y4 W4 H4 B4 M4 <<< "$(geom)"
anota "4 duplo clique:     $X4 $Y4 $W4 $H4 (tela $LARG x $ALT)"
foto 4_maximizada
FX=$((X4 + $(fundo))); FY=$((Y4 + 14))
rato "move $FX $FY" "down" "sleep 60" "up" "sleep 120" "down" "sleep 60" "up"
read -r P X5 Y5 W5 H5 B5 M5 <<< "$(geom)"
anota "5 duplo clique:     $X5 $Y5 $W5 $H5"
# Os botões: cada um tem a altura da faixa + 8 px de largura; × encostado à direita.
BW=$((FAIXA_H + 8)); BY=$((Y5 + FAIXA_H / 2))
rato "move $((X5 + W5 - BW / 2)) $BY" "sleep 600"
foto 5_rato_sobre_o_fechar
rato "move $((X5 + W5 - BW - BW / 2)) $BY" "sleep 300" "down" "sleep 60" "up"
read -r P X6 Y6 W6 H6 B6 M6 <<< "$(geom)"
anota "6 botão □:          $X6 $Y6 $W6 $H6"
rato "move $((X6 + W6 - BW - BW / 2)) $((Y6 + FAIXA_H / 2))" "sleep 300" "down" "sleep 60" "up"
read -r P X7 Y7 W7 H7 B7 M7 <<< "$(geom)"
anota "6 botão □ de novo:  $X7 $Y7 $W7 $H7"
rato "move $((X7 + W7 - 2 * BW - BW / 2)) $((Y7 + FAIXA_H / 2))" "sleep 300" "down" "sleep 60" "up"
read -r P X8 Y8 W8 H8 B8 M8 <<< "$(geom)"
anota "7 botão −:          $M8"
kwin_js "$TMP/ativa.js" >/dev/null; sleep 1
read -r P X9 Y9 W9 H9 B9 M9 <<< "$(geom)"
rato "move $((X9 + W9 - BW / 2)) $((Y9 + FAIXA_H / 2))" "sleep 300" "down" "sleep 60" "up"
sleep 2
foto 8_fechar_pergunta
if grep -aq 'pedido=Some(Close)' "$TMP/app.log"; then anota "8 botão ×:          pediu para fechar"; fi
if kill -0 $APP 2>/dev/null; then
  anota "8 botão ×:          o app CONTINUA aberto (a pergunta de gravar?)"
  # O × do SISTEMA, para comparar: o KWin fecha a janela como a barra dele faria.
  kwin_js "$TMP/fecha.js" >/dev/null; sleep 2
  if kill -0 $APP 2>/dev/null; then anota "8b × do sistema:     o app também continua aberto (a mesma pergunta)"
  else anota "8b × do sistema:     o app FECHOU (o nosso × é diferente!)"; fi
  kill -9 $APP 2>/dev/null
else anota "8 botão ×:          o app fechou"; fi
exec 3>&-
EOF
chmod +x "$TMP/sessao.sh"
LARG=1600; ALT=1000
FAIXA_H="${FAIXA_H:-22}"
RAIZ_FOTOS="$RAIZ/docs/BarraDeMenus/fotos"; FOTO_PY="$RAIZ/docs/BarraDeMenus/ferramentas/foto_kwin.py"
declare -p RAIZ_FOTOS FOTO_PY TMP BIN CACHE DONO_WL DONO_BUS LARG ALT FAIXA_H > "$TMP/vars"
bash "$RAIZ/scripts/vez-da-placa.sh" || exit $?
XDG_CONFIG_HOME="$TMP/cfg" QT_FORCE_STDERR_LOGGING=1 QT_LOGGING_RULES="${KWIN_LOG_RULES:-kwin_scripting.debug=true;js.debug=true}" \
  timeout 240 dbus-run-session -- kwin_wayland --virtual --socket "$SOCK" --width $LARG --height $ALT \
  --exit-with-session "$TMP/sessao.sh" > "$TMP/kwin.log" 2>&1 || true
mkdir -p "$(dirname "$SAIDA")"
{ echo "# kwin_faixa_unica.sh — $(date -Iseconds) — $(kwin_wayland --version 2>/dev/null)"; cat "$TMP/log" 2>/dev/null || true; } > "$SAIDA"
cat "$SAIDA"
grep -i -E 'panick|faixa única' "$TMP/app.log" | head -20 || true
[ -s "$TMP/log" ] || { echo "sem resultado — kwin.log:"; tail -30 "$TMP/kwin.log"; exit 1; }
