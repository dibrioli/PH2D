#!/bin/bash
# REPRODUZ O GESTO: abre a cena numa janela PEQUENA, fotografa, MAXIMIZA, fotografa outra vez.
#
# ⛔ O `fotografa_cena.sh` maximiza no MAPEAMENTO (regra do kwin) — isso e' um resize que ja'
# se mediu a funcionar. O report do dono e' sobre maximizar uma janela QUE JA' ESTA' A DESENHAR.
set -euo pipefail
# ⚠️ DERIVADA do sítio do roteiro, como a do irmão `fotografa_cena.sh` — um caminho absoluto
# aqui seria a worktree de quem o escreveu, e mediria a árvore errada em toda a outra.
RAIZ="$(cd "$(dirname "$0")/../../.." && pwd)"
AQUI="$(cd "$(dirname "$0")" && pwd)"
# ⭐ O cliente X vive ao lado deste roteiro e compila-se A PEDIDO (uma vez, ~0,2 s).
# ⚠️ Ele NAO fica no repo compilado: um binario versionado e' um binario que envelhece
# em silencio, e este e' 60 linhas de C sobre a libX11 que toda esta maquina ja' tem.
MAX="$AQUI/maximiza_janela"
if [ ! -x "$MAX" ] || [ "$AQUI/maximiza_janela.c" -nt "$MAX" ]; then
  gcc -O1 -o "$MAX" "$AQUI/maximiza_janela.c" -lX11 \
    || { echo "nao consegui compilar o cliente X (falta libX11?)" >&2; exit 2; }
fi
ENV_KV="${1:?VAR=valor}"
LARG="${2:?largura}"
ALT="${3:?altura}"
SAIDA="${4:?prefixo de saida}"
BIN="$RAIZ/target/release/ph2d-host-desktop"
[ -x "$BIN" ] || { echo "falta o binario release" >&2; exit 2; }
NOVO="$(find "$RAIZ/crates" "$RAIZ/shells" -name '*.rs' -newer "$BIN" -print -quit 2>/dev/null || true)"
if [ -n "$NOVO" ]; then
  echo "RECUSA: codigo mais novo que o binario (${NOVO#"$RAIZ/"})" >&2; exit 2
fi
mkdir -p "$(dirname "$SAIDA")"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/cfg" "$TMP/home"
# ⚠️ SEM a regra de maximizar: a janela tem de nascer pequena para o gesto existir.
printf '[General]\ncount=0\n' > "$TMP/cfg/kwinrulesrc"

cat > "$TMP/sessao.sh" <<EOF
#!/bin/bash
if [ -z "\$DISPLAY" ] || [ "\$DISPLAY" = ":0" ] || [ "\$WAYLAND_DISPLAY" = "wayland-0" ]; then
  echo "RECUSA: nao e' a tela virtual" > "$TMP/log"; exit 3
fi
cd "$RAIZ"
env -u WAYLAND_DISPLAY HOME="$TMP/home" $ENV_KV PH2D_EXIT_AFTER_FRAMES=100000 "$BIN" > "$TMP/app.log" 2>&1 &
APP=\$!
sleep 12
WIN=\$(xprop -display "\$DISPLAY" -root _NET_CLIENT_LIST | grep -o '0x[0-9a-f]*' | tail -1)
import -display "\$DISPLAY" -window "\$WIN" "${SAIDA}_antes.png" >> "$TMP/log" 2>&1
echo "--- ANTES do maximizar ---" >> "$TMP/log"
"$MAX" "\$DISPLAY" >> "$TMP/log" 2>&1
sleep 10
WIN2=\$(xprop -display "\$DISPLAY" -root _NET_CLIENT_LIST | grep -o '0x[0-9a-f]*' | tail -1)
import -display "\$DISPLAY" -window "\$WIN2" "${SAIDA}_depois.png" >> "$TMP/log" 2>&1
kill -9 \$APP 2>/dev/null
EOF
chmod +x "$TMP/sessao.sh"
if grep -q '`' "$TMP/sessao.sh"; then echo "RECUSA: crase no roteiro gerado" >&2; exit 4; fi

XDG_CONFIG_HOME="$TMP/cfg" timeout 180 kwin_wayland --virtual --width "$LARG" --height "$ALT" \
  --xwayland --exit-with-session "$TMP/sessao.sh" > "$TMP/kwin.log" 2>&1 || true
cat "$TMP/log" 2>/dev/null || true
cp "$TMP/app.log" "${SAIDA}.log" 2>/dev/null && echo "log: ${SAIDA}.log"
for f in "${SAIDA}_antes.png" "${SAIDA}_depois.png"; do
  [ -s "$f" ] && echo "foto: $f" || echo "SEM FOTO: $f"
done
