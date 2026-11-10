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
# soltar · 3 s (Wet: imagens/s) · Ctrl+Z · parado 3 s · CONTROLO (SIGSTOP 250 ms: a régua tem de o ver) · trocar de
# camada (aba Layers · «+» · a de baixo · a de cima · duplicar · apagar · grupo; CAMADA_*="x y", fotos com PNG=1) ·
# slider de luz (painel impasto; LUZ="x0 y x1" arrasta a PEGA de um número, LUZ_ROLA cliques da roda: 80 = o fundo do
# cartão, onde a pega do Intensity fica em 1332,193) · abrir doc (ABRIR="x y" = «Composite 2» na Hierarquia, 117 301;
# REABRIR volta à «Composite»; PH2D_COMPOSITE_SEGUNDA);
# as coordenadas de omissão são as da janela 1600x1000, ver ANTES.md §B08b). `blocos` = a imagem da cena (degradê, grelha, discos) a WxH;
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
    [ -n "$ABRIR" ] && EXTRA+=(PH2D_COMPOSITE_SEGUNDA=1)
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
      # B08.1, o CONTROLO da régua: o app parado 250 ms por SIGSTOP (o processo é nosso). O intervalo de quadro desse
      # segundo tem de ler ≥ 250 ms; se não lê, a régua não vê uma trava e a matriz vale zero (mede_pen_up_4k.py ✗).
      marca controlo; kill -STOP $APP; sleep 0.25; kill -CONT $APP; sleep 1; marca controlo_fim
      [ -n "$PNG" ] && DISPLAY=$D import -window root "$M/$TAG.png" 2>/dev/null
      # B08.1, trocar de camada: cliques no cartão BRUSH (janela 1600x1000 em 0,0): aba Layers · «+» (camada nova, que
      # fica a ativa) · a camada de baixo · a de cima. Uma foto por passo (PNG=1) confere que o clique caiu no sítio.
      foto() { [ -n "$PNG" ] && DISPLAY=$D import -window root "$M/$TAG.$1.png" 2>/dev/null; }
      marca camada_aba; DISPLAY=$D xdotool mousemove ${CAMADA_ABA:-1510 159} click 1; sleep 1.5; foto camada1_aba
      marca camada_mais; DISPLAY=$D xdotool mousemove ${CAMADA_MAIS:-1332 233} click 1; sleep 1.5; foto camada2_mais
      marca camada_troca; DISPLAY=$D xdotool mousemove ${CAMADA_OUTRA:-1430 397} click 1; sleep 1.5; foto camada3_troca
      marca camada_volta; DISPLAY=$D xdotool mousemove ${CAMADA_VOLTA:-1430 306} click 1; sleep 1.5; foto camada4_volta
      marca camada_dup; DISPLAY=$D xdotool mousemove ${CAMADA_DUP:-1389 233} click 1; sleep 1.5; foto camada5_dup
      marca camada_apaga; DISPLAY=$D xdotool mousemove ${CAMADA_APAGA:-1417 233} click 1; sleep 1.5; foto camada6_apaga
      marca camada_grupo; DISPLAY=$D xdotool mousemove ${CAMADA_GRUPO:-1361 233} click 1; sleep 1.5; foto camada7_grupo
      marca camada_fim
      # B08.1, o slider de luz do Impasto (só no painel impasto): aba Brush, rolar o cartão até «Lighting» (LUZ_ROLA
      # cliques da roda), arrastar o número da luz (LUZ="x0 y x1": 30 passos × 16 ms). LUZ_SONDA=n: n fotos a cada 5
      # cliques da roda, para achar as coordenadas.
      if [ "$PAINEL" = impasto ]; then
        DISPLAY=$D xdotool mousemove ${LUZ_ABA:-1376 159} click 1; sleep 1; DISPLAY=$D xdotool mousemove ${LUZ_ROLA_EM:-1440 600}
        for _ in $(seq 1 "${LUZ_ROLA:-0}"); do DISPLAY=$D xdotool click 5; sleep 0.05; done; sleep 1; foto luz1_cartao
        for k in $(seq 1 "${LUZ_SONDA:-0}"); do
          for _ in 1 2 3 4 5; do DISPLAY=$D xdotool click 5; sleep 0.05; done; sleep 0.7; foto "luz_sonda$k"
        done
        if [ -n "$LUZ" ]; then
          read -r LX0 LY LX1 <<< "$LUZ"; cmd="mousemove $LX0 $LY mousedown 1"
          for i in $(seq 1 30); do cmd="$cmd mousemove $((LX0 + (LX1 - LX0) * i / 30)) $LY sleep 0.016"; done
          marca luz; DISPLAY=$D xdotool $cmd mouseup 1; marca luz_solta; sleep 1; marca luz_fim; foto luz2_arrasto
        fi
      fi
      # B08.1, abrir doc WxH: ABRIR="x y" = a linha «Composite 2» na Hierarquia (a 2.ª tela, PH2D_COMPOSITE_SEGUNDA)
      # liga um doc novo no Painter; REABRIR="x y" = a «Composite» (volta ao doc guardado). 2 s cada.
      if [ -n "$ABRIR" ]; then
        marca abrir; DISPLAY=$D xdotool mousemove $ABRIR click 1; sleep 2; foto abrir1_novo
        marca reabrir; DISPLAY=$D xdotool mousemove ${REABRIR:-117 278} click 1; sleep 2; foto abrir2_volta
        marca abrir_fim
      fi
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

