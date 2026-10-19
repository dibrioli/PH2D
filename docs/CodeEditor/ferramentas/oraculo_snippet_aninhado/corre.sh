#!/usr/bin/env bash
# Oráculo «snippet aninhado» (CodeEditor): VS Code 1.142.0 — escolher da lista de completação um item COM buracos
# estando o cursor DENTRO de um buraco de outro snippet vivo. Passo a passo: Tab, Shift+Tab, Esc, escrever fora.
# O produto é CORRIDO e FOTOGRAFADO, nunca lido (CLAUDE.md §0.9). VS Code = binário Microsoft, só o corremos.
#
# Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/oraculo_snippet_aninhado/corre.sh <saida>
#   Grava em <saida>: amostras.txt (uma linha por passo) e <cenário>_<passo>_c.png (linha 1 do editor, 1:1).
#   Fotos de ecrã inteiro em /tmp/oraculo_snip_raw.XXXX (dito no fim) — não entram no repo.
#
# A régua de cada passo (estado.py), sem pixels de texto:
#   - a SELECÇÃO: Ctrl+C com editor.emptySelectionClipboard=false (selecção vazia não copia) e a CLIPBOARD lida por
#     tkinter (não há xclip nesta máquina); a CLIPBOARD é esvaziada antes de cada Ctrl+C;
#   - o CURSOR: «Ln a, Col b (n selected)» da barra de estado, por OCR;
#   - as MARCAS dos buracos: a cor de fundo de cada coluna da linha 1 (moda da célula) + riscas verticais.
#   - a sessão VIVA: lê-se pelo passo seguinte (um Tab numa sessão morta INDENTA: o texto e a Col mudam).
#
# ⛔ Regras (copiadas do oraculo_acabamento/corre.sh): Xvfb PRIVADO :119 (recusa :0); foto = `import -window root`
#   (NUNCA `spectacle`); `code` com `env -i`, HOME / --user-data-dir / --extensions-dir próprios e XDG_RUNTIME_DIR
#   curto; sessão dbus PRIVADA sem pastas de serviços; cada processo sob `timeout`; pára-se pelo PID lançado aqui.
#
# Coordenadas: ecrã 1600x1000, janela em (0,0), pasta aberta (explorador à esquerda). Linha 1 em y = 97,
# coluna c em x = 423 + 8,4c (as do oraculo_acabamento, mesma disposição).
set -euo pipefail

D=":119"
[ "${SNIP_DISPLAY:-$D}" = ":0" ] && { echo "recuso: :0 é o ecrã do dono" >&2; exit 2; }
D="${SNIP_DISPLAY:-$D}"
AQUI=$(cd "$(dirname "$0")" && pwd)

if [ -z "${SNIP_DENTRO:-}" ]; then
  OUT="${1:-$(mktemp -d /tmp/oraculo_snip.XXXX)}"; mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
  RAW=$(mktemp -d /tmp/oraculo_snip_raw.XXXX)
  T=$(mktemp -d /tmp/snp.XXXX); chmod 700 "$T"
  cat >"$T/bus.conf" <<'BC'
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN" "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig><type>session</type><listen>unix:tmpdir=/tmp</listen><auth>EXTERNAL</auth>
<policy context="default"><allow send_destination="*" eavesdrop="true"/><allow eavesdrop="true"/><allow own="*"/></policy></busconfig>
BC
  rc=0
  SNIP_DENTRO=1 SNIP_T="$T" SNIP_RAW="$RAW" timeout 1500 \
    dbus-run-session --config-file="$T/bus.conf" -- bash "$0" "$OUT" || rc=$?
  rm -rf "$T"
  echo "gravado em $OUT ; fotos de ecrã inteiro em $RAW (rc=$rc)"
  exit "$rc"
fi

OUT="$1"; T="$SNIP_T"; RAW="$SNIP_RAW"
SO="${SNIP_SO:-A B B2 C C2 D D2 D3 V2 V3}"
quer() { [[ " $SO " == *" $1 "* ]]; }
export DISPLAY="$D" XDG_RUNTIME_DIR="$T"
unset WAYLAND_DISPLAY
A="$OUT/amostras.txt"
P="${SNIP_PAUSA:-1.2}"          # pausa depois de cada tecla (a máquina está carregada: load ~20+)

