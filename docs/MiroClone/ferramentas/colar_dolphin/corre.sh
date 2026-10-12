#!/usr/bin/env bash
# ORÁCULO: o que o Dolphin põe na área de transferência quando o artista copia ficheiros (Ctrl+C),
# e o que o `arboard` 3 — a MESMA linha do shells/desktop/Cargo.toml — consegue ler disso.
#
# Tudo num `kwin_wayland --virtual` (com Xwayland: sem `wayland-data-control` o arboard só fala X11)
# num D-Bus PRÓPRIO, com XDG_RUNTIME_DIR/XDG_CONFIG_HOME/HOME temporários. Nunca toca a sessão do
# dono: o roteiro de dentro recusa-se se WAYLAND_DISPLAY for wayland-0 ou se DISPLAY for o de fora.
# Quem copia é o DOLPHIN, pela acção «edit_copy» que ele exporta no D-Bus (a mesma do Ctrl+C);
# a selecção é `dolphin --select` (1 ficheiro) e a acção «edit_select_all» (pasta com 2 ficheiros).
#
# Uso:   bash docs/MiroClone/ferramentas/colar_dolphin/corre.sh [saida.txt]
#        (sem argumento escreve no stdout)
#        SONDA_DIR=<pasta> muda onde a sonda do arboard é compilada (default: /tmp/ph2d-sonda-arboard-$USER)
# ⚠️ O controlo de TEXTO (wl-copy) vai no FIM: corrido ANTES dos casos (11/10), a cópia do Dolphin
#    não substituiu o texto (a sonda leu «olá texto» nos dois casos) — por apurar (foco?).
# Saída: os tipos MIME oferecidos (wl-paste --list-types), o hexdump de cada um, e a sonda do
#        arboard em dois caminhos: x11 (o do app) e wayland-data-control (só para comparar).
set -euo pipefail
AQUI="$(cd "$(dirname "$0")" && pwd)"
RAIZ="$(cd "$AQUI/../../../.." && pwd)"
SAIDA="${1:-/dev/stdout}"
cd "$RAIZ"

for p in dolphin kwin_wayland qdbus6 wl-paste magick dbus-run-session; do
  command -v "$p" >/dev/null || { echo "falta: $p" >&2; exit 2; }
done

# --- a sonda do arboard, fora da árvore do produto, com target próprio -------------------------
SONDA="${SONDA_DIR:-/tmp/ph2d-sonda-arboard-$USER}"
mkdir -p "$SONDA/src"
cat > "$SONDA/Cargo.toml" <<'EOF'
[package]
name = "sonda_arboard"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
# A MESMA linha do shells/desktop/Cargo.toml (default features: image-data; SEM wayland-data-control).
arboard = "=3.6.1"

[features]
# Só para COMPARAR: o caminho Wayland nativo do arboard (o app NÃO o liga).
wl = ["arboard/wayland-data-control"]
EOF
cat > "$SONDA/src/main.rs" <<'EOF'
//! Sonda: o que o arboard devolve depois de outro app (o Dolphin) copiar ficheiros.
fn main() {
    let via = if cfg!(feature = "wl") { "wayland-data-control" } else { "x11 (o do app)" };
    println!("arboard 3.6.1 · caminho = {via}");
    let mut cb = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(e) => {
            println!("Clipboard::new() = Err({e:?})");
            return;
        }
    };
    match cb.get_text() {
        Ok(t) => println!("get_text() = Ok({t:?}) [{} bytes]", t.len()),
        Err(e) => println!("get_text() = Err({e:?})"),
    }
    match cb.get_image() {
        Ok(i) => println!("get_image() = Ok({}x{}, {} bytes)", i.width, i.height, i.bytes.len()),
        Err(e) => println!("get_image() = Err({e:?})"),
    }
    match cb.get().file_list() {
        Ok(v) => println!("get().file_list() = Ok({v:?})"),
        Err(e) => println!("get().file_list() = Err({e:?})"),
    }
    match cb.get().html() {
        Ok(h) => println!("get().html() = Ok({h:?})"),
        Err(e) => println!("get().html() = Err({e:?})"),
    }
}
EOF
export CARGO_TARGET_DIR="$SONDA/target"
bash scripts/ph2d-run.sh cargo build -q --manifest-path "$SONDA/Cargo.toml" >&2
cp "$SONDA/target/debug/sonda_arboard" "$SONDA/sonda_x11"
bash scripts/ph2d-run.sh cargo build -q --manifest-path "$SONDA/Cargo.toml" --features wl >&2
cp "$SONDA/target/debug/sonda_arboard" "$SONDA/sonda_wl"
unset CARGO_TARGET_DIR