# ── PARADO (B23, 10/11): CPU do processo com o app aberto e sem entrada, SEM smoke (o smoke segura o laço contínuo).
# A × B no MESMO binário, intercalados: a pedido (omissão) × `PH2D_QUADRO_A_PEDIDO=0` (o laço contínuo de antes).
# uso: bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/mede_pen_up_4k.sh parado [rodadas] [segundos] [binario]
if [ "$1" = parado ]; then
  N=${2:-3}; SEG=${3:-10}; BIN=${4:-$R/target/smoke/ph2d-host-desktop}; D=${DISPLAY_MATRIZ:-:97}
  PLACA_ENV=(VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json
    MESA_VK_WSI_DEBUG=sw __GLX_VENDOR_LIBRARY_NAME=mesa __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json)
  Xvfb $D -screen 0 1600x1000x24 >/dev/null 2>&1 & XV=$!
  sleep 1
  cd "$R"
  tiques() { awk '{print $14+$15}' "/proc/$1/stat"; }
  for i in $(seq 1 "$N"); do for modo in pedido continuo; do
    VAR=(); [ $modo = continuo ] && VAR=(PH2D_QUADRO_A_PEDIDO=0)
    env -u WAYLAND_DISPLAY DISPLAY=$D HOME=$S/home_parado "${PLACA_ENV[@]}" "${VAR[@]}" "$BIN" > "$S/parado_$modo.log" 2>&1 &
    APP=$!; sleep 8
    a=$(tiques $APP); sleep "$SEG"; b=$(tiques $APP)
    echo "[parado] rodada=$i modo=$modo cpu=$(( (b - a) * 100 / $(getconf CLK_TCK) / SEG ))% de um núcleo em ${SEG}s"
    kill -INT $APP 2>/dev/null
    for _ in $(seq 1 40); do kill -0 $APP 2>/dev/null || break; sleep 0.25; done
    kill -KILL $APP 2>/dev/null; wait $APP 2>/dev/null
  done; done
  kill $XV
  exit 0
fi

