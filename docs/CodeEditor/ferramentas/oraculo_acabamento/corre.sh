#!/usr/bin/env bash
# Oráculo «acabamento» (CodeEditor): VS Code 1.142.0 e Godot 4.7.2, numa corrida só (CLAUDE.md §0.10):
#   (A) Ctrl + rato PARADO sobre uma palavra → o ponteiro vira MÃO? a palavra sublinha? que intervalo, que cor,
#       com que atraso, e em que alvos (variável, declaração, propriedade, API, nome com erro, palavra-chave,
#       comentário, string, espaço);
#   (B) a lista de problemas: o contador (barra de estado / fundo do editor de script), o que o clique abre,
#       Ctrl+Shift+M, cada linha, a ORDEM, clique / foco / teclado, actualização ao corrigir, lista vazia;
#   (C) o leitor de ecrã: o nome acessível e o papel (AT-SPI) de cada linha da lista e do contador.
# O produto é CORRIDO e FOTOGRAFADO, nunca lido (CLAUDE.md §0.9).
#
# Licenças: VS Code = binário Microsoft (núcleo Code-OSS MIT), só o corremos; Godot = MIT (pacman -Qi godot).
#
# Uso (da raiz da worktree):
#   PH2D_GPU=1 bash scripts/ph2d-run.sh bash docs/CodeEditor/ferramentas/oraculo_acabamento/corre.sh <saida>
#   Grava em <saida>: *_c.png (recortes), amostras.txt, a11y_vscode.txt, a11y_godot.txt.
#   As fotos de ecrã inteiro (raw) ficam numa pasta /tmp/oraculo_acab_raw.XXXX (dita no fim) — não entram no repo.
#   ACAB_SO="vs vs11y gd" escolhe as fases (por omissão as três).
#
# ⛔ Regras (copiadas do oraculo_i8b/corre.sh — o dono já foi prejudicado por cada uma):
#   - Xvfb PRIVADO :118 (recusa :0); foto = `import -window root` (NUNCA `spectacle`); xdotool no Xvfb.
#     O `import` NÃO fotografa o cursor do rato: a régua dele é o cursor_nome.c (XFixes: nome, tamanho,
#     hotspot e hash dos pixels; o nome vem VAZIO nestes dois apps, por isso a tabela de equivalência
#     hash → I-beam / mão / seta é medida no início, sobre texto, item da árvore e menu, e gravada como imagem).
#   - O VS Code do dono está aberto: o `code` corre com `env -i`, HOME / --user-data-dir / --extensions-dir
#     próprios e XDG_RUNTIME_DIR curto em /tmp. Godot com XDG_* próprios, x11 + opengl3.
#   - Tudo corre numa sessão dbus PRIVADA (dbus-run-session com uma configuração SEM pastas de serviços: com
#     as de série, o bus activava xdg-desktop-portal-kde e ksecretd, que ficavam ÓRFÃOS do systemd --user
#     com DISPLAY=:118 depois da corrida — medido 19/10). O bus do AT-SPI e o registryd lançam-se à mão.
#   - Cada processo sob `timeout`; pára-se pelo PID que ESTE script lançou.
#
# Coordenadas: ecrã 1600x1000, janela em (0,0), leiaute de fábrica. VS Code: linha n em y = 97 + 19(n-1),
# coluna c em x = 423 + 8,4c. Godot: linha n em y = 134 + 25(n-1), coluna c em x = 604 + 8c (tab = 4).
set -euo pipefail

D=":118"
[ "${ACAB_DISPLAY:-$D}" = ":0" ] && { echo "recuso: :0 é o ecrã do dono" >&2; exit 2; }
D="${ACAB_DISPLAY:-$D}"
AQUI=$(cd "$(dirname "$0")" && pwd)

# ── fora: a sessão dbus privada ─────────────────────────────────────────────────────────────────────
if [ -z "${ACAB_DENTRO:-}" ]; then
  OUT="${1:-$(mktemp -d /tmp/oraculo_acab.XXXX)}"; mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
  RAW=$(mktemp -d /tmp/oraculo_acab_raw.XXXX)
  T=$(mktemp -d /tmp/acb.XXXX); chmod 700 "$T"     # curto de propósito (socket do VS Code ≤ 107 chars)
  cat >"$T/bus.conf" <<'BC'
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN" "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig><type>session</type><listen>unix:tmpdir=/tmp</listen><auth>EXTERNAL</auth>
<policy context="default"><allow send_destination="*" eavesdrop="true"/><allow eavesdrop="true"/><allow own="*"/></policy></busconfig>
BC
  rc=0
  ACAB_DENTRO=1 ACAB_T="$T" ACAB_RAW="$RAW" timeout 1500 \
    dbus-run-session --config-file="$T/bus.conf" -- bash "$0" "$OUT" || rc=$?
  rm -rf "$T"
  echo "gravado em $OUT ; fotos de ecrã inteiro em $RAW (rc=$rc)"
  exit "$rc"
