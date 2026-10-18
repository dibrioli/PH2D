#!/usr/bin/env bash
# Oráculo do I8b (CodeEditor, Onda 2): como o VS Code e o Godot mostram as CORRECÇÕES no balão de um
# problema, e como desenham a 1.ª linha pré-seleccionada de uma lista (Ctrl+. / autocompletar).
# O produto é CORRIDO e FOTOGRAFADO, nunca lido (CLAUDE.md §0.9). Medição: ../medicoes_onda2/I8b_botoes_no_balao.md
#
# Licenças: VS Code = binário Microsoft (núcleo Code-OSS MIT), só o corremos; Godot = MIT (pacman -Qi godot).
#
# Uso: bash corre.sh [pasta_de_saída]      (por omissão: uma pasta nova em /tmp)
#   Grava raw/*.png (ecrã inteiro), *_c.png (recortes) e amostras.txt (cor #RRGGBB de cada ponto medido).
#
# ⛔ Regras (o dono já foi prejudicado por cada uma):
#   - Tudo corre num Xvfb PRIVADO (:117 por omissão, I8B_DISPLAY muda); nunca no ecrã do dono, nunca `spectacle`.
#     Foto = `import -window root` do Xvfb; rato e teclas = `xdotool` com DISPLAY do Xvfb (o XTest funciona
#     no Xvfb; na Xwayland de um `kwin --virtual` NÃO funciona — ver docs/Components/ferramentas/fotografa_cena.sh).
#   - O VS Code do dono está aberto e as variáveis VSCODE_IPC_HOOK* da janela do agente apontam para ELE: o
#     `code` corre com `env -i`, HOME / --user-data-dir / --extensions-dir próprios, e XDG_RUNTIME_DIR curto
#     (o socket do VS Code recusa caminhos > 107 caracteres). Sem HOME isolado ele escreve no
#     ~/.vscode-shared/sharedStorage do dono (medido 18/10).
#   - Cada processo sob `timeout`; pára-se pelo PID que ESTE script lançou (Ctrl+Q no VS Code, TERM no Godot).
#
# As coordenadas são as do ecrã 1600x1000 com a janela em (0,0) e o leiaute de fábrica do VS Code 1.142.0
# e do Godot 4.7.2; outra versão pode mover o balão — confira as fotos raw/ antes de ler amostras.txt.
set -euo pipefail

D="${I8B_DISPLAY:-:117}"
[ "$D" = ":0" ] && { echo "recuso: :0 é o ecrã do dono" >&2; exit 2; }
OUT="${1:-$(mktemp -d /tmp/oraculo_i8b.XXXX)}"
mkdir -p "$OUT/raw"
T=$(mktemp -d /tmp/i8b.XXXX)          # curto de propósito (socket do VS Code)
chmod 700 "$T"
export DISPLAY="$D"
unset WAYLAND_DISPLAY

XVFB_PID=""
limpa() { [ -n "$XVFB_PID" ] && kill "$XVFB_PID" 2>/dev/null || true; rm -rf "$T"; }
trap limpa EXIT

shot() { import -window root "$OUT/raw/$1.png"; }
crop() { magick "$OUT/raw/$1.png" -crop "$2" +repage "$OUT/$1_c.png"; }
px()   { magick "$OUT/raw/$1.png" -format "%[hex:p{$2,$3}]" info:; }
amostra() { echo "$1 ($2,$3) #$(px "$1" "$2" "$3") — $4" >>"$OUT/amostras.txt"; }
hover() { xdotool mousemove 1200 800; sleep 0.8; xdotool mousemove $(($1 - 2)) "$2"; sleep 0.2; xdotool mousemove "$1" "$2"; sleep "${3:-2}"; }

{
  echo "# oráculo I8b — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg)"
  echo "# code --version: $(code --version | paste -sd ' ' -)"
  echo "# godot --version: $(godot --version)"
  echo "# pacman: $(LC_ALL=C pacman -Qi godot visual-studio-code-bin 2>/dev/null | grep -E '^(Name|Version|Licenses) ' | sed 's/ *: /: /' | paste -sd ';' -)"
  echo "# display: Xvfb $D 1600x1000x24"
} >"$OUT/amostras.txt"

timeout 900 Xvfb "$D" -screen 0 1600x1000x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
XVFB_PID=$!
sleep 1.5

# ───────────────────────── VS Code ─────────────────────────
V="$T/vs"; mkdir -p "$V/home" "$V/ud/User" "$V/ext" "$V/ws"
printf 'const amplitude = 1;\nconsole.log(amplitud);\n' >"$V/ws/typo.ts"
printf 'class Ponto { x = 0; }\nconst p = new Ponto();\np.y = 1;\n' >"$V/ws/varios.ts"
cat >"$V/ud/User/settings.json" <<'EOF'
{ "workbench.startupEditor": "none", "telemetry.telemetryLevel": "off", "update.mode": "none",
  "extensions.autoCheckUpdates": false, "extensions.autoUpdate": false, "workbench.tips.enabled": false,
  "security.workspace.trust.enabled": false, "window.restoreWindows": "none",
  "chat.disableAIFeatures": true, "workbench.enableExperiments": false }