# --- a sessão virtual -------------------------------------------------------------------------
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/cfg" "$TMP/runtime" "$TMP/home" "$TMP/um" "$TMP/dois"
chmod 700 "$TMP/runtime"
magick -size 160x120 gradient:'#2a9d8f-#e9c46a' "$TMP/um/água teste.png"
cp "$TMP/um/água teste.png" "$TMP/dois/água teste.png"
magick -size 32x32 xc:'#e76f51' "$TMP/dois/b.gif"
DISPLAY_DE_FORA="${DISPLAY:-}"

cat > "$TMP/sessao.sh" <<EOF
#!/bin/bash
exec > "$TMP/log" 2>&1
if [ -z "\$WAYLAND_DISPLAY" ] || [ "\$WAYLAND_DISPLAY" = "wayland-0" ]; then
  echo "RECUSA: isto nao e' a sessao virtual (WAYLAND_DISPLAY=\$WAYLAND_DISPLAY)"; exit 3
fi
if [ -z "\$DISPLAY" ] || [ "\$DISPLAY" = "$DISPLAY_DE_FORA" ]; then
  echo "RECUSA: DISPLAY=\$DISPLAY nao e' o Xwayland da sessao virtual"; exit 3
fi
export HOME="$TMP/home" QT_QPA_PLATFORM=wayland
echo "sessao: WAYLAND_DISPLAY=\$WAYLAND_DISPLAY DISPLAY=\$DISPLAY"

servico() {
  for i in \$(seq 1 60); do
    S=\$(qdbus6 | grep -m1 'org.kde.dolphin' | tr -d ' ')
    [ -n "\$S" ] && { echo "\$S"; return 0; }
    sleep 0.25
  done
  return 1
}

acao() {
  # \$1 servico, \$2 nome da accao; procura o objecto da accao em qualquer janela do Dolphin
  OBJ=\$(qdbus6 "\$1" | grep -m1 "/actions/\$2\\\$")
  [ -n "\$OBJ" ] || { echo "accao \$2 NAO exportada no D-Bus"; return 1; }
  echo "trigger \$OBJ"
  qdbus6 "\$1" "\$OBJ" org.qtproject.Qt.QAction.trigger
}

despeja() {
  echo "--- wl-paste --list-types ---"
  TIPOS=\$(timeout 5 wl-paste --list-types 2>&1) || { echo "\$TIPOS (vazio)"; TIPOS=; }
  echo "\$TIPOS"
  echo "\$TIPOS" | while IFS= read -r T; do
    [ -n "\$T" ] || continue
    echo "--- tipo: \$T ---"
    timeout 5 wl-paste --no-newline --type "\$T" 2>&1 | xxd | head -20
  done
  echo "--- sonda arboard ---"
  timeout 10 "$SONDA/sonda_x11" 2>&1; echo "(exit=\$?)"
  timeout 10 "$SONDA/sonda_wl" 2>&1; echo "(exit=\$?)"
}