# ── OCULTO (B23.3/B23.5, 10/11): q/s, quadro p50 e CPU com a janela à vista, desfocada e oculta (uma janela Tk por
# cima: o X manda VisibilityNotify FullyObscured = `Occluded(true)`). Variantes intercaladas no MESMO binário:
# `salta` = smoke 4K Textured com PH2D_OCULTA_APRESENTA=0 (a lei fora do smoke) · `apresenta` = o mesmo com `=1`
# (o present de antes) · `pedido` = o app sem smoke (a lei a pedido). Resumo por fase no fim de cada lançamento, e
# quantos `[quadro] oculta=true` o app viu (0 = o `Occluded` não chegou: a fase «oculto» não mede nada).
# uso: bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/mede_pen_up_4k.sh oculto [rodadas] [segundos] [binario]
if [ "$1" = oculto ]; then
  N=${2:-2}; SEG=${3:-5}; BIN=${4:-$R/target/smoke/ph2d-host-desktop}; D=${DISPLAY_MATRIZ:-:97}; O=$S/oculto
  mkdir -p "$O"
  PLACA_ENV=(VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json
    MESA_VK_WSI_DEBUG=sw __GLX_VENDOR_LIBRARY_NAME=mesa __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json)
  Xvfb $D -screen 0 1600x1000x24 >/dev/null 2>&1 & XV=$!
  sleep 1
  cd "$R"
  tapa() { DISPLAY=$D python3 -I -c "import tkinter as t; r=t.Tk(); r.overrideredirect(1); r.geometry('$1'); r.mainloop()" & TK=$!; sleep 1.5; }
  for i in $(seq 1 "$N"); do for modo in salta apresenta pedido; do
    TAG=${modo}_$i; L=$O/$TAG.log; : > "$O/$TAG.marcas"
    VAR=(PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PAINEL=wet PH2D_COMPOSITE_PILHA=0 PH2D_COMPOSITE_FUNDO=papel
      PH2D_COMPOSITE_TELA=3840x2160 PH2D_COMPOSITE_PAPEL=1 PH2D_EXIT_AFTER_FRAMES=1000000)
    # O smoke apresenta sempre por omissão (`Laco::apresenta`): o salto pede o `=0` explícito.
    [ $modo = salta ] && VAR+=(PH2D_OCULTA_APRESENTA=0)
    [ $modo = apresenta ] && VAR+=(PH2D_OCULTA_APRESENTA=1)
    [ $modo = pedido ] && VAR=()
    env -u WAYLAND_DISPLAY DISPLAY=$D HOME=$O/home_$TAG PH2D_PAINT_PERF=1 PH2D_QUADRO_LENTO_MS=0 "${PLACA_ENV[@]}" "${VAR[@]}" "$BIN" \
      > "$L" 2>&1 &
    APP=$!; WIN=
    for _ in $(seq 1 120); do
      { [ $modo = pedido ] || grep -q "pilha armada" "$L"; } && WIN=$(DISPLAY=$D xdotool search --pid $APP 2>/dev/null | tail -1)
      [ -n "$WIN" ] && break; sleep 0.5
    done
    marca() { echo "$1 $(date +%s%N) $(awk '{print $14+$15}' /proc/$APP/stat 2>/dev/null)" >> "$O/$TAG.marcas"; }
    if [ -n "$WIN" ]; then
      DISPLAY=$D xdotool windowsize $WIN 1600 1000 windowmove $WIN 0 0 windowfocus --sync $WIN 2>/dev/null; sleep 4
      marca visivel_ini; sleep "$SEG"; marca visivel_fim
      tapa 160x90+1440+910; DISPLAY=$D xdotool search --pid $TK 2>/dev/null | tail -1 | xargs -r -I{} env DISPLAY=$D xdotool windowfocus {}
      marca desfocado_ini; sleep "$SEG"; marca desfocado_fim; kill $TK; wait $TK 2>/dev/null
      tapa 1600x1000+0+0
      marca oculto_ini; sleep "$SEG"; marca oculto_fim; kill $TK; wait $TK 2>/dev/null
    fi
    kill -INT $APP 2>/dev/null
    for _ in $(seq 1 40); do kill -0 $APP 2>/dev/null || break; sleep 0.25; done
    kill -KILL $APP 2>/dev/null; wait $APP 2>/dev/null
    # Por fase: quadros/s, quadro p50 (ms, o relógio do quadro), CPU % de um núcleo (tiques do processo).
    awk -v hz="$(getconf CLK_TCK)" -v tag="$TAG" -v win="${WIN:-NENHUMA}" '
      FNR == NR { split($1, f, "_"); if (f[2] == "ini") { a[f[1]] = $2; ta[f[1]] = $3; ordem[++n] = f[1] } else { b[f[1]] = $2; tb[f[1]] = $3 }; next }
      /^\[quadro-lento\]/ { sub("parede=", "", $2); sub("quadro=", "", $4); for (k in a) if ($2 >= a[k] && $2 <= b[k]) q[k, ++c[k]] = $4 + 0 }
      END { for (j = 1; j <= n; j++) { k = ordem[j]; s = (b[k] - a[k]) / 1e9; m = c[k] + 0
        for (x = 1; x <= m; x++) v[x] = q[k, x]; asort(v); p50 = m ? v[int((m + 1) / 2)] : 0; delete v
        printf "[oculto] %s win=%s %s q/s=%.1f p50=%.1f ms cpu=%d%%\n", tag, win, k, m / s, p50, (tb[k] - ta[k]) * 100 / hz / s } }
    ' "$O/$TAG.marcas" "$L" | tee -a "$O/resumo.txt"
    echo "[oculto] $TAG occluded=$(grep -c '^\[quadro\] oculta=true' "$L")" | tee -a "$O/resumo.txt"
  done; done
  kill $XV
  exit 0