fi

# ── dentro ─────────────────────────────────────────────────────────────────────────────────────────
OUT="$1"; T="$ACAB_T"; RAW="$ACAB_RAW"
SO="${ACAB_SO:-vs vs11y gd}"
quer() { [[ " $SO " == *" $1 "* ]]; }
export DISPLAY="$D" XDG_RUNTIME_DIR="$T"
unset WAYLAND_DISPLAY
A="$OUT/amostras.txt"

PIDS=()
limpa() { local i; for ((i=${#PIDS[@]}-1; i>=0; i--)); do kill "${PIDS[$i]}" 2>/dev/null || true; done; }
trap limpa EXIT

gcc -O2 -o "$T/cursor_nome" "$AQUI/cursor_nome.c" -lX11 -lXfixes
shot() { import -window root "$RAW/$1.png"; }
crop() { magick "$RAW/$1.png" -crop "$2" +repage "$OUT/$1_c.png"; }
px()   { magick "$RAW/$1.png" -format "%[hex:p{$2,$3}]" info:; }
fato() { echo "$*" >>"$A"; }
amostra() { fato "$1 ($2,$3) #$(px "$1" "$2" "$3") — $4"; }
# editor (linhas 1–9) + painel + lado direito da barra de estado (Ln/Col), num recorte só
trio() { magick "$RAW/$1.png" \( -clone 0 -crop 950x180+353+82 +repage \) \( -clone 0 -crop 950x300+353+660 +repage \) \( -clone 0 -crop 950x28+650+972 +repage \) -delete 0 -append "$OUT/$1_c.png"; }
declare -A NOME_CURSOR=()
cur() { local h; h=$("$T/cursor_nome" | sed 's/.*hash=//'); echo "${NOME_CURSOR[$h]:-outro:$h}"; }
longe() { xdotool mousemove "$1" "$2"; sleep "${3:-0.8}"; }
# a altura do painel muda entre corridas (medido: a mesma linha a y=786 e a y=755): as linhas da lista acham-se por
# OCR da foto (tesseract, texto invertido e 2×) — devolve o y do centro da palavra no ecrã, ou vazio
acha_y() { # foto palavra
  magick "$RAW/$1.png" -crop 950x330+353+640 +repage -negate -resize 200% "$T/ocr.png"
  tesseract "$T/ocr.png" - --psm 11 tsv 2>/dev/null | awk -F'\t' -v w="$2" '$12==w {print int(640 + ($8 + $10 / 2) / 2); exit}'
}
pousa() { xdotool mousemove $(($1 - 2)) "$2"; sleep 0.2; xdotool mousemove "$1" "$2"; sleep "${3:-1.5}"; }

{
  echo "# oráculo acabamento (Ctrl+rato · lista de problemas · AT-SPI) — $(date -u +%Y-%m-%dT%H:%MZ) — loadavg: $(cat /proc/loadavg)"
  echo "# code --version: $(code --version | paste -sd ' ' -)"
  echo "# godot: $(LC_ALL=C pacman -Q godot)"
  echo "# pacman: $(LC_ALL=C pacman -Qi godot visual-studio-code-bin 2>/dev/null | grep -E '^(Name|Version|Licenses) ' | sed 's/ *: /: /' | paste -sd ';' -)"
  echo "# licenças: VS Code = binário Microsoft, só CORRIDO; Godot = MIT"
  echo "# display: Xvfb $D 1600x1000x24 ; dbus privado sem serviços; fotos de ecrã inteiro em $RAW"
  echo "# estados do (A): 0sem = rato parado 1,5 s sem Ctrl · 1k01 = 0,1 s depois de keydown ctrl (rato parado)"
  echo "#   · 2k15 = 1,5 s depois · 3mexeu = Ctrl carregado e +2 px · 4solto = 0,6 s depois de keyup · depois = +1 px sem Ctrl"
} >"$A"

timeout 1400 Xvfb "$D" -screen 0 1600x1000x24 -nolisten tcp >"$T/xvfb.log" 2>&1 &
PIDS+=($!); sleep 1.5
timeout 1350 /usr/lib/at-spi-bus-launcher --launch-immediately >"$T/atspi.log" 2>&1 &
PIDS+=($!); sleep 1.5
timeout 1350 /usr/lib/at-spi2-registryd >"$T/registryd.log" 2>&1 &
PIDS+=($!); sleep 1
gdbus call --session --dest org.a11y.Bus --object-path /org/a11y/bus \
  --method org.freedesktop.DBus.Properties.Set org.a11y.Status IsEnabled '<true>' >>"$T/atspi.log" 2>&1 || true
fato "# at-spi: IsEnabled=$(gdbus call --session --dest org.a11y.Bus --object-path /org/a11y/bus --method org.freedesktop.DBus.Properties.Get org.a11y.Status IsEnabled 2>&1) ; bus $(gdbus call --session --dest org.a11y.Bus --object-path /org/a11y/bus --method org.a11y.Bus.GetAddress 2>&1)"

a11y() { (cd /tmp && timeout 120 python3 -I "$AQUI/a11y_despejo.py" "$1") || echo "despejo falhou rc=$?"; }

# (A) um alvo: as 5 fotos + cursores, e a régua da faixa da linha
alvo_A() { # app alvo x y texto_da_linha meia x0 cw longe_x longe_y
  local app=$1 alvo=$2 x=$3 y=$4 linha=$5 meia=$6 x0=$7 cw=$8 lx=$9 ly=${10} c0 c1 c2 c3 c4 c5
  longe "$lx" "$ly" 1.5; shot "${app}_${alvo}_ref"   # o balão do alvo anterior já se foi
  pousa "$x" "$y" 1.5
  c0=$(cur); shot "${app}_${alvo}_0sem"
  xdotool keydown ctrl; sleep 0.1; c1=$(cur); shot "${app}_${alvo}_1k01"
  sleep 1.4; c2=$(cur); shot "${app}_${alvo}_2k15"
  xdotool mousemove $((x + 2)) "$y"; sleep 0.8; c3=$(cur); shot "${app}_${alvo}_3mexeu"
  xdotool keyup ctrl; sleep 0.6; c4=$(cur); shot "${app}_${alvo}_4solto"
  xdotool mousemove $((x + 3)) "$y"; sleep 0.5; c5=$(cur)
  python3 -I "$AQUI/analisa.py" "$RAW" "$OUT" "$app" "$alvo" "$linha" "$y" "$meia" "$x0" "$cw" "$x" \
    "0sem=$c0" "1k01=$c1" "2k15=$c2" "3mexeu=$c3" "4solto=$c4" >>"$A" || fato "$app A $alvo: a régua falhou"
  fato "$app A $alvo depois (sem Ctrl, +1 px): cursor=$c5 — prova: ${app}_A_${alvo}_c.png"
}
# o RELÓGIO do ponteiro: o cursor do X amostrado em contínuo depois do keydown e do keyup (rato parado)
trajeto() { # app alvo x y longe_x longe_y
  local t0 r="" i
  longe "$5" "$6"; pousa "$3" "$4" 1.5
  xdotool keydown ctrl; t0=$(date +%s%N)
  for i in $(seq 40); do r+=" $(( ($(date +%s%N) - t0) / 1000000 ))ms=$(cur)"; sleep 0.04; done
  fato "$1 A $2 relógio do cursor depois do keydown ctrl (rato parado):$r"
  xdotool keyup ctrl; t0=$(date +%s%N); r=""
  for i in $(seq 25); do r+=" $(( ($(date +%s%N) - t0) / 1000000 ))ms=$(cur)"; sleep 0.04; done
  fato "$1 A $2 relógio do cursor depois do keyup (rato parado):$r"
  xdotool mousemove $(($3 + 1)) "$4"; sleep 0.3; fato "$1 A $2 depois de mexer 1 px sem Ctrl: $(cur)"
}

# ───────────────────────── VS Code ─────────────────────────
vs_abre() { # pasta a11y(on|off) extra-flag
  local V="$T/$1"; mkdir -p "$V/home" "$V/ud/User" "$V/ext" "$V/ws"
  printf 'class Ponto { x = 0; }\nconst amplitude = 1;\nconst p = new Ponto();\np.x = amplitude;\nconsole.log(amplitud);\n// amplitude no comentário\nconst s = "amplitude";\np.y = 2;\n' >"$V/ws/a.ts"
  printf 'a { colr: red; }\nb { }\nc { color: ; }\n' >"$V/ws/w.css"
  cat >"$V/ud/User/settings.json" <<EOF
{ "workbench.startupEditor": "none", "telemetry.telemetryLevel": "off", "update.mode": "none",
  "extensions.autoCheckUpdates": false, "extensions.autoUpdate": false, "workbench.tips.enabled": false,
  "security.workspace.trust.enabled": false, "window.restoreWindows": "none", "files.hotExit": "off",
  "chat.disableAIFeatures": true, "workbench.enableExperiments": false,
  "editor.accessibilitySupport": "$2" }
EOF
  env -i HOME="$V/home" PATH=/usr/bin:/bin USER="$USER" LANG=en_US.UTF-8 DISPLAY="$D" XDG_RUNTIME_DIR="$T" \
    DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" \
    timeout 600 /usr/share/code/code --user-data-dir="$V/ud" --extensions-dir="$V/ext" --new-window \
    --disable-workspace-trust --disable-gpu --password-store=basic --ozone-platform=x11 ${3:-} \
    --skip-welcome --skip-release-notes "$V/ws" "$V/ws/a.ts" "$V/ws/w.css" >"$T/code_$1.log" 2>&1 &
  CODE_PID=$!; PIDS+=("$CODE_PID")
  W=""
  for _ in $(seq 40); do W=$(xdotool search --name 'Visual Studio Code' 2>/dev/null | head -1 || true); [ -n "$W" ] && break; sleep 1; done
  [ -n "$W" ] || { echo "VS Code não abriu" >&2; cat "$T/code_$1.log" >&2; exit 1; }
  xdotool windowmove "$W" 0 0 windowsize "$W" 1600 1000
  sleep 8                                                           # o servidor TS sobe
  xdotool mousemove 480 50 click 1; sleep 3                         # w.css visível uma vez: o CSS valida
  xdotool mousemove 390 50 click 1; sleep 2                         # volta ao a.ts
}
vs_fecha() { xdotool windowactivate --sync "$W" 2>/dev/null || true; xdotool key ctrl+q; sleep 3; kill "$CODE_PID" 2>/dev/null || true; sleep 2; }
vs_comando() { xdotool key ctrl+shift+p; sleep 0.8; xdotool type --delay 30 "$1"; sleep 1; xdotool key Return; sleep 1.5; }

VS_L=(
  ''
  'class Ponto { x = 0; }'
  'const amplitude = 1;'
  'const p = new Ponto();'
  'p.x = amplitude;'
  'console.log(amplitud);'
  '// amplitude no comentário'
  'const s = "amplitude";'
  'p.y = 2;'
)
vs_A() { # app alvo linha coluna comprimento   (x ao meio da palavra)
  local x y; x=$(python3 -c "print(round(423 + 8.4 * ($4 + $5 / 2)))"); y=$((97 + 19 * ($3 - 1)))
  alvo_A "$1" "$2" "$x" "$y" "${VS_L[$3]}" 9 423 8.4 1200 820
}

if quer vs; then
  vs_abre vs off
  xdotool mousemove 1200 820; sleep 1; shot v0_inicio; crop v0_inicio 950x200+353+60
  # tabela de equivalência dos cursores (o nome XFixes vem vazio)
  for ref in "507 154 ibeam texto" "94 107 mao item-da-arvore" "53 18 seta menu-File"; do
    set -- $ref; xdotool mousemove "$1" "$2"; sleep 0.7
    l=$("$T/cursor_nome" "$T/cur_$3.pam"); NOME_CURSOR[${l##*hash=}]=$3
    magick "$T/cur_$3.pam" -background '#808080' -flatten -scale 400% "$T/cur_$3.png"
    fato "cursor de referência $3 (sobre $4 em $1,$2): $l"
  done
  magick "$T/cur_ibeam.png" "$T/cur_mao.png" "$T/cur_seta.png" +append "$OUT/cursor_ref_c.png"
  fato "prova da tabela: cursor_ref_c.png (I-beam · mão · seta, 4×)"

  # (A)
  vs_A vs amplitude_l4   4 6 9
  vs_A vs amplitude_decl 2 6 9
  vs_A vs x_prop         4 2 1
  vs_A vs y_inexistente  8 2 1
  vs_A vs p_local        4 0 1
  vs_A vs log_api        5 8 3
  vs_A vs console_api    5 0 7
  vs_A vs Ponto_classe   3 14 5
  vs_A vs amplitud_erro  5 12 8
  vs_A vs const_chave    2 0 5
  vs_A vs comentario     6 3 9
  vs_A vs string         7 11 9
  vs_A vs espaco_entre   4 3 1
  vs_A vs depois_do_fim  4 40 1
  trajeto vs amplitude_l4 473 154 1200 820

  # (B) o contador
  longe 1200 820; shot vb0_barra; crop vb0_barra 400x28+0+972
  pousa 75 984 1.5; fato "vs B contador: cursor sobre ele = $(cur)"; shot vb1_balao_contador; crop vb1_balao_contador 400x80+0+920
  xdotool click 1; sleep 2; longe 1200 300; shot vb2_clique_contador; crop vb2_clique_contador 950x330+353+640
  xdotool type K; sleep 1; shot vb2b_letra_depois_de_abrir; trio vb2b_letra_depois_de_abrir; xdotool key BackSpace; sleep 0.8
  pousa 75 984 0.5; xdotool click 1; sleep 2; longe 1200 300; shot vb3_clique_contador_2; crop vb3_clique_contador_2 950x330+353+640
  xdotool key ctrl+shift+m; sleep 2; shot vb4_ctrl_shift_m; crop vb4_ctrl_shift_m 950x330+353+640
  xdotool type K; sleep 1; shot vb4b_letra_depois_de_ctrl_shift_m; trio vb4b_letra_depois_de_ctrl_shift_m; xdotool key BackSpace; sleep 0.8
  xdotool key ctrl+shift+m; sleep 2; shot vb5_ctrl_shift_m_2; crop vb5_ctrl_shift_m_2 950x330+353+640
  xdotool mousemove 600 400 click 1; sleep 0.5; longe 1200 300           # foco no editor
  xdotool key ctrl+shift+m; sleep 2; shot vb6_ctrl_shift_m_do_editor; crop vb6_ctrl_shift_m_do_editor 950x330+353+640
  xdotool type K; sleep 1; shot vb6b_letra; trio vb6b_letra; xdotool key BackSpace; sleep 0.8
  vs_comando "View: Focus on Problems View"; longe 1200 300; shot vb7_painel; crop vb7_painel 950x330+353+640
  # o (A) com o foco do teclado na lista (não no editor)
  vs_A vs amplitude_l4_foco_na_lista 4 6 9
  # clique simples numa linha (Property 'y', a 4.ª do painel), e uma letra a seguir
  PY=$(acha_y vb7_painel Property); [ -n "$PY" ] || { fato "vs B: OCR não achou «Property» em vb7_painel"; PY=786; }
  fato "vs B: a linha «Property 'y'…» está em y=$PY (OCR de vb7_painel); a 1.ª linha de problema, 2 linhas acima, em y=$((PY - 44))"
  xdotool mousemove 600 "$PY" click 1; sleep 1.5; longe 1200 300; shot vb8_clique_linha; trio vb8_clique_linha
  xdotool type Z; sleep 1; shot vb9_letra_depois_do_clique; trio vb9_letra_depois_do_clique
  xdotool key BackSpace; sleep 1; shot vb10_backspace; trio vb10_backspace
  # teclado: ↓ ↓ ↑ e Enter a partir da 1.ª linha de problema
  xdotool mousemove 600 $((PY - 44)) click 1; sleep 1.2; longe 1200 300; shot vb11_l1; trio vb11_l1
  xdotool key Down; sleep 1.2; shot vb12_seta_baixo; trio vb12_seta_baixo
  xdotool key Down; sleep 1.2; shot vb13_seta_baixo_2; trio vb13_seta_baixo_2
  xdotool key Up; sleep 1.2; shot vb14_seta_cima; trio vb14_seta_cima
  xdotool key Return; sleep 1.5; shot vb15_enter; trio vb15_enter
  xdotool type Q; sleep 1; shot vb16_letra_depois_do_enter; trio vb16_letra_depois_do_enter
  xdotool key ctrl+z; sleep 0.8
  # corrigir `amplitud` a escrever (Ctrl+G 5:21 → «e»): a lista actualiza sozinha?
  xdotool key ctrl+g; sleep 0.6; xdotool type --delay 40 "5:21"; xdotool key Return; sleep 0.8
  xdotool type e; t0=$(date +%s%N)
  shot vb17a_corrige_0s; t1=$(( ($(date +%s%N) - t0) / 1000000 ))
  sleep 0.15; shot vb17b_corrige_02s; t2=$(( ($(date +%s%N) - t0) / 1000000 ))
  sleep 0.25; shot vb17_corrige_05s; t3=$(( ($(date +%s%N) - t0) / 1000000 ))
  sleep 1.5; shot vb18_corrige_2s; t4=$(( ($(date +%s%N) - t0) / 1000000 ))
  for f in vb17a_corrige_0s vb17b_corrige_02s vb17_corrige_05s vb18_corrige_2s; do trio $f; done
  fato "vs B correcção («e» em amplitud): fotos a ${t1} ms (vb17a), ${t2} ms (vb17b), ${t3} ms (vb17), ${t4} ms (vb18) depois da tecla"
  # lista vazia: p.y → p.x e o w.css limpo
  xdotool key ctrl+g; sleep 0.6; xdotool type --delay 40 "8:3"; xdotool key Return; sleep 0.5
  xdotool key shift+Right; xdotool type x; sleep 0.5
  xdotool key ctrl+p; sleep 0.8; xdotool type --delay 40 w.css; sleep 0.8; xdotool key Return; sleep 1.5
  xdotool key ctrl+a; xdotool type --delay 20 'a { color: red; }'; sleep 4
  longe 1200 300; shot vb19_vazia; crop vb19_vazia 950x330+353+640
  magick "$RAW/vb19_vazia.png" -crop 400x28+0+972 +repage "$OUT/vb19_vazia_barra_c.png"
  vs_fecha
fi

if quer vs11y; then
  # (C) VS Code com o leitor de ecrã: accessibilitySupport on + árvore do Chromium forçada
  vs_abre vs2 on --force-renderer-accessibility
  for ref in "507 154 ibeam" "94 107 mao" "53 18 seta"; do
    set -- $ref; xdotool mousemove "$1" "$2"; sleep 0.7; l=$("$T/cursor_nome"); NOME_CURSOR[${l##*hash=}]=$3
  done
  # o modo leitor de ecrã muda o (A)? dois alvos de controlo, com o foco no editor (antes de abrir a lista)
  vs_A vs11y amplitude_l4 4 6 9
  vs_A vs11y comentario   6 3 9
  pousa 75 984 0.5; xdotool click 1; sleep 2.5; longe 1200 300; shot vc0_painel_a11y; crop vc0_painel_a11y 950x330+353+640
  a11y "$RAW/a11y_vscode_inteiro.txt"
  {
    echo "# AT-SPI do VS Code (accessibilitySupport=on, --force-renderer-accessibility), depois de clicar no contador"
    echo "# só os nós do separador Problems, da árvore de problemas e do contador da barra; árvore inteira em $RAW/a11y_vscode_inteiro.txt"
    echo "# formato: papel «nome acessível» desc«descrição» [x,y LxA] {estados}"
    head -1 "$RAW/a11y_vscode_inteiro.txt"
    grep -E 'page tab «Problems|tree «Showing|section «Showing|tree item «(Error|Warning|Info|[0-9]+ problems? in file|.* at line )|(button|section) «Errors:|status bar «' "$RAW/a11y_vscode_inteiro.txt" | sed 's/^ *//' || echo "(nenhum nó)"
  } >"$OUT/a11y_vscode.txt"
  # o foco depois do clique numa linha, neste modo (o despejo diz quem tem «focused»)
  # a linha «Property 'y'» pela CAIXA que o próprio AT-SPI dá
  PY=$(grep -m1 "tree item «Error: Property 'y'" "$RAW/a11y_vscode_inteiro.txt" | sed -E 's/.*\[[0-9]+,([0-9]+) [0-9]+x([0-9]+)\].*/\1 \2/' | awk '{print $1 + int($2 / 2)}')
  xdotool mousemove 600 "${PY:-786}" click 1; sleep 1.5; a11y "$RAW/a11y_vscode_clique.txt"
  fato "vs11y B depois do clique na linha «Property 'y'»: nós focados = $(grep -c 'focused' "$RAW/a11y_vscode_clique.txt" || true): $(grep 'focused' "$RAW/a11y_vscode_clique.txt" | sed 's/^ *//' | cut -c1-200 | paste -sd '|' -)"
  vs_fecha
fi

# ───────────────────────── Godot ─────────────────────────
GD_L=(
  ''
  'extends Node'
  'var amplitude = 1'
  'func _ready():'
  '    var p = Vector2()'
  '    p.x = amplitude'
  '    print(amplitud)'
  '    # amplitude'
  '    var s = "amplitude"'
  '    var nao_usada = 3'
)
gd_A() { # alvo linha coluna comprimento (coluna com tab = 4)
  local x y; x=$((604 + 8 * $3 + 4 * $4)); y=$((134 + 25 * ($2 - 1)))
  alvo_A gd "$1" "$x" "$y" "${GD_L[$2]}" 11 604 8 1450 700   # longe = o Inspector (fora do editor)
}
if quer gd; then
  G="$T/gd"; mkdir -p "$G/proj" "$G/home" "$G/cfg" "$G/data" "$G/cache"
  printf 'config_version=5\n\n[application]\nconfig/name="oraculo_acab"\n\n[rendering]\nrenderer/rendering_method="gl_compatibility"\n' >"$G/proj/project.godot"
  printf 'extends Node\nvar amplitude = 1\nfunc _ready():\n\tvar p = Vector2()\n\tp.x = amplitude\n\tprint(amplitud)\n\t# amplitude\n\tvar s = "amplitude"\n\tvar nao_usada = 3\n' >"$G/proj/err.gd"
  # LD_DEBUG=libs: a prova de que o Godot procura a biblioteca do AccessKit (e se a encontra)
  env -i HOME="$G/home" PATH=/usr/bin:/bin USER="$USER" LANG=en_US.UTF-8 DISPLAY="$D" XDG_RUNTIME_DIR="$T" \
    DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" LD_DEBUG=libs \
    XDG_CONFIG_HOME="$G/cfg" XDG_DATA_HOME="$G/data" XDG_CACHE_HOME="$G/cache" \
    timeout 600 godot --display-driver x11 --rendering-driver opengl3 --accessibility always --accessibility-driver accesskit \
    -e --path "$G/proj" >"$T/godot.log" 2>&1 &
  GODOT_PID=$!; PIDS+=("$GODOT_PID")
  GW=""
  for _ in $(seq 40); do GW=$(xdotool search --class godot 2>/dev/null | tail -1 || true); [ -n "$GW" ] && break; sleep 1; done
  [ -n "$GW" ] || { echo "Godot não abriu" >&2; tail -30 "$T/godot.log" >&2; exit 1; }
  sleep 10
  xdotool windowmove "$GW" 0 0 windowsize "$GW" 1600 1000; sleep 4
  xdotool mousemove 80 690 click --repeat 2 --delay 80 1; sleep 4        # err.gd no FileSystem (único ficheiro)
  xdotool mousemove 1100 650 click 1; sleep 0.5                          # foco no editor (cursor na linha 10)
  for ref in "720 234 ibeam texto" "1000 926 mao contador" "1410 22 seta botao-play"; do
    set -- $ref; xdotool mousemove "$1" "$2"; sleep 0.7; l=$("$T/cursor_nome")
    fato "godot: cursor de referência sobre $4 ($1,$2): ${NOME_CURSOR[${l##*hash=}]:-NOVO} — $l"
    [ -n "${NOME_CURSOR[${l##*hash=}]:-}" ] || NOME_CURSOR[${l##*hash=}]=$3
  done
  longe 1100 650; shot g0_inicio; crop g0_inicio 830x260+490+115

  gd_A amplitude_l5   5 10 9
  gd_A amplitude_decl 2 4 9
  gd_A x_prop         5 6 1
  gd_A p_local        5 4 1
  gd_A print_api      6 4 5
  gd_A Vector2_api    4 12 7
  gd_A Node_classe    1 8 4
  gd_A amplitud_erro  6 10 8
  gd_A var_chave      4 4 3
  gd_A func_chave     3 0 4
  gd_A comentario     7 6 9
  gd_A string         8 13 9
  gd_A espaco_entre   5 7 1
  gd_A depois_do_fim  5 40 1
  trajeto gd amplitude_l5 720 234 1100 650

  # (B) o contador no fundo do editor de script
  longe 1100 650; shot gb0_fundo; crop gb0_fundo 830x60+490+895
  pousa 1000 926 1.5; fato "gd B contador de erros: cursor sobre ele = $(cur)"; shot gb1_balao_contador; crop gb1_balao_contador 830x200+490+760
  xdotool click 1; sleep 1.5; longe 1100 650; shot gb2_lista_erros; crop gb2_lista_erros 830x220+490+760
  # para onde vai uma letra depois de abrir a lista pelo clique? (BackSpace desfaz, esteja ela onde estiver)
  xdotool type K; sleep 1; shot gb2b_letra; crop gb2b_letra 830x830+490+120
  fato "gd B letra «K» depois de clicar no contador: pixels mudados no ecrã inteiro = $(magick compare -metric AE "$RAW/gb2_lista_erros.png" "$RAW/gb2b_letra.png" null: 2>&1 || true) (gb2b_letra_c.png)"
  xdotool key BackSpace; sleep 0.5
  pousa 1000 823 0.5; xdotool click 1; sleep 1.5; longe 1100 650; shot gb3_clique_de_novo; crop gb3_clique_de_novo 830x220+490+760
  pousa 1048 926 0.5; xdotool click 1; sleep 1.5; longe 1100 650; shot gb4_lista_avisos; crop gb4_lista_avisos 830x300+490+680
  a11y "$RAW/a11y_godot_inteiro.txt"
  pousa 1000 823 0.5; xdotool click 1; sleep 1.5; longe 1100 650; shot gb5_erros_com_avisos_abertos; crop gb5_erros_com_avisos_abertos 830x300+490+680
  pousa 524 866 0.6; fato "gd B «Line 6:» cursor sobre o link = $(cur)"
  xdotool mousemove 700 866; sleep 0.6; fato "gd B resto do texto da linha: cursor = $(cur)"
  xdotool mousemove 524 866 click 1; sleep 1; longe 1100 650; shot gb6_clique_link; crop gb6_clique_link 830x830+490+120
  xdotool type Q; sleep 1; shot gb7_letra_depois_do_link; crop gb7_letra_depois_do_link 830x830+490+120
  fato "gd B letra «Q» depois do clique em «Line 6:»: pixels mudados no ecrã inteiro = $(magick compare -metric AE "$RAW/gb6_clique_link.png" "$RAW/gb7_letra_depois_do_link.png" null: 2>&1 || true) (gb7_letra_depois_do_link_c.png)"
  xdotool key BackSpace; sleep 0.5
  # corrigir `amplitud`: clique entre o «d» e o «)» (coluna 18 da linha 6) e «e»; Esc fecha a lista de sugestões
  xdotool mousemove 748 259 click 1; sleep 0.4; xdotool type e; xdotool key Escape; t0=$(date +%s%N)
  shot gb8a_corrige_0s; ta=$(( ($(date +%s%N) - t0) / 1000000 )); crop gb8a_corrige_0s 830x830+490+120
  sleep 0.2; shot gb8b_corrige_02s; tb=$(( ($(date +%s%N) - t0) / 1000000 )); crop gb8b_corrige_02s 830x830+490+120
  sleep 0.2; shot gb8_corrige_05s; t1=$(( ($(date +%s%N) - t0) / 1000000 )); crop gb8_corrige_05s 830x830+490+120
  sleep 1.5; shot gb9_corrige_2s; t2=$(( ($(date +%s%N) - t0) / 1000000 )); crop gb9_corrige_2s 830x830+490+120
  sleep 2; shot gb10_corrige_4s; t3=$(( ($(date +%s%N) - t0) / 1000000 )); crop gb10_corrige_4s 830x830+490+120
  fato "gd B correcção («e» em amplitud): fotos a ${ta} ms (gb8a), ${tb} ms (gb8b), ${t1} ms (gb8), ${t2} ms (gb9), ${t3} ms (gb10) depois da tecla"
  # lista vazia
  xdotool mousemove 1100 650 click 1; sleep 0.3; xdotool key ctrl+a; xdotool type --delay 20 'extends Node'; xdotool key Return; sleep 4
  longe 1100 650; shot gb11_vazio; crop gb11_vazio 830x830+490+120
  pousa 1000 926 0.5; xdotool click 1; sleep 1.5; longe 1100 650; shot gb12_vazio_clique; crop gb12_vazio_clique 830x830+490+120
  {
    echo "# AT-SPI do Godot 4.7.2 (--accessibility always), com a lista de AVISOS aberta"
    echo "# árvore inteira em $RAW/a11y_godot_inteiro.txt"
    head -40 "$RAW/a11y_godot_inteiro.txt"
    echo "# nós com «Line » ou o texto dos problemas:"
    grep -E 'Line [0-9]|UNUSED|amplitud' "$RAW/a11y_godot_inteiro.txt" | sed 's/^ *//' || echo "(nenhum)"
    echo "# a biblioteca do AccessKit (LD_DEBUG=libs do processo do editor):"
    echo "#   linhas LD_DEBUG no registo: $(grep -c 'find library=' "$T/godot.log" || true) «find library»"
    grep -i 'accesskit' "$T/godot.log" | sed 's/^ *[0-9]*://' | sort | uniq -c | head -20 || echo "(nenhuma linha com accesskit)"
    echo "# ficheiros libaccesskit* no sistema: $(ls /usr/lib/libaccesskit* /usr/lib64/libaccesskit* /usr/local/lib/libaccesskit* 2>/dev/null | paste -sd ' ' - || true)"
    echo "# linhas do registo do Godot que falam de acessibilidade:"
    grep -v '^ *[0-9]*:' "$T/godot.log" | grep -i -E 'access|a11y|screen reader' | head -10 || echo "(nenhuma)"
  } >"$OUT/a11y_godot.txt"
  kill "$GODOT_PID" 2>/dev/null || true; sleep 2
  cp "$T/godot.log" "$RAW/godot.log"
fi
echo "fim — $(date -u +%H:%MZ) loadavg $(cat /proc/loadavg)" >>"$A"