EOF
env -i HOME="$V/home" PATH=/usr/bin:/bin USER="$USER" LANG=en_US.UTF-8 DISPLAY="$D" XDG_RUNTIME_DIR="$T" \
  timeout 600 /usr/share/code/code --user-data-dir="$V/ud" --extensions-dir="$V/ext" --new-window \
  --disable-workspace-trust --disable-gpu --password-store=basic --ozone-platform=x11 \
  --skip-welcome --skip-release-notes "$V/ws" "$V/ws/typo.ts" >"$T/code.log" 2>&1 &
CODE_TIMEOUT_PID=$!
for _ in $(seq 30); do W=$(xdotool search --name 'Visual Studio Code' 2>/dev/null | head -1 || true); [ -n "$W" ] && break; sleep 1; done
[ -n "$W" ] || { echo "VS Code não abriu" >&2; cat "$T/code.log" >&2; exit 1; }
xdotool windowmove "$W" 0 0 windowsize "$W" 1600 1000
sleep 8   # o servidor TS sobe e sublinha

# 1. balão sobre a palavra sublinhada
hover 557 117 2; shot v1_balao; crop v1_balao 800x300+350+0
amostra v1_balao 1000 140 "fundo do balão"; amostra v1_balao 1000 218 "faixa de baixo (View Problem / Quick Fix...)"
amostra v1_balao 1000 180 "separador"; amostra v1_balao 524 190 "borda"; amostra v1_balao 800 400 "fundo do editor"
# 1b. sarjeta (linha 2) e a lâmpada (com o cursor na linha do erro)
hover 362 116 2; shot v2_sarjeta; crop v2_sarjeta 600x200+340+60
xdotool mousemove 557 116 click 1; hover 432 132 3.5; shot v3_lampada; crop v3_lampada 700x220+340+60
# 2. clicar «Quick Fix...» dentro do balão
hover 557 117 2; xdotool mousemove 700 219; sleep 0.6; shot v4_sobre_link; crop v4_sobre_link 800x300+350+0
xdotool click 1; sleep 1.5; shot v5_menu_do_balao; crop v5_menu_do_balao 800x320+350+0
amostra v5_menu_do_balao 1000 218 "o balão continua aberto?"; amostra v5_menu_do_balao 900 272 "1.ª acção (menu aberto pelo RATO)"
amostra v5_menu_do_balao 683 262 "borda interna da 1.ª acção (anel?)"; amostra v5_menu_do_balao 680 262 "fundo do menu"
xdotool key Escape; sleep 1; shot v6_esc1; amostra v6_esc1 1000 218 "balão após 1.º Esc"
xdotool key Escape; sleep 1; shot v6_esc2; amostra v6_esc2 1000 218 "balão após 2.º Esc"
# 2b. aplicar pela lista aberta a partir do balão
hover 557 117 2; xdotool mousemove 700 219; sleep 0.5; xdotool click 1; sleep 1.5
xdotool mousemove 800 272; sleep 0.5; xdotool click 1; sleep 2; shot v7_aplicado; crop v7_aplicado 800x200+350+0
amostra v7_aplicado 1000 218 "balão depois de aplicar"
# 3. Ctrl+. pelo TECLADO, num erro com várias correcções
xdotool key ctrl+p; sleep 0.8; xdotool type --delay 40 varios.ts; sleep 0.8; xdotool key Return; sleep 4
hover 443 136 2.2; shot v8_balao_varios; crop v8_balao_varios 800x260+350+0
xdotool mousemove 443 136 click 1; xdotool mousemove 1200 800; sleep 1.2
xdotool key ctrl+period; sleep 2; shot v9_ctrl_ponto; crop v9_ctrl_ponto 300x220+430+140
amostra v9_ctrl_ponto 650 160 "fundo do menu"; amostra v9_ctrl_ponto 650 187 "1.ª acção pré-seleccionada (enchimento)"
amostra v9_ctrl_ponto 650 176 "1.ª acção, 1 px abaixo do topo (anel)"; amostra v9_ctrl_ponto 446 187 "1.ª acção, borda esquerda (anel)"
amostra v9_ctrl_ponto 705 211 "2.ª acção (não seleccionada)"
hover 602 211 1.2; shot v10_rato_na_2a; crop v10_rato_na_2a 300x220+430+140
amostra v10_rato_na_2a 650 187 "1.ª acção com o rato na 2.ª"; amostra v10_rato_na_2a 650 220 "2.ª acção sob o rato"
amostra v10_rato_na_2a 446 211 "2.ª acção, borda esquerda (anel?)"
xdotool mousemove 1200 800; sleep 0.8; xdotool key Down; sleep 0.8; shot v11_seta_abaixo; crop v11_seta_abaixo 300x220+430+140
amostra v11_seta_abaixo 650 260 "3.ª acção após ↓ (enchimento)"; amostra v11_seta_abaixo 446 260 "3.ª acção após ↓, borda (anel)"
xdotool key Escape; sleep 0.5
xdotool windowactivate --sync "$W" 2>/dev/null || true; xdotool key ctrl+q; sleep 4
kill "$CODE_TIMEOUT_PID" 2>/dev/null || true   # o timeout passa o TERM ao filho, se o Ctrl+Q não bastou