fi

# ── WAYLAND (B23.6, 10/11): o mesmo roteiro do OCULTO num `kwin_wayland --virtual` (QPainter: o kwin não entra na
# placa) sob `dbus-run-session` (o bus é desta corrida, nunca o do dono); minimizar/restaurar pelo script do KWin.
# Conta `[quadro] oculta=true|false` (o `suspended` do xdg_toplevel que o winit vendorizado faz `Occluded`).
# uso: bash scripts/ph2d-run.sh dbus-run-session -- bash docs/Painter/ferramentas/mede_pen_up_4k.sh wayland [rodadas] [segundos] [binario]
if [ "$1" = wayland ]; then
  N=${2:-1}; SEG=${3:-5}; BIN=${4:-$R/target/smoke/ph2d-host-desktop}; O=$S/wayland; SOCK=ph2d-b23-$$
  mkdir -p "$O"
  # WSI_SW=1 (o do Xvfb) no kwin em QPainter: o RADV perde o dispositivo ao configurar a superfície (medido 10/11:
  # `DeviceLost(Unknown)` logo depois do `present mode`, 0 quadros). Por omissão o kwin compõe em OpenGL no Mesa
  # (KWIN_COMPOSE=O) e o app apresenta por dmabuf, como na sessão do dono.
  KC=${KWIN_COMPOSE:-O}
  PLACA_ENV=(VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/radeon_icd.json
    __GLX_VENDOR_LIBRARY_NAME=mesa __EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json)
  [ "${WSI_SW:-0}" = 1 ] && PLACA_ENV+=(MESA_VK_WSI_DEBUG=sw)
  # O kwin em OpenGL no nó da Radeon (vendor 0x1002): noutro nó ele recusa os dmabufs do app (protocol error 7).
  NO=; for c in /sys/class/drm/renderD*; do [ "$(cat "$c/device/vendor")" = 0x1002 ] && NO=/dev/dri/${c##*/}; done
  bash "$R/scripts/vez-da-placa.sh" || exit $?
  env -u DISPLAY -u WAYLAND_DISPLAY KWIN_COMPOSE=$KC KWIN_RENDER_NODES=$NO "${PLACA_ENV[@]}" kwin_wayland --virtual --width 1600 --height 1000 \
    --socket "$SOCK" --no-lockscreen > "$O/kwin.log" 2>&1 & KW=$!
  for _ in $(seq 1 40); do [ -S "$XDG_RUNTIME_DIR/$SOCK" ] && break; sleep 0.25; done
  cd "$R"
  minimiza() { echo "for (const w of workspace.windowList()) if (w.pid === $APP) w.minimized = $1;" > "$O/min.js"
    local id; id=$(dbus-send --session --print-reply --dest=org.kde.KWin /Scripting org.kde.kwin.Scripting.loadScript \
      string:"$O/min.js" string:"ph2d$1$k" | awk '/int32/ {print $2}'); k=$((k + 1))
    dbus-send --session --dest=org.kde.KWin "/Scripting/Script$id" org.kde.kwin.Script.run; sleep 1; }
  k=0
  for i in $(seq 1 "$N"); do for modo in salta pedido; do
    TAG=${modo}_$i; L=$O/$TAG.log; : > "$O/$TAG.marcas"
    VAR=(PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PAINEL=wet PH2D_COMPOSITE_PILHA=0 PH2D_COMPOSITE_FUNDO=papel
      PH2D_COMPOSITE_TELA=3840x2160 PH2D_COMPOSITE_PAPEL=1 PH2D_EXIT_AFTER_FRAMES=1000000 PH2D_OCULTA_APRESENTA=0)
    [ $modo = pedido ] && VAR=()
    env -u DISPLAY WAYLAND_DISPLAY=$SOCK HOME=$O/home_$TAG PH2D_PAINT_PERF=1 PH2D_QUADRO_LENTO_MS=0 "${PLACA_ENV[@]}" \
      "${VAR[@]}" "$BIN" > "$L" 2>&1 &
    APP=$!
    for _ in $(seq 1 120); do { [ $modo = pedido ] || grep -q "pilha armada" "$L"; } && break
      kill -0 $APP 2>/dev/null || break; sleep 0.5; done
    sleep 4
    marca() { echo "$1 $(date +%s%N) $(awk '{print $14+$15}' /proc/$APP/stat 2>/dev/null)" >> "$O/$TAG.marcas"; }
    marca visivel_ini; sleep "$SEG"; marca visivel_fim
    minimiza true; marca minimizado_ini; sleep "$SEG"; marca minimizado_fim
    minimiza false; marca restaurado_ini; sleep "$SEG"; marca restaurado_fim
    kill -INT $APP 2>/dev/null
    for _ in $(seq 1 40); do kill -0 $APP 2>/dev/null || break; sleep 0.25; done
    kill -KILL $APP 2>/dev/null; wait $APP 2>/dev/null
    awk -v hz="$(getconf CLK_TCK)" -v tag="$TAG" '
      FNR == NR { split($1, f, "_"); if (f[2] == "ini") { a[f[1]] = $2; ta[f[1]] = $3; ordem[++n] = f[1] } else { b[f[1]] = $2; tb[f[1]] = $3 }; next }
      /^\[quadro-lento\]/ { sub("parede=", "", $2); sub("quadro=", "", $4); for (k in a) if ($2 >= a[k] && $2 <= b[k]) q[k, ++c[k]] = $4 + 0 }
      END { for (j = 1; j <= n; j++) { k = ordem[j]; s = (b[k] - a[k]) / 1e9; m = c[k] + 0
        for (x = 1; x <= m; x++) v[x] = q[k, x]; asort(v); p50 = m ? v[int((m + 1) / 2)] : 0; delete v
        printf "[wayland] %s %s q/s=%.1f p50=%.1f ms cpu=%d%%\n", tag, k, m / s, p50, (tb[k] - ta[k]) * 100 / hz / s } }
    ' "$O/$TAG.marcas" "$L" | tee -a "$O/resumo.txt"
    echo "[wayland] $TAG oculta=true:$(grep -c '^\[quadro\] oculta=true' "$L") oculta=false:$(grep -c '^\[quadro\] oculta=false' "$L")" \
      | tee -a "$O/resumo.txt"
  done; done
  kill $KW; wait $KW 2>/dev/null
  # O kwin activa serviços no bus DESTA corrida (knighttimed, 10/11) que sobrevivem e seguram o stdout: só esses.
  for p in /proc/[0-9]*; do [ "${p#/proc/}" = $$ ] && continue
    grep -qzaxF "DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS" "$p/environ" 2>/dev/null && kill "${p#/proc/}" 2>/dev/null
  done
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
