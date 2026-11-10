#!/bin/bash
# uso: bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/mede_pen_up_4k.sh [binario] [rotulo] [WxH] [painel]
# (10/11) O dono: «trava no mouse-up numa imagem 4K, mesmo num traço pequeno». Xvfb + rato real (XTEST, como o
# mede_rato_xvfb.sh) sobre a tela Textured Paper de WxH (PH2D_COMPOSITE_TELA); 6 traços CURTOS, cada um seguido de
# 2 s parado. perf com relógio MONOTÓNICO (-k mono) e o instante de cada mouse-up no mesmo relógio: o resumo
# (mede_pen_up_4k.py) olha só o segundo depois de cada Up. Saída: target/mede-pen-up/<rotulo>.{log,perf,ups,png}.
R="$(cd "$(dirname "$0")/../../.." && pwd)"
S=${SAIDA:-$R/target/mede-pen-up}; mkdir -p "$S"

# ── MATRIZ (10/11, missão 4K, medição «antes»/«depois») ──────────────────────────────────────────────────────────
# uso: bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/mede_pen_up_4k.sh matriz <WxH> <papel|blocos|branco> [binario]
#      [pincel:painel …]   (sem pincéis = a lista PINCEIS abaixo; «omissão:<painel>» = o pincel de fábrica do painel;
#      «Composite Brush:digital» = a pilha da cena, PH2D_COMPOSITE_PILHA=1)
# Um lançamento por pincel, SEM perf; cada quadro com PH2D_QUADRO_LENTO_MS=0 (relógio de parede). Roteiro por fases:
# parado 3 s (CPU de /proc/<pid>/stat) · toque (mousedown parado 0,4 s) · traço (125 eventos × 8 ms ≈ 1 s, 400 px) ·
# soltar · 3 s (Wet: imagens/s) · Ctrl+Z · parado 3 s. `blocos` = a imagem da cena (degradê, grelha, discos) a WxH;
# `branco` = tela branca sem papel (PH2D_COMPOSITE_FUNDO, B08.2). `papel` = Textured Paper,
# relief 0,5 do papel do dono. Saída: $S/matriz/<rotulo>.{log,marcas,meta}; resumo: mede_pen_up_4k.py --matriz $S/matriz.
if [ "$1" = matriz ]; then
  TELA=$2; FUNDO=$3; BIN=${4:-$R/target/smoke/ph2d-host-desktop}; shift 4 2>/dev/null || shift $#
  PINCEIS=("omissão:digital" "omissão:water" "omissão:impasto" "omissão:wet" "Round Mixing:digital"
    "Wet Wash Mix:digital" "Wet Liner:wet" "Watercolor Basic:water" "Pastel:digital" "Chalk:digital" "Crayon:digital"
    "Finger Tip:digital" "Filbert:wet" "Express Oil:wet" "Composite Brush:digital")
  [ $# -gt 0 ] && PINCEIS=("$@")
  M=$S/matriz; mkdir -p "$M"; D=${DISPLAY_MATRIZ:-:97}
  # A PLACA: o app pede HighPerformance = a RTX (medido 10/11: 959 MiB no nvidia-smi sob Xvfb), PROIBIDA na missão
  # 4K. Só o Vulkan da Radeon integrada (RADV) e o GL do Mesa ficam visíveis; sem DRI3 no Xvfb, o RADV apresenta pelo
  # caminho de software do WSI (MESA_VK_WSI_DEBUG=sw). `PLACA=app` devolve a escolha do app (fora da missão).
  PLACA_ENV=(VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json
    MESA_VK_WSI_DEBUG=sw __GLX_VENDOR_LIBRARY_NAME=mesa __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json)
  [ "$PLACA" = app ] && PLACA_ENV=()
  Xvfb $D -screen 0 1600x1000x24 >/dev/null 2>&1 & XV=$!
  sleep 1
  cd "$R"
  for pp in "${PINCEIS[@]}"; do
    PIN=${pp%%:*}; PAINEL=${pp##*:}
    TAG="$(echo "${PIN}_${PAINEL}_${TELA}_${FUNDO}" | tr ' ' '-' | tr -cd 'A-Za-z0-9_.-')"
    EXTRA=(PH2D_COMPOSITE_PILHA=0)
    [ "$PIN" = "Composite Brush" ] && EXTRA=(PH2D_COMPOSITE_PILHA=1)
    [ "$PIN" != omissão ] && [ "$PIN" != "Composite Brush" ] && EXTRA+=("PH2D_COMPOSITE_PRESET=$PIN")
    # B08.2: `PH2D_COMPOSITE_FUNDO` (papel|blocos|branco) e a tela WxH em qualquer fundo; o PAPEL=1 serve o binário antigo.
    EXTRA+=("PH2D_COMPOSITE_FUNDO=$FUNDO" "PH2D_COMPOSITE_TELA=$TELA")
    [ "$FUNDO" = papel ] && EXTRA+=(PH2D_COMPOSITE_PAPEL=1)
    printf '{"pincel":"%s","painel":"%s","tela":"%s","fundo":"%s","carga":"%s"}\n' "$PIN" "$PAINEL" "$TELA" \
      "$FUNDO" "$(cut -d' ' -f1-3 /proc/loadavg)" > "$M/$TAG.meta"
    : > "$M/$TAG.marcas"
    env -u WAYLAND_DISPLAY DISPLAY=$D HOME=$M/home_$TAG PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PAINEL=$PAINEL \
      PH2D_PAINT_PERF=1 PH2D_QUADRO_LENTO_MS=0 PH2D_EXIT_AFTER_FRAMES=1000000 "${PLACA_ENV[@]}" "${EXTRA[@]}" "$BIN" \
      > "$M/$TAG.log" 2>&1 &
    APP=$!
    marca() { echo "$1 $(date +%s%N) $(awk '{print $14+$15}' /proc/$APP/stat 2>/dev/null)" >> "$M/$TAG.marcas"; }
    WIN=
    for _ in $(seq 1 120); do
      grep -q "pilha armada" "$M/$TAG.log" && WIN=$(DISPLAY=$D xdotool search --pid $APP 2>/dev/null | tail -1)
      [ -n "$WIN" ] && break; sleep 0.5
    done
    echo "[matriz] $TAG app $APP win ${WIN:-NENHUMA}"
    if [ -n "$WIN" ]; then
      # A placa que o app abriu (as bibliotecas do driver no mapa do processo): sob Xvfb não há DRI3.
      grep -oE '[^/ ]*(nvidia|radeon|swrast|llvmpipe|lvp|gallium|dri)[^/ ]*\.so[^ ]*' /proc/$APP/maps | sort -u \
        | tr '\n' ' ' > "$M/$TAG.placa"
      DISPLAY=$D xdotool windowsize $WIN 1600 1000 windowmove $WIN 0 0 2>/dev/null; sleep 4
      # Sem gestor de janelas o teclado não segue o rato: sem o foco, o Ctrl+Z não chega (medido 10/11, a foto).
      DISPLAY=$D xdotool windowfocus --sync $WIN 2>/dev/null
      marca parado0_ini; sleep 3; marca parado0_fim
      DISPLAY=$D xdotool mousemove 560 460; sleep 0.3
      marca down; DISPLAY=$D xdotool mousedown 1; sleep 0.4
      cmd=""; for i in $(seq 1 125); do
        cmd="$cmd mousemove $((560 + i * 32 / 10)) $(awk -v i=$i 'BEGIN{printf "%d", 460+50*sin(i*0.05)}') sleep 0.008"
      done
      marca traco_ini; DISPLAY=$D xdotool $cmd; marca traco_fim
      marca up; DISPLAY=$D xdotool mouseup 1; sleep 3; marca wet_fim
      marca ctrlz; DISPLAY=$D xdotool key ctrl+z; sleep 1.5; marca ctrlz_fim
      marca parado1_ini; sleep 3; marca parado1_fim
      [ -n "$PNG" ] && DISPLAY=$D import -window root "$M/$TAG.png" 2>/dev/null
    fi
    kill -INT $APP 2>/dev/null
    for _ in $(seq 1 40); do kill -0 $APP 2>/dev/null || break; sleep 0.25; done
    kill -KILL $APP 2>/dev/null; wait $APP 2>/dev/null
  done
  kill $XV
  echo "[matriz] feito: $M"
  exit 0
fi

# ── PERF (10/11, diagnóstico do quadro de ~100 ms e do parado): o roteiro da matriz para UM pincel, mas o perf ABRE o
# app (-k CLOCK_MONOTONIC -F 997, sem -g: paranoid 2) e cada marca leva os DOIS relógios (mono = o do perf; parede = o
# do [quadro-lento]). Fases: parado 5 s · toque · traço · soltar · 3 s · Ctrl+Z · 1,5 s · parado 5 s. Mesma placa.
# uso: bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/mede_pen_up_4k.sh perf <WxH> <pincel:painel> [binario]
# Saída: $S/perf/<rotulo>.{perf,log,marcas}; resumo: mede_pen_up_4k.py --perf $S/perf/<rotulo>.
if [ "$1" = perf ]; then
  TELA=$2; PP=$3; BIN=${4:-$R/target/smoke/ph2d-host-desktop}; PIN=${PP%%:*}; PAINEL=${PP##*:}
  P=$S/perf; mkdir -p "$P"; D=${DISPLAY_MATRIZ:-:96}; LENTO=${LENTO_MS:-60}
  TAG="$(echo "${PIN}_${PAINEL}_${TELA}" | tr ' ' '-' | tr -cd 'A-Za-z0-9_.-')"
  EXTRA=(PH2D_COMPOSITE_PILHA=0 PH2D_COMPOSITE_PAPEL=1 "PH2D_COMPOSITE_TELA=$TELA")
  [ "$PIN" != omissão ] && EXTRA+=("PH2D_COMPOSITE_PRESET=$PIN")
  PLACA_ENV=(VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json
    MESA_VK_WSI_DEBUG=sw __GLX_VENDOR_LIBRARY_NAME=mesa __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json)
  Xvfb $D -screen 0 1600x1000x24 >/dev/null 2>&1 & XV=$!
  sleep 1
  cd "$R"
  : > "$P/$TAG.marcas"
  env -u WAYLAND_DISPLAY DISPLAY=$D HOME=$P/home_$TAG PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PAINEL=$PAINEL \
    PH2D_PAINT_PERF=1 PH2D_QUADRO_LENTO_MS=$LENTO PH2D_EXIT_AFTER_FRAMES=1000000 "${PLACA_ENV[@]}" "${EXTRA[@]}" \
    perf record -k CLOCK_MONOTONIC -F 997 -o "$P/$TAG.perf" -- "$BIN" > "$P/$TAG.log" 2>&1 &
  PERF=$!
  APP=; WIN=
  for _ in $(seq 1 120); do
    [ -z "$APP" ] && APP=$(pgrep -P $PERF | head -1)
    grep -q "pilha armada" "$P/$TAG.log" && WIN=$(DISPLAY=$D xdotool search --pid $APP 2>/dev/null | tail -1)
    [ -n "$WIN" ] && break; sleep 0.5
  done
  marca() { echo "$1 $(python3 -c 'import time;print(time.monotonic_ns(), time.time_ns())') $(awk '{print $14+$15}' /proc/$APP/stat 2>/dev/null)" >> "$P/$TAG.marcas"; }
  echo "[perf] $TAG perf $PERF app $APP win ${WIN:-NENHUMA}"
  if [ -n "$WIN" ]; then
    grep -oE '[^/ ]*(nvidia|radeon|swrast|llvmpipe|lvp|gallium|dri)[^/ ]*\.so[^ ]*' /proc/$APP/maps | sort -u \
      | tr '\n' ' ' > "$P/$TAG.placa"
    DISPLAY=$D xdotool windowsize $WIN 1600 1000 windowmove $WIN 0 0 2>/dev/null; sleep 4
    DISPLAY=$D xdotool windowfocus --sync $WIN 2>/dev/null
    marca parado0_ini; sleep 5; marca parado0_fim
    DISPLAY=$D xdotool mousemove 560 460; sleep 0.3
    marca down; DISPLAY=$D xdotool mousedown 1; sleep 0.4
    cmd=""; for i in $(seq 1 125); do
      cmd="$cmd mousemove $((560 + i * 32 / 10)) $(awk -v i=$i 'BEGIN{printf "%d", 460+50*sin(i*0.05)}') sleep 0.008"
    done
    marca traco_ini; DISPLAY=$D xdotool $cmd; marca traco_fim
    marca up; DISPLAY=$D xdotool mouseup 1; sleep 3; marca wet_fim
    marca ctrlz; DISPLAY=$D xdotool key ctrl+z; sleep 1.5; marca ctrlz_fim
    marca parado1_ini; sleep 5; marca parado1_fim
  fi
  kill -INT $APP 2>/dev/null
  for _ in $(seq 1 40); do kill -0 $APP 2>/dev/null || break; sleep 0.25; done
  kill -KILL $APP 2>/dev/null; wait $PERF 2>/dev/null; kill $XV
  echo "[perf] feito: $P/$TAG"
  exit 0
fi

# ── ÓLEO (10/11): o Real Oil não tem porta na cena Composite; mede-se pela cena do FPS (PH2D_WETPAINT_SMOKE=3, o app
# risca sozinho: Dabber com Real Oil, seca, Wet Drip, corre; 6 rodadas; período e imagens da água por segundo).
# uso: bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/mede_pen_up_4k.sh oleo <lado> [binario]
#      (lado 2880 = os píxeis de 3840x2160; 1440 = os de 1920x1080). Mesma placa da matriz. Saída: $S/oleo_<lado>.log
if [ "$1" = oleo ]; then
  LADO=$2; BIN=${3:-$R/target/smoke/ph2d-host-desktop}; D=${DISPLAY_MATRIZ:-:97}
  PLACA_ENV=(VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json
    MESA_VK_WSI_DEBUG=sw __GLX_VENDOR_LIBRARY_NAME=mesa __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json)
  Xvfb $D -screen 0 1600x1000x24 >/dev/null 2>&1 & XV=$!
  sleep 1
  cd "$R"
  L=$S/oleo_$LADO.log
  env -u WAYLAND_DISPLAY DISPLAY=$D HOME=$S/home_oleo_$LADO PH2D_WETPAINT_SMOKE=3 PH2D_FPS_OLEO=1 PH2D_FPS_TELA=$LADO \
    PH2D_EXIT_AFTER_FRAMES=1000000 "${PLACA_ENV[@]}" "$BIN" > "$L" 2>&1 &
  APP=$!
  for _ in $(seq 1 360); do grep -q "^\[fps\] ===" "$L" && break; kill -0 $APP 2>/dev/null || break; sleep 0.5; done
  sleep 1; kill -INT $APP 2>/dev/null
  for _ in $(seq 1 40); do kill -0 $APP 2>/dev/null || break; sleep 0.25; done
  kill -KILL $APP 2>/dev/null; wait $APP 2>/dev/null; kill $XV
  grep "^\[fps\]" "$L"
  exit 0
fi

BIN=${1:-$R/target/smoke/ph2d-host-desktop}; TAG=${2:-x}; TELA=${3:-3840x2160}; PAINEL=${4:-wet}
D=:94
mono() { python3 -c 'import time;print(time.monotonic_ns())'; }
Xvfb $D -screen 0 1600x1000x24 >/dev/null 2>&1 & XV=$!
sleep 1
cd "$R"
# O perf ABRE o app (o ptrace_scope do Yama recusa o `perf record -p` num processo que não é filho). Sem `-g`: com o
# perf_event_paranoid 2 a cadeia de chamadas não abre (medido 10/11) — a amostra é a função FOLHA.
env -u WAYLAND_DISPLAY DISPLAY=$D HOME=$S/home_$TAG PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PILHA=0 \
  PH2D_COMPOSITE_PAINEL=$PAINEL PH2D_COMPOSITE_PAPEL=1 PH2D_COMPOSITE_TELA=$TELA PH2D_PAINT_PERF=1 \
  PH2D_QUADRO_LENTO_MS=${LENTO_MS:-40} PH2D_EXIT_AFTER_FRAMES=100000 perf record -k CLOCK_MONOTONIC -F 997 -o $S/$TAG.perf -- "$BIN" \
  > $S/$TAG.log 2>&1 & PERF=$!
sleep 15
APP=$(pgrep -P $PERF | head -1)
WIN=$(DISPLAY=$D xdotool search --pid $APP 2>/dev/null | tail -1)
echo "[pen-up] app $APP win $WIN tela $TELA painel $PAINEL"
DISPLAY=$D xdotool windowsize $WIN 1600 1000 windowmove $WIN 0 0 2>/dev/null; sleep 3
: > $S/$TAG.ups; : > $S/$TAG.ups_parede
for t in 1 2 3 4 5 6; do
  cmd="mousemove 760 460 mousedown 1"
  for i in $(seq 0 40); do cmd="$cmd mousemove $((760 + i * 2)) $((460 + (i % 7))) sleep 0.008"; done
  DISPLAY=$D xdotool $cmd
  mono >> $S/$TAG.ups; date +%s%N >> $S/$TAG.ups_parede
  DISPLAY=$D xdotool mouseup 1
  sleep 2.5
done
DISPLAY=$D import -window root $S/$TAG.png 2>/dev/null
kill -INT $APP; wait $PERF; kill $XV
echo "[pen-up] feito: $S/$TAG.{log,perf,ups,png}"