# ───────────────────────── Godot ─────────────────────────
G="$T/gd"; mkdir -p "$G/proj" "$G/home" "$G/cfg" "$G/data" "$G/cache"
printf 'config_version=5\n\n[application]\nconfig/name="oraculo_i8b"\n\n[rendering]\nrenderer/rendering_method="gl_compatibility"\n' >"$G/proj/project.godot"
printf 'extends Node\n\nfunc _ready():\n\tpirnt(1)\n\tvar amplitude = 1\n\tprint(amplitud)\n' >"$G/proj/err.gd"
env -i HOME="$G/home" PATH=/usr/bin:/bin USER="$USER" LANG=en_US.UTF-8 DISPLAY="$D" XDG_RUNTIME_DIR="$T" \
  XDG_CONFIG_HOME="$G/cfg" XDG_DATA_HOME="$G/data" XDG_CACHE_HOME="$G/cache" \
  timeout 600 godot --display-driver x11 --rendering-driver opengl3 -e --path "$G/proj" >"$T/godot.log" 2>&1 &
GODOT_TIMEOUT_PID=$!
for _ in $(seq 40); do GW=$(xdotool search --class godot 2>/dev/null | tail -1 || true); [ -n "$GW" ] && break; sleep 1; done
[ -n "$GW" ] || { echo "Godot não abriu" >&2; cat "$T/godot.log" >&2; exit 1; }
sleep 10
xdotool windowmove "$GW" 0 0 windowsize "$GW" 1600 1000; sleep 4
xdotool mousemove 80 690 click --repeat 2 --delay 80 1; sleep 4        # err.gd no FileSystem
xdotool mousemove 900 284 click 1; sleep 0.5                           # foco no editor (sem ele, nenhum balão)
hover 650 259 3; shot g1_balao_print; crop g1_balao_print 900x300+490+110   # controlo: o balão de doc APARECE
hover 656 209 3; shot g2_palavra_pirnt; crop g2_palavra_pirnt 760x230+490+110
hover 700 259 3; shot g3_palavra_amplitud
hover 607 209 3; shot g4_marca_sarjeta
hover 570 209 3; shot g5_numero_linha
amostra g2_palavra_pirnt 1000 400 "fundo do editor"; amostra g2_palavra_pirnt 1000 209 "linha com erro (4)"
amostra g2_palavra_pirnt 1000 234 "linha 5 (aviso)"
hover 994 926 2; shot g6_contador; xdotool click 1; sleep 1.5; xdotool mousemove 1200 700; sleep 0.5
shot g7_lista_erros; crop g7_lista_erros 820x120+490+800
xdotool mousemove 656 209 click 3; sleep 1.2; shot g8_menu_direito; crop g8_menu_direito 500x320+600+180; xdotool key Escape; sleep 0.5
xdotool mousemove 900 284 click 1; sleep 0.3; xdotool key End Tab; xdotool type --delay 80 pri; sleep 0.5
xdotool key ctrl+space; sleep 2; xdotool mousemove 1300 800; sleep 0.8; shot g9_completar; crop g9_completar 460x230+600+280
amostra g9_completar 880 307 "1.ª linha seleccionada"; amostra g9_completar 880 332 "2.ª linha"; amostra g9_completar 880 300 "topo da 1.ª (anel?)"
amostra g9_completar 636 312 "borda esquerda do popup"
hover 882 357 1.2; shot g10_rato_na_3a; crop g10_rato_na_3a 460x230+600+280
amostra g10_rato_na_3a 880 307 "1.ª com o rato na 3.ª"; amostra g10_rato_na_3a 880 357 "3.ª sob o rato"
# só as linhas da lista (y ≥ 20 no recorte): acima está o cursor de texto, que pisca
echo "rato sobre o popup muda pixels da lista? AE=$(magick compare -metric AE \
  <(magick "$OUT/g9_completar_c.png" -crop 460x210+0+20 +repage png:-) \
  <(magick "$OUT/g10_rato_na_3a_c.png" -crop 460x210+0+20 +repage png:-) null: 2>&1)" >>"$OUT/amostras.txt"
xdotool mousemove 1300 800; sleep 0.5; xdotool key Down; sleep 0.8; shot g11_seta_abaixo; crop g11_seta_abaixo 460x230+600+280
amostra g11_seta_abaixo 880 307 "1.ª após ↓"; amostra g11_seta_abaixo 880 332 "2.ª após ↓"
kill "$GODOT_TIMEOUT_PID" 2>/dev/null || true
sleep 2
echo "gravado em $OUT"; cat "$OUT/amostras.txt"