PIDS=()
limpa() { local i; for ((i=${#PIDS[@]}-1; i>=0; i--)); do kill "${PIDS[$i]}" 2>/dev/null || true; done; }
trap limpa EXIT
fato() { echo "$*" >>"$A"; }
shot() { import -window root "$RAW/$1.png"; }

{
  echo "# oráculo snippet aninhado — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg)"
  echo "# code --version: $(code --version | paste -sd ' ' -)"
  echo "# pacman: $(LC_ALL=C pacman -Qi visual-studio-code-bin 2>/dev/null | grep -E '^(Name|Version|Licenses) ' | sed 's/ *: /: /' | paste -sd ';' -)"
  echo "# licença: VS Code = binário Microsoft, só CORRIDO"
  echo "# display: Xvfb $D 1600x1000x24 ; dbus privado sem serviços; fotos de ecrã inteiro em $RAW"
  echo "# snippets de utilizador (ficheiro .code-snippets, sem scope): zfora = f(\${1:a}, \${2:b}, \${3:c})\$0 · zdentro = g(\${1:x}, \${2:y})\$0"
  echo "# régua: sel = CLIPBOARD depois de Ctrl+C (emptySelectionClipboard=false: <vazia> = selecção vazia) · barra = OCR"
  echo "#   · marcas = cor de fundo (moda) de cada coluna da linha 1, c0 = 1.º carácter · bordas = riscas verticais claras"
  echo "#   (cursor = risca CLARA de 2 px, cursorBlinking=solid; marca de um tabstop VAZIO = risca escura de 2 px × 19 px)"
  echo "#   (fundo do editor #121314 · realce da linha actual #242526 só com selecção vazia · par de parênteses #2b4957)"
} >"$A"

timeout 1400 Xvfb "$D" -screen 0 1600x1000x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
PIDS+=($!); sleep 1.5

V="$T/vs"; mkdir -p "$V/home" "$V/ud/User/snippets" "$V/ext" "$V/ws"
: >"$V/ws/s.txt"
printf '\n\nfunction f(a: number, b: number, c: number) { return a + b + c; }\nfunction gpar(x: number, y: number) { return x + y; }\n' >"$V/ws/b.ts"
cat >"$V/ud/User/snippets/oraculo.code-snippets" <<'EOF'
{
  "fora":   { "prefix": "zfora",   "body": "f(${1:a}, ${2:b}, ${3:c})$0", "description": "fora" },
  "dentro": { "prefix": "zdentro", "body": "g(${1:x}, ${2:y})$0",        "description": "dentro" }
}
EOF
cat >"$V/ud/User/settings.json" <<'EOF'
{ "workbench.startupEditor": "none", "telemetry.telemetryLevel": "off", "update.mode": "none",
  "extensions.autoCheckUpdates": false, "extensions.autoUpdate": false, "workbench.tips.enabled": false,
  "security.workspace.trust.enabled": false, "window.restoreWindows": "none", "files.hotExit": "off",
  "chat.disableAIFeatures": true, "workbench.enableExperiments": false,
  "editor.emptySelectionClipboard": false, "editor.cursorBlinking": "solid",
  "typescript.suggest.completeFunctionCalls": true }
EOF
env -i HOME="$V/home" PATH=/usr/bin:/bin USER="$USER" LANG=en_US.UTF-8 DISPLAY="$D" XDG_RUNTIME_DIR="$T" \
  DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" \
  timeout 1300 /usr/share/code/code --user-data-dir="$V/ud" --extensions-dir="$V/ext" --new-window \
  --disable-workspace-trust --disable-gpu --password-store=basic --ozone-platform=x11 \
  --skip-welcome --skip-release-notes "$V/ws" "$V/ws/b.ts" "$V/ws/s.txt" >"$T/code.log" 2>&1 &
CODE_PID=$!; PIDS+=("$CODE_PID")
W=""
for _ in $(seq 40); do W=$(xdotool search --name 'Visual Studio Code' 2>/dev/null | head -1 || true); [ -n "$W" ] && break; sleep 1; done
[ -n "$W" ] || { echo "VS Code não abriu" >&2; cat "$T/code.log" >&2; exit 1; }
xdotool windowmove "$W" 0 0 windowsize "$W" 1600 1000
sleep 10                                                     # o servidor TS sobe
xdotool mousemove 1200 820; sleep 1; shot inicio
magick "$RAW/inicio.png" -crop 1000x120+300+30 +repage "$OUT/inicio_c.png"

abre() { # ficheiro
  xdotool key ctrl+p; sleep 1; xdotool type --delay 40 "$1"; sleep 1.2; xdotool key Return; sleep 2
}
reset() { # limpa a linha 1 (o resto do ficheiro b.ts fica): Esc, Esc, ctrl+Home, shift+End, Delete
  xdotool key Escape; sleep 0.3; xdotool key Escape; sleep 0.3
  xdotool key ctrl+Home; xdotool key shift+End; xdotool key Delete; sleep 0.8
}
# um passo: a tecla, a foto, a régua
N=0
passo() { # cenário etiqueta descrição-da-tecla
  local tag="$1_$(printf %02d $N)_$2"; N=$((N + 1))
  sleep "$P"
  shot "$tag"
  python3 -I "$AQUI/estado.py" limpa
  xdotool key ctrl+c; sleep 0.6
  local r; r=$(python3 -I "$AQUI/estado.py" le "$RAW/$tag.png" 97 423 8.4 28 2>&1 || echo "régua falhou")
  magick "$RAW/$tag.png" -crop 420x24+415+85 +repage "$OUT/${tag}_c.png"
  fato "$1 | $3 | $r | foto ${tag}_c.png"
}
tecla() { xdotool key "$1"; }
# inserir pela lista: escrever o prefixo, pedir a lista (Ctrl+Espaço — com a lista já aberta só alterna o detalhe),
# esperar, fotografar a lista, Enter
pela_lista() { # cenário prefixo etiqueta
  xdotool type --delay 60 "$2"; sleep 1.5; xdotool key ctrl+space; sleep "${ESPERA_LISTA:-2.5}"
  shot "$1_lista_$3"   # recorte da lista no repo só para A, V2 e V3 (os outros cenários repetem o A; tecto 300 KB)
  case "$1" in A|V2|V3) magick "$RAW/$1_lista_$3.png" -crop 700x200+400+85 +repage "$OUT/$1_lista_$3_c.png" ;; esac
  xdotool key Return
}
fora_dentro() { # cenário — fora no início da linha 1, depois dentro no buraco 1 de fora
  pela_lista "$1" zfora fora;     passo "$1" fora   "zfora⏎ (lista)"
  pela_lista "$1" zdentro dentro; passo "$1" dentro "zdentro⏎ (lista) com «a» seleccionado"
}
fim_cena() { # cenário: o texto final inteiro da linha 1 (Home, shift+End, Ctrl+C) — mata a sessão, por isso no FIM
  xdotool key Escape; sleep 0.3; xdotool key Home; xdotool key shift+End
  python3 -I "$AQUI/estado.py" limpa; xdotool key ctrl+c; sleep 0.6
  fato "$1 | texto final da linha 1 | «$(python3 -I -c 'import tkinter as t;r=t.Tk();r.withdraw();print(r.clipboard_get())' 2>/dev/null || echo '<vazia>')»"
}

abre s.txt
if quer A; then  # 1. Tab, Tab, Tab…
  N=0; reset; fora_dentro A
  for i in 1 2 3 4 5 6 7; do tecla Tab; passo A "tab$i" "Tab #$i"; done
  fim_cena A
fi
if quer B; then  # 2. Shift+Tab a partir do 1.º buraco de dentro
  N=0; reset; fora_dentro B
  for i in 1 2 3; do tecla shift+Tab; passo B "stab$i" "Shift+Tab #$i"; done
  for i in 1 2 3 4; do tecla Tab; passo B "tab$i" "Tab #$i (depois dos Shift+Tab)"; done
  fim_cena B
fi
if quer B2; then  # 2b. ir até ao fim do de dentro e voltar com Shift+Tab
  N=0; reset; fora_dentro B2
  for i in 1 2 3; do tecla Tab; passo B2 "tab$i" "Tab #$i"; done
  for i in 1 2 3 4 5; do tecla shift+Tab; passo B2 "stab$i" "Shift+Tab #$i"; done
  fim_cena B2
fi
if quer C; then  # 3. Esc no 1.º buraco de dentro
  N=0; reset; fora_dentro C
  tecla Escape; passo C esc "Esc (no buraco x de dentro)"
  for i in 1 2; do tecla Tab; passo C "tab$i" "Tab #$i depois do Esc"; done
  fim_cena C
fi
if quer C2; then  # 3b. Esc no 2.º buraco de dentro
  N=0; reset; fora_dentro C2
  tecla Tab; passo C2 tab1 "Tab #1"
  tecla Escape; passo C2 esc "Esc (no buraco y de dentro)"
  tecla Tab; passo C2 tab2 "Tab depois do Esc"
  fim_cena C2
fi
if quer D; then  # 4. sair de todos os buracos (Home) e escrever
  N=0; reset; fora_dentro D
  tecla Home; passo D home "Home (antes de f, fora de tudo)"
  xdotool type Q; passo D escreve "escreve «Q»"
  for i in 1 2; do tecla Tab; passo D "tab$i" "Tab #$i"; done
  fim_cena D
fi
if quer D2; then  # 4b. End (depois do «)» final = onde está o $0 de fora) e escrever
  N=0; reset; fora_dentro D2
  tecla End; passo D2 end "End (depois do último «)»)"
  xdotool type Q; passo D2 escreve "escreve «Q»"
  for i in 1 2; do tecla Tab; passo D2 "tab$i" "Tab #$i"; done
  fim_cena D2
fi
if quer D3; then  # 4c. clique na col 10 (entre «, » do de fora, fora de qualquer buraco) e escrever
  N=0; reset; fora_dentro D3
  xdotool mousemove $((423 + 84)) 97 click 1; sleep 0.3; xdotool mousemove 1200 820
  passo D3 clique "clique na col 10 (f(g(x, y),|␣b…)"
  xdotool type Q; passo D3 escreve "escreve «Q»"
  for i in 1 2; do tecla Tab; passo D3 "tab$i" "Tab #$i"; done
  fim_cena D3
fi
if quer V2 || quer V3; then abre b.ts; sleep 6; fi
if quer V2; then  # o de dentro vem do TypeScript (completeFunctionCalls sobre gpar(x, y))
  N=0; reset
  pela_lista V2 zfora fora; passo V2 fora "zfora⏎ (snippet de utilizador, em b.ts)"
  ESPERA_LISTA=5 pela_lista V2 gpar dentro; passo V2 dentro "gpar⏎ (TS completeFunctionCalls) com «a» seleccionado"
  for i in 1 2 3 4 5 6; do tecla Tab; passo V2 "tab$i" "Tab #$i"; done
  fim_cena V2
fi
if quer V3; then  # os DOIS vêm do TypeScript: f(a, b, c) por completeFunctionCalls, gpar dentro
  N=0; reset
  ESPERA_LISTA=5 pela_lista V3 "f" fora; passo V3 fora "f⏎ (TS completeFunctionCalls)"
  ESPERA_LISTA=5 pela_lista V3 gpar dentro; passo V3 dentro "gpar⏎ (TS) no 1.º buraco"
  for i in 1 2 3 4 5 6; do tecla Tab; passo V3 "tab$i" "Tab #$i"; done
  fim_cena V3
fi

xdotool key ctrl+q; sleep 3; kill "$CODE_PID" 2>/dev/null || true; sleep 2
echo "fim — $(date -u +%H:%MZ) loadavg $(cat /proc/loadavg)" >>"$A"