x11_ativa() {
  # O KWin só passa a selecção Wayland ao Xwayland quando uma janela X11 está ACTIVA: repete a
  # sonda x11 com uma janela xmessage à frente (a do app é Wayland nativa — winit 0.30 por omissão).
  echo "--- sonda x11 COM uma janela X11 activa (xmessage) ---"
  xmessage -geometry +10+10 "x11" > /dev/null 2>&1 &
  XM=\$!
  sleep 2
  echo "_NET_ACTIVE_WINDOW: \$(xprop -root _NET_ACTIVE_WINDOW 2>&1) · xmessage: \$(xdotool search --pid \$XM 2>/dev/null | tr '\\n' ' ')"
  timeout 10 "$SONDA/sonda_x11" 2>&1; echo "(exit=\$?)"
  kill \$XM 2>/dev/null; wait \$XM 2>/dev/null
  sleep 1
}

caso() {
  # \$1 titulo; \$2 pasta; \$3 ficheiro a seleccionar (vazio = edit_select_all)
  echo
  echo "================ \$1 ================"
  if [ -n "\$3" ]; then
    dolphin --new-window --select "\$2/\$3" > "$TMP/dolphin.log" 2>&1 &
  else
    dolphin --new-window "\$2" > "$TMP/dolphin.log" 2>&1 &
  fi
  PID=\$!
  SV=\$(servico) || { echo "o Dolphin nao apareceu no D-Bus"; kill \$PID; return 1; }
  echo "servico: \$SV"
  sleep 3
  echo "objectos D-Bus do Dolphin (sem as accoes):"
  qdbus6 "\$SV" | grep -v '/actions/' | sed 's/^/  /'
  echo "accoes exportadas: \$(qdbus6 "\$SV" | grep -c '/actions/')"
  [ -n "\$3" ] || acao "\$SV" edit_select_all
  sleep 0.5
  acao "\$SV" edit_copy
  sleep 1
  despeja
  x11_ativa
  kill \$PID 2>/dev/null; wait \$PID 2>/dev/null
  sleep 1
}

echo "================ CONTROLO: antes de qualquer copia ================"
despeja
caso "CASO 1: um ficheiro (agua teste.png com acento e espaco)" "$TMP/um" "água teste.png"
caso "CASO 2: dois ficheiros (agua teste.png + b.gif)" "$TMP/dois" ""
echo
echo "================ CONTROLO: TEXTO copiado por um cliente Wayland (wl-copy) ================"
printf 'olá texto' | wl-copy
sleep 1
despeja
x11_ativa
echo "--- dolphin.log (fim) ---"
tail -20 "$TMP/dolphin.log"
EOF
chmod +x "$TMP/sessao.sh"
if grep -q '`' "$TMP/sessao.sh"; then
  echo "RECUSA: o roteiro gerado tem uma CRASE" >&2
  exit 4
fi

XDG_CONFIG_HOME="$TMP/cfg" XDG_RUNTIME_DIR="$TMP/runtime" KWIN_WAYLAND_NO_PERMISSION_CHECKS=1 \
  timeout 150 dbus-run-session -- kwin_wayland --virtual --xwayland --no-lockscreen --no-global-shortcuts \
  --width 1280 --height 800 --socket ph2d-colar --exit-with-session "$TMP/sessao.sh" \
  > "$TMP/kwin.log" 2>&1 || true

{
  echo "# Oráculo: Ctrl+C de ficheiros no Dolphin → área de transferência → arboard"
  echo "# data: $(date -Iseconds)"
  echo "# $(dolphin --version 2>/dev/null) · $(kwin_wayland --version 2>/dev/null)"
  echo "# arboard (Cargo.lock da worktree): $(grep -A1 '^name = "arboard"$' Cargo.lock | sed -n 's/version = //p' | tr -d '"')"
  echo "# shells/desktop/Cargo.toml: $(grep -m1 '^arboard' shells/desktop/Cargo.toml)"
  echo "# $(pacman -Q qt6-base kio wl-clipboard xorg-xwayland 2>/dev/null | tr '\n' ' ')"
  echo "# roteiro: docs/MiroClone/ferramentas/colar_dolphin/corre.sh"
  if [ -f "$TMP/log" ]; then cat "$TMP/log"; else echo "a sessao nem correu; o KWin disse:"; tail -30 "$TMP/kwin.log"; fi
} > "$SAIDA"
